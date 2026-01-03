use anyhow::{Result, anyhow};
use memchr::memchr;

use crate::chunk::ScanChunk;
use crate::config::Config;
use crate::scanner::{Hit, SignatureScanner};

#[derive(Debug, Clone)]
struct Pattern {
    id: String,
    file_type_id: String,
    bytes: Vec<u8>,
}

pub struct CpuScanner {
    patterns: Vec<Pattern>,
}

impl CpuScanner {
    pub fn new(cfg: &Config) -> Result<Self> {
        let mut patterns = Vec::new();
        for file_type in &cfg.file_types {
            for pat in &file_type.header_patterns {
                let bytes = hex::decode(pat.hex.trim())
                    .map_err(|e| anyhow!("invalid hex pattern {}: {e}", pat.id))?;
                if bytes.is_empty() {
                    continue;
                }
                patterns.push(Pattern {
                    id: pat.id.clone(),
                    file_type_id: file_type.id.clone(),
                    bytes,
                });
            }
        }
        Ok(Self { patterns })
    }
}

impl SignatureScanner for CpuScanner {
    fn scan_chunk(&self, chunk: &ScanChunk, data: &[u8]) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pattern in &self.patterns {
            if pattern.bytes.is_empty() {
                continue;
            }
            let first = pattern.bytes[0];
            let mut pos = 0usize;
            while pos < data.len() {
                let found = memchr(first, &data[pos..]);
                let idx = match found {
                    Some(i) => pos + i,
                    None => break,
                };
                if idx + pattern.bytes.len() <= data.len()
                    && data[idx..idx + pattern.bytes.len()] == pattern.bytes[..]
                {
                    hits.push(Hit {
                        chunk_id: chunk.id,
                        local_offset: idx as u64,
                        pattern_id: pattern.id.clone(),
                        file_type_id: pattern.file_type_id.clone(),
                    });
                }
                pos = idx + 1;
            }
        }
        hits
    }
}
