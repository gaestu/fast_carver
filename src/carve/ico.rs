//! ICO/CUR carving handler.
//!
//! ICO files have a small header with directory entries containing offsets/sizes.

use std::fs::File;

use sha2::{Digest, Sha256};

use crate::carve::{
    CarveError, CarveHandler, CarvedFile, ExtractionContext, output_path, write_range,
};
use crate::scanner::NormalizedHit;

pub struct IcoCarveHandler {
    extension: String,
    min_size: u64,
    max_size: u64,
}

impl IcoCarveHandler {
    pub fn new(extension: String, min_size: u64, max_size: u64) -> Self {
        Self {
            extension,
            min_size,
            max_size,
        }
    }
}

impl CarveHandler for IcoCarveHandler {
    fn file_type(&self) -> &str {
        "ico"
    }

    fn extension(&self) -> &str {
        &self.extension
    }

    fn process_hit(
        &self,
        hit: &NormalizedHit,
        ctx: &ExtractionContext,
    ) -> Result<Option<CarvedFile>, CarveError> {
        let header = read_exact_at(ctx, hit.global_offset, 6)
            .ok_or_else(|| CarveError::Invalid("ico header too short".to_string()))?;
        if header[0] != 0 || header[1] != 0 {
            return Ok(None);
        }
        let icon_type = u16::from_le_bytes([header[2], header[3]]);
        if icon_type != 1 && icon_type != 2 {
            return Ok(None);
        }
        let count = u16::from_le_bytes([header[4], header[5]]) as usize;
        if count == 0 || count > 256 {
            return Ok(None);
        }

        let dir_len = count * 16;
        let dir = read_exact_at(ctx, hit.global_offset + 6, dir_len)
            .ok_or_else(|| CarveError::Invalid("ico directory truncated".to_string()))?;
        let mut max_end = 0u64;
        let header_size = 6u64 + dir_len as u64;
        for i in 0..count {
            let base = i * 16;
            let size =
                u32::from_le_bytes([dir[base + 8], dir[base + 9], dir[base + 10], dir[base + 11]])
                    as u64;
            let offset = u32::from_le_bytes([
                dir[base + 12],
                dir[base + 13],
                dir[base + 14],
                dir[base + 15],
            ]) as u64;
            if size == 0 || offset < header_size {
                return Ok(None);
            }
            max_end = max_end.max(offset.saturating_add(size));
        }

        let mut total_end = hit.global_offset.saturating_add(max_end);
        if self.max_size > 0 {
            let max_allowed = hit.global_offset.saturating_add(self.max_size);
            if total_end > max_allowed {
                total_end = max_allowed;
            }
        }

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
            total_end,
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

fn read_exact_at(ctx: &ExtractionContext, offset: u64, len: usize) -> Option<Vec<u8>> {
    let mut buf = vec![0u8; len];
    let n = ctx.evidence.read_at(offset, &mut buf).ok()?;
    if n < len {
        return None;
    }
    Some(buf)
}

#[cfg(test)]
mod tests {
    use super::IcoCarveHandler;
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
    fn carves_minimal_ico() {
        let mut data = Vec::new();
        data.extend_from_slice(&[0x00, 0x00, 0x01, 0x00]);
        data.extend_from_slice(&[0x01, 0x00]);
        data.extend_from_slice(&[16, 16, 0, 0]);
        data.extend_from_slice(&[1, 0]);
        data.extend_from_slice(&[32, 0]);
        data.extend_from_slice(&(4u32).to_le_bytes());
        data.extend_from_slice(&(22u32).to_le_bytes());
        data.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

        let evidence = SliceEvidence { data: data.clone() };
        let handler = IcoCarveHandler::new("ico".to_string(), 0, 0);
        let hit = NormalizedHit {
            global_offset: 0,
            file_type_id: "ico".to_string(),
            pattern_id: "ico_header".to_string(),
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
