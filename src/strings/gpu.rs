use tracing::warn;

use crate::chunk::ScanChunk;
use crate::strings::cpu::CpuStringScanner;
use crate::strings::{StringScanner, StringSpan};

pub struct GpuStringScanner {
    fallback: CpuStringScanner,
}

impl GpuStringScanner {
    pub fn new(min_len: usize, max_len: usize) -> Self {
        warn!("gpu string scanner initialized without GPU backend; using CPU fallback");
        Self {
            fallback: CpuStringScanner::new(min_len, max_len),
        }
    }
}

impl StringScanner for GpuStringScanner {
    fn scan_chunk(&self, chunk: &ScanChunk, data: &[u8]) -> Vec<StringSpan> {
        self.fallback.scan_chunk(chunk, data)
    }
}
