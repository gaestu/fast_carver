//! EML carving handler.
//!
//! Heuristically scans email headers and stops at the next "From " boundary or EOF.

use std::fs::File;

use sha2::{Digest, Sha256};

use crate::carve::{
    output_path, write_range, CarveError, CarveHandler, CarvedFile, ExtractionContext,
};
use crate::scanner::NormalizedHit;

const HEADER_MARKERS: [&[u8]; 4] = [b"From:", b"To:", b"Subject:", b"Date:"];
const MBOX_BOUNDARY: &[u8] = b"\nFrom ";

pub struct EmlCarveHandler {
    extension: String,
    min_size: u64,
    max_size: u64,
}

impl EmlCarveHandler {
    pub fn new(extension: String, min_size: u64, max_size: u64) -> Self {
        Self {
            extension,
            min_size,
            max_size,
        }
    }
}

impl CarveHandler for EmlCarveHandler {
    fn file_type(&self) -> &str {
        "eml"
    }

    fn extension(&self) -> &str {
        &self.extension
    }

    fn process_hit(
        &self,
        hit: &NormalizedHit,
        ctx: &ExtractionContext,
    ) -> Result<Option<CarvedFile>, CarveError> {
        let head = read_prefix(ctx, hit.global_offset, 512);
        if head.is_empty() {
            return Ok(None);
        }
        if !HEADER_MARKERS
            .iter()
            .any(|m| find_pattern(&head, m).is_some())
        {
            return Ok(None);
        }

        let max_end = if self.max_size > 0 {
            hit.global_offset.saturating_add(self.max_size)
        } else {
            u64::MAX
        };

        let mut offset = hit.global_offset;
        let mut end_offset = None;
        let mut carry: Vec<u8> = Vec::new();
        let buf_size = 64 * 1024;

        while offset < max_end {
            let remaining = (max_end - offset).min(buf_size as u64) as usize;
            let mut buf = vec![0u8; remaining];
            let n = ctx
                .evidence
                .read_at(offset, &mut buf)
                .map_err(|e| CarveError::Evidence(e.to_string()))?;
            if n == 0 {
                end_offset = Some(offset);
                break;
            }
            buf.truncate(n);

            let mut search_buf = carry.clone();
            search_buf.extend_from_slice(&buf);
            if let Some(pos) = find_pattern(&search_buf, MBOX_BOUNDARY) {
                let boundary = offset
                    .saturating_sub(carry.len() as u64)
                    .saturating_add(pos as u64);
                if boundary > hit.global_offset {
                    end_offset = Some(boundary);
                    break;
                }
            }

            offset = offset.saturating_add(buf.len() as u64);
            if buf.len() >= MBOX_BOUNDARY.len() - 1 {
                carry = buf[buf.len() - (MBOX_BOUNDARY.len() - 1)..].to_vec();
            } else {
                carry = buf;
            }
        }

        let end_offset = end_offset.unwrap_or(max_end);
        let (full_path, rel_path) = output_path(
            ctx.output_root,
            self.file_type(),
            &self.extension,
            hit.global_offset,
        )?;
        let mut file = File::create(&full_path)?;
        let mut md5 = md5::Context::new();
        let mut sha256 = Sha256::new();

        let (written, eof_truncated) = write_range(
            ctx,
            hit.global_offset,
            end_offset,
            &mut file,
            &mut md5,
            &mut sha256,
        )?;

        if written < self.min_size {
            let _ = std::fs::remove_file(&full_path);
            return Ok(None);
        }

        let md5_hex = format!("{:x}", md5.compute());
        let sha256_hex = hex::encode(sha256.finalize());
        let global_end = if written == 0 {
            hit.global_offset
        } else {
            hit.global_offset + written - 1
        };

        Ok(Some(CarvedFile {
            run_id: ctx.run_id.to_string(),
            file_type: self.file_type().to_string(),
            path: rel_path,
            extension: self.extension.clone(),
            global_start: hit.global_offset,
            global_end,
            size: written,
            md5: Some(md5_hex),
            sha256: Some(sha256_hex),
            validated: !eof_truncated,
            truncated: eof_truncated,
            errors: Vec::new(),
            pattern_id: Some(hit.pattern_id.clone()),
        }))
    }
}

fn find_pattern(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let first = needle[0];
    let mut i = 0usize;
    while i + needle.len() <= haystack.len() {
        if haystack[i] == first && &haystack[i..i + needle.len()] == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn read_prefix(ctx: &ExtractionContext, offset: u64, len: usize) -> Vec<u8> {
    let mut buf = vec![0u8; len];
    let n = ctx.evidence.read_at(offset, &mut buf).ok().unwrap_or(0);
    buf.truncate(n);
    buf
}

#[cfg(test)]
mod tests {
    use super::EmlCarveHandler;
    use crate::carve::{CarveHandler, ExtractionContext};
    use crate::evidence::{EvidenceError, EvidenceSource};
    use crate::scanner::NormalizedHit;
    use tempfile::tempdir;

    struct SliceEvidence {
        data: Vec<u8>,
    }

    impl EvidenceSource for SliceEvidence {
        fn len(&self) -> u64 {
            self.data.len() as u64
        }

        fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, EvidenceError> {
            if offset as usize >= self.data.len() {
                return Ok(0);
            }
            let max = self.data.len() - offset as usize;
            let to_copy = buf.len().min(max);
            buf[..to_copy].copy_from_slice(&self.data[offset as usize..offset as usize + to_copy]);
            Ok(to_copy)
        }
    }

    #[test]
    fn carves_simple_eml() {
        let data = b"From: a\nTo: b\nSubject: c\n\nBody".to_vec();
        let evidence = SliceEvidence { data: data.clone() };
        let handler = EmlCarveHandler::new("eml".to_string(), 0, 0);
        let hit = NormalizedHit {
            global_offset: 0,
            file_type_id: "eml".to_string(),
            pattern_id: "eml_from".to_string(),
        };
        let dir = tempdir().expect("tempdir");
        let ctx = ExtractionContext {
            run_id: "test",
            output_root: dir.path(),
            evidence: &evidence,
        };

        let carved = handler.process_hit(&hit, &ctx).expect("process");
        let carved = carved.expect("carved");
        assert_eq!(carved.size, data.len() as u64);
    }
}
