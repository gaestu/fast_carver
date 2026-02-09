use std::fs::File;
use std::io::Write;

use sha2::{Digest, Sha256};

use crate::carve::{
    CarveError, CarveHandler, CarvedFile, ExtractionContext, output_path, write_range,
};
use crate::scanner::NormalizedHit;

const WAL_HEADER_LEN: u64 = 32;
const WAL_FRAME_HEADER_LEN: u64 = 24;
const WAL_MAGIC_1: u32 = 0x377F_0682;
const WAL_MAGIC_2: u32 = 0x377F_0683;

#[derive(Debug, Clone, Copy)]
struct WalHeader {
    page_size: u32,
    salt_1: u32,
    salt_2: u32,
}

#[derive(Debug)]
struct WalWalkResult {
    size: u64,
    frames: u32,
    truncated: bool,
    errors: Vec<String>,
}

pub struct SqliteWalCarveHandler {
    extension: String,
    min_size: u64,
    max_size: u64,
}

impl SqliteWalCarveHandler {
    pub fn new(extension: String, min_size: u64, max_size: u64) -> Self {
        Self {
            extension,
            min_size,
            max_size,
        }
    }
}

impl CarveHandler for SqliteWalCarveHandler {
    fn file_type(&self) -> &str {
        "sqlite_wal"
    }

    fn extension(&self) -> &str {
        &self.extension
    }

    fn process_hit(
        &self,
        hit: &NormalizedHit,
        ctx: &ExtractionContext,
    ) -> Result<Option<CarvedFile>, CarveError> {
        let header_bytes = match read_exact_at(ctx, hit.global_offset, WAL_HEADER_LEN as usize)? {
            Some(bytes) => bytes,
            None => return Ok(None),
        };
        let header = match parse_wal_header(&header_bytes) {
            Some(header) => header,
            None => return Ok(None),
        };

        let walked = walk_frames(ctx, hit.global_offset, header, self.max_size)?;
        if walked.frames == 0 {
            return Ok(None);
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

        let mut truncated = walked.truncated;
        let mut errors = walked.errors;
        let end = hit.global_offset.saturating_add(walked.size);
        let (written, eof_truncated) = write_range(
            ctx,
            hit.global_offset,
            end,
            &mut file,
            &mut md5,
            &mut sha256,
        )?;
        if eof_truncated {
            truncated = true;
            if !errors.iter().any(|e| e.contains("eof")) {
                errors.push("eof before WAL end".to_string());
            }
        }
        file.flush()?;

        if written < self.min_size {
            let _ = std::fs::remove_file(&full_path);
            return Ok(None);
        }

        let global_end = if written == 0 {
            hit.global_offset
        } else {
            hit.global_offset + written - 1
        };
        let validated = walked.frames > 0 && !truncated;
        let md5_hex = format!("{:x}", md5.compute());
        let sha256_hex = hex::encode(sha256.finalize());

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
            validated,
            truncated,
            errors,
            pattern_id: Some(hit.pattern_id.clone()),
        }))
    }
}

fn read_exact_at(
    ctx: &ExtractionContext,
    offset: u64,
    len: usize,
) -> Result<Option<Vec<u8>>, CarveError> {
    let mut buf = vec![0u8; len];
    let n = ctx
        .evidence
        .read_at(offset, &mut buf)
        .map_err(|e| CarveError::Evidence(e.to_string()))?;
    if n < len {
        return Ok(None);
    }
    Ok(Some(buf))
}

fn parse_wal_header(bytes: &[u8]) -> Option<WalHeader> {
    if bytes.len() < WAL_HEADER_LEN as usize {
        return None;
    }
    let magic = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    if magic != WAL_MAGIC_1 && magic != WAL_MAGIC_2 {
        return None;
    }

    // SQLite WAL uses the database page size here.
    let raw_page_size = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let page_size = if raw_page_size == 1 {
        65536
    } else {
        raw_page_size
    };
    if page_size < 512 || page_size > 65536 || !page_size.is_power_of_two() {
        return None;
    }

    Some(WalHeader {
        page_size,
        salt_1: u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
        salt_2: u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
    })
}

