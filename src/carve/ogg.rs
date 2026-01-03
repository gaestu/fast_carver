//! OGG container carving handler.
//!
//! Ogg streams consist of pages with a fixed header and lacing table.
//! We walk pages until an end-of-stream flag is observed.

use std::fs::File;

use crate::carve::{
    CarveError, CarveHandler, CarveStream, CarvedFile, ExtractionContext, output_path,
};
use crate::scanner::NormalizedHit;

pub struct OggCarveHandler {
    extension: String,
    min_size: u64,
    max_size: u64,
}

impl OggCarveHandler {
    pub fn new(extension: String, min_size: u64, max_size: u64) -> Self {
        Self {
            extension,
            min_size,
            max_size,
        }
    }
}

impl CarveHandler for OggCarveHandler {
    fn file_type(&self) -> &str {
        "ogg"
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
        let mut stream = CarveStream::new(ctx.evidence, hit.global_offset, self.max_size, file);

        let mut validated = false;
        let mut truncated = false;
        let mut errors = Vec::new();

        let result: Result<u64, CarveError> = (|| {
            let mut pages = 0u64;
            loop {
                let header = stream.read_exact(27)?;
                if &header[0..4] != b"OggS" {
                    return Err(CarveError::Invalid(
                        "ogg page signature mismatch".to_string(),
                    ));
                }
                if header[4] != 0 {
                    return Err(CarveError::Invalid("ogg version unsupported".to_string()));
                }
                let header_type = header[5];
                let segment_count = header[26] as usize;
                let segment_table = stream.read_exact(segment_count)?;
                let mut data_len = 0u64;
                for len in &segment_table {
                    data_len = data_len.saturating_add(*len as u64);
                }
                if data_len > 0 {
                    stream.read_exact(data_len as usize)?;
                }

                pages += 1;
                if header_type & 0x04 != 0 {
                    validated = true;
                    break;
                }
                if pages > 1_000_000 {
                    return Err(CarveError::Invalid("ogg page limit exceeded".to_string()));
                }
            }

            Ok(stream.bytes_written())
        })();

        if let Err(err) = result {
            match err {
                CarveError::Truncated | CarveError::Eof => {
                    truncated = true;
                    errors.push(err.to_string());
                }
                CarveError::Invalid(_) => {
                    let _ = std::fs::remove_file(&full_path);
                    return Ok(None);
                }
                other => return Err(other),
            }
        }

        let (size, md5_hex, sha256_hex) = stream.finish()?;

        if size < self.min_size {
            let _ = std::fs::remove_file(&full_path);
            return Ok(None);
        }

        if self.max_size > 0 && size >= self.max_size {
            truncated = true;
            if !errors.iter().any(|e| e.contains("max_size")) {
                errors.push("max_size reached".to_string());
            }
        }

        let global_end = if size == 0 {
            hit.global_offset
        } else {
            hit.global_offset + size - 1
        };

        Ok(Some(CarvedFile {
            run_id: ctx.run_id.to_string(),
            file_type: self.file_type().to_string(),
            path: rel_path,
            extension: self.extension.clone(),
            global_start: hit.global_offset,
            global_end,
            size,
            md5: Some(md5_hex),
            sha256: Some(sha256_hex),
            validated,
            truncated,
            errors,
            pattern_id: Some(hit.pattern_id.clone()),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::OggCarveHandler;
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

    fn minimal_ogg_page() -> Vec<u8> {
        let mut page = Vec::new();
        page.extend_from_slice(b"OggS");
        page.push(0); // version
        page.push(0x04); // end of stream
        page.extend_from_slice(&[0u8; 8]); // granule position
        page.extend_from_slice(&[0u8; 4]); // serial
        page.extend_from_slice(&[0u8; 4]); // seq
        page.extend_from_slice(&[0u8; 4]); // crc
        page.push(0); // segment count
        page
    }

    #[test]
    fn carves_minimal_ogg() {
        let data = minimal_ogg_page();
        let evidence = SliceEvidence { data: data.clone() };
        let handler = OggCarveHandler::new("ogg".to_string(), 0, 0);
        let hit = NormalizedHit {
            global_offset: 0,
            file_type_id: "ogg".to_string(),
            pattern_id: "ogg_sync".to_string(),
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
