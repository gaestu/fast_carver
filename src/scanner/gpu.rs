use anyhow::Result;
use tracing::warn;

use crate::chunk::ScanChunk;
use crate::config::Config;
use crate::scanner::cpu::CpuScanner;
use crate::scanner::{Hit, SignatureScanner};

pub struct GpuScanner {
    fallback: CpuScanner,
}

impl GpuScanner {
    pub fn new(cfg: &Config) -> Result<Self> {
        warn!("gpu scanner initialized without GPU backend; using CPU fallback");
        Ok(Self {
            fallback: CpuScanner::new(cfg)?,
        })
    }
}

impl SignatureScanner for GpuScanner {
    fn scan_chunk(&self, chunk: &ScanChunk, data: &[u8]) -> Vec<Hit> {
        self.fallback.scan_chunk(chunk, data)
    }
}