fn walk_frames(
    ctx: &ExtractionContext,
    start: u64,
    header: WalHeader,
    max_size: u64,
) -> Result<WalWalkResult, CarveError> {
    let evidence_len = ctx.evidence.len();
    let hard_end = if max_size > 0 {
        start.saturating_add(max_size).min(evidence_len)
    } else {
        evidence_len
    };

    let mut offset = start.saturating_add(WAL_HEADER_LEN);
    let mut frames = 0u32;
    let mut truncated = false;
    let mut errors = Vec::new();

    let frame_size = WAL_FRAME_HEADER_LEN.saturating_add(header.page_size as u64);
    while offset.saturating_add(WAL_FRAME_HEADER_LEN) <= hard_end {
        let frame_header = match read_exact_at(ctx, offset, WAL_FRAME_HEADER_LEN as usize)? {
            Some(bytes) => bytes,
            None => break,
        };

        let page_no = u32::from_be_bytes([
            frame_header[0],
            frame_header[1],
            frame_header[2],
            frame_header[3],
        ]);
        let frame_salt_1 = u32::from_be_bytes([
            frame_header[8],
            frame_header[9],
            frame_header[10],
            frame_header[11],
        ]);
        let frame_salt_2 = u32::from_be_bytes([
            frame_header[12],
            frame_header[13],
            frame_header[14],
            frame_header[15],
        ]);
        if page_no == 0 || frame_salt_1 != header.salt_1 || frame_salt_2 != header.salt_2 {
            break;
        }

        let frame_end = offset.saturating_add(frame_size);
        if frame_end > hard_end {
            truncated = true;
            if max_size > 0 && start.saturating_add(max_size) <= evidence_len {
                errors.push("max_size reached before WAL frame end".to_string());
            } else {
                errors.push("eof before WAL frame end".to_string());
            }
            break;
        }

        frames = frames.saturating_add(1);
        offset = frame_end;
    }

    Ok(WalWalkResult {
        size: offset.saturating_sub(start),
        frames,
        truncated,
        errors,
    })
}

#[cfg(test)]
mod tests {
    use super::{SqliteWalCarveHandler, parse_wal_header};
    use crate::carve::{CarveHandler, ExtractionContext};
    use crate::evidence::RawFileSource;
    use crate::scanner::NormalizedHit;

    fn build_wal_with_frames(frame_count: u32, truncate_last_frame_bytes: usize) -> Vec<u8> {
        let page_size = 4096u32;
        let salt_1: u32 = 0xAABB_CCDD;
        let salt_2: u32 = 0x1122_3344;
        let mut wal = vec![0u8; 32];
        wal[0..4].copy_from_slice(&0x377F_0682u32.to_be_bytes());
        wal[4..8].copy_from_slice(&3007000u32.to_be_bytes());
        wal[8..12].copy_from_slice(&page_size.to_be_bytes());
        wal[16..20].copy_from_slice(&salt_1.to_be_bytes());
        wal[20..24].copy_from_slice(&salt_2.to_be_bytes());

        for i in 0..frame_count {
            let mut frame = vec![0u8; 24 + page_size as usize];
            frame[0..4].copy_from_slice(&(i + 1).to_be_bytes());
            frame[8..12].copy_from_slice(&salt_1.to_be_bytes());
            frame[12..16].copy_from_slice(&salt_2.to_be_bytes());
            wal.extend_from_slice(&frame);
        }

        if truncate_last_frame_bytes > 0 && wal.len() > truncate_last_frame_bytes {
            wal.truncate(wal.len() - truncate_last_frame_bytes);
        }

        wal
    }

    #[test]
    fn parses_valid_magic_values() {
        let mut header = [0u8; 32];
        header[0..4].copy_from_slice(&0x377F_0682u32.to_be_bytes());
        header[8..12].copy_from_slice(&4096u32.to_be_bytes());
        assert!(parse_wal_header(&header).is_some());

        header[0..4].copy_from_slice(&0x377F_0683u32.to_be_bytes());
        assert!(parse_wal_header(&header).is_some());
    }

    #[test]
    fn rejects_invalid_magic_value() {
        let mut header = [0u8; 32];
        header[0..4].copy_from_slice(&0x1234_5678u32.to_be_bytes());
        header[8..12].copy_from_slice(&4096u32.to_be_bytes());
        assert!(parse_wal_header(&header).is_none());
    }

    #[test]
    fn stops_cleanly_on_truncated_frame() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let output_root = temp_dir.path().join("out");
        std::fs::create_dir_all(&output_root).expect("output dir");

        let wal = build_wal_with_frames(1, 200);
        let input_path = temp_dir.path().join("wal.bin");
        std::fs::write(&input_path, &wal).expect("write wal");

        let evidence = RawFileSource::open(&input_path).expect("evidence");
        let ctx = ExtractionContext {
            run_id: "test",
            output_root: &output_root,
            evidence: &evidence,
        };
        let handler = SqliteWalCarveHandler::new("sqlite-wal".to_string(), 32, 0);
        let hit = NormalizedHit {
            global_offset: 0,
            file_type_id: "sqlite_wal".to_string(),
            pattern_id: "sqlite_wal_magic_82".to_string(),
        };

        let carved = handler.process_hit(&hit, &ctx).expect("process");
        assert!(carved.is_none(), "truncated first frame should be rejected");
    }
}
