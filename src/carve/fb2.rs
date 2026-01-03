//! FB2 (FictionBook) carving handler.
//!
//! Validates presence of <FictionBook> and scans for closing tag.

use std::fs::File;
use std::io::{BufWriter, Write};

use sha2::{Digest, Sha256};

use crate::carve::{CarveError, CarveHandler, CarvedFile, ExtractionContext, output_path};
use crate::scanner::NormalizedHit;

const FB2_HEADER: &[u8] = b"<?xml";
const FB2_TAG: &[u8] = b"<FictionBook";
const FB2_END: &[u8] = b"</FictionBook>";

pub struct Fb2CarveHandler {
    extension: String,
    min_size: u64,
    max_size: u64,
}

impl Fb2CarveHandler {
    pub fn new(extension: String, min_size: u64, max_size: u64) -> Self {
        Self {
            extension,
            min_size,
            max_size,
        }
    }
}

impl CarveHandler for Fb2CarveHandler {
    fn file_type(&self) -> &str {
        "fb2"
    }

    fn extension(&self) -> &str {
        &self.extension
    }

    fn process_hit(
        &self,
        hit: &NormalizedHit,
        ctx: &ExtractionContext,
    ) -> Result<Option<CarvedFile>, CarveError> {
        let (full_path, rel_path) = output_path(
            ctx.output_root,
            self.file_type(),
            &self.extension,
            hit.global_offset,
        )?;
        let file = File::create(&full_path)?;
        let mut writer = BufWriter::new(file);
        let mut md5 = md5::Context::new();
        let mut sha256 = Sha256::new();

        let mut validated = false;
        let mut truncated = false;
        let mut errors = Vec::new();

        let mut offset = hit.global_offset;
        let buf_size = 64 * 1024;
        let mut bytes_written = 0u64;
        let mut carry: Vec<u8> = Vec::new();
        let mut saw_tag = false;

        loop {
            if self.max_size > 0 && bytes_written >= self.max_size {
                truncated = true;
                errors.push("max_size reached before fb2 end".to_string());
                break;
            }

            let remaining = if self.max_size > 0 {
                (self.max_size - bytes_written).min(buf_size as u64)
            } else {
                buf_size as u64
            };

            let mut buf = vec![0u8; remaining as usize];
            let n = ctx
                .evidence
                .read_at(offset, &mut buf)
                .map_err(|e| CarveError::Evidence(e.to_string()))?;
            if n == 0 {
                truncated = true;
                errors.push("eof before fb2 end".to_string());
                break;
            }
            buf.truncate(n);

            if bytes_written == 0 {
                if buf.len() < FB2_HEADER.len() || &buf[..FB2_HEADER.len()] != FB2_HEADER {
                    let _ = std::fs::remove_file(&full_path);
                    return Ok(None);
                }
            }

            if !saw_tag {
                let mut tag_scan = carry.clone();
                tag_scan.extend_from_slice(&buf);
                if find_pattern(&tag_scan, FB2_TAG).is_some() {
                    saw_tag = true;
                }
            }

            let mut search_buf = carry.clone();
            search_buf.extend_from_slice(&buf);
            if let Some(pos) = find_pattern(&search_buf, FB2_END) {
                let write_len = if pos < carry.len() {
                    pos + FB2_END.len() - carry.len()
                } else {
                    pos - carry.len() + FB2_END.len()
                };
                let write_len = write_len.min(buf.len());
                if write_len > 0 {
                    let slice = &buf[..write_len];
                    writer.write_all(slice)?;
                    md5.consume(slice);
                    sha256.update(slice);
                    bytes_written = bytes_written.saturating_add(slice.len() as u64);
                }
                validated = true;
                break;
            }

            writer.write_all(&buf)?;
            md5.consume(&buf);
            sha256.update(&buf);
            bytes_written = bytes_written.saturating_add(buf.len() as u64);
            offset = offset.saturating_add(buf.len() as u64);

            if FB2_END.len() > 1 {
                let keep = FB2_END.len() - 1;
                if buf.len() >= keep {
                    carry = buf[buf.len() - keep..].to_vec();
                } else {
                    carry = buf.clone();
                }
            } else {
                carry.clear();
            }
        }

        writer.flush()?;

        if !saw_tag {
            let _ = std::fs::remove_file(&full_path);
            return Ok(None);
        }

        if bytes_written < self.min_size {
            let _ = std::fs::remove_file(&full_path);
            return Ok(None);
        }

        let md5_hex = format!("{:x}", md5.compute());
        let sha256_hex = hex::encode(sha256.finalize());
        let global_end = if bytes_written == 0 {
            hit.global_offset
        } else {
            hit.global_offset + bytes_written - 1
        };

        Ok(Some(CarvedFile {
            run_id: ctx.run_id.to_string(),
            file_type: self.file_type().to_string(),
            path: rel_path,
            extension: self.extension.clone(),
            global_start: hit.global_offset,
            global_end,
            size: bytes_written,
            md5: Some(md5_hex),
            sha256: Some(sha256_hex),
            validated,
            truncated,
            errors,
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

#[cfg(test)]
mod tests {
    use super::Fb2CarveHandler;
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
    fn carves_fb2_with_footer() {
        let data = b"<?xml version='1.0'?><FictionBook>hi</FictionBook>".to_vec();
        let evidence = SliceEvidence { data: data.clone() };
        let handler = Fb2CarveHandler::new("fb2".to_string(), 0, 0);
        let hit = NormalizedHit {
            global_offset: 0,
            file_type_id: "fb2".to_string(),
            pattern_id: "fb2_xml".to_string(),
        };
        let dir = tempdir().expect("tempdir");
        let ctx = ExtractionContext {
            run_id: "test",
            output_root: dir.path(),
            evidence: &evidence,
        };

        let carved = handler.process_hit(&hit, &ctx).expect("process");
        let carved = carved.expect("carved");
        assert!(carved.validated);
        assert_eq!(carved.size, data.len() as u64);
    }
}
