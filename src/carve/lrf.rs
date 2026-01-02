//! LRF (Sony BroadBand eBook) carving handler.
//!
//! Uses a heuristic size field; falls back to max_size when unknown.

use std::fs::File;

use sha2::{Digest, Sha256};

use crate::carve::{
    output_path, write_range, CarveError, CarveHandler, CarvedFile, ExtractionContext,
};
use crate::scanner::NormalizedHit;

const LRF_MAGIC: [u8; 4] = [0x4C, 0x52, 0x46, 0x00];

pub struct LrfCarveHandler {
    extension: String,
    min_size: u64,
    max_size: u64,
}

impl LrfCarveHandler {
    pub fn new(extension: String, min_size: u64, max_size: u64) -> Self {
        Self {
            extension,
            min_size,
            max_size,
        }
    }
}

impl CarveHandler for LrfCarveHandler {
    fn file_type(&self) -> &str {
        "lrf"
    }

    fn extension(&self) -> &str {
        &self.extension
    }

    fn process_hit(
        &self,
        hit: &NormalizedHit,
        ctx: &ExtractionContext,
    ) -> Result<Option<CarvedFile>, CarveError> {
        let header = read_exact_at(ctx, hit.global_offset, 12)
            .ok_or_else(|| CarveError::Invalid("lrf header too short".to_string()))?;
        if header[0..4] != LRF_MAGIC {
            return Ok(None);
        }
        let declared = u32::from_le_bytes([header[8], header[9], header[10], header[11]]) as u64;
        let mut size = if declared > 0 { declared } else { 0 };
        if self.max_size > 0 && (size == 0 || size > self.max_size) {
            size = self.max_size;
        }
        if size == 0 {
            size = 1024;
        }

        let mut total_end = hit.global_offset.saturating_add(size);
        if self.max_size > 0 {
            let max_end = hit.global_offset.saturating_add(self.max_size);
            if total_end > max_end {
                total_end = max_end;
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
    use super::LrfCarveHandler;
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
    fn carves_minimal_lrf() {
        let mut data = vec![0u8; 64];
        data[0..4].copy_from_slice(&[0x4C, 0x52, 0x46, 0x00]);
        data[8..12].copy_from_slice(&(64u32).to_le_bytes());
        let evidence = SliceEvidence { data: data.clone() };
        let handler = LrfCarveHandler::new("lrf".to_string(), 0, 0);
        let hit = NormalizedHit {
            global_offset: 0,
            file_type_id: "lrf".to_string(),
            pattern_id: "lrf_header".to_string(),
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
