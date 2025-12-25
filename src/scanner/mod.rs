pub mod cpu;

use crate::chunk::ScanChunk;

#[derive(Debug, Clone)]
pub struct Hit {
    pub chunk_id: u64,
    pub local_offset: u64,
    pub pattern_id: String,
    pub file_type_id: String,
}

#[derive(Debug, Clone)]
pub struct NormalizedHit {
    pub global_offset: u64,
    pub file_type_id: String,
    pub pattern_id: String,
}

pub trait SignatureScanner: Send + Sync {
    fn scan_chunk(&self, chunk: &ScanChunk, data: &[u8]) -> Vec<Hit>;
}

use crate::config::Config;
use anyhow::Result;

pub fn build_signature_scanner(cfg: &Config) -> Result<Box<dyn SignatureScanner>> {
    Ok(Box::new(cpu::CpuScanner::new(cfg)?))
}
