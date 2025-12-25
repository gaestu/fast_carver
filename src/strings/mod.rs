use crate::chunk::ScanChunk;

#[derive(Debug, Clone)]
pub struct StringSpan {
    pub chunk_id: u64,
    pub local_start: u64,
    pub length: u32,
    pub flags: u32,
}

pub trait StringScanner: Send + Sync {
    fn scan_chunk(&self, chunk: &ScanChunk, data: &[u8]) -> Vec<StringSpan>;
}

use crate::config::Config;
use anyhow::Result;

pub fn build_string_scanner(_cfg: &Config) -> Result<Box<dyn StringScanner>> {
    Err(anyhow::anyhow!("string scanning not implemented in phase 1"))
}

pub mod artifacts {
    use serde::Serialize;

    #[derive(Debug, Clone, Serialize)]
    pub enum ArtefactKind {
        Url,
        Email,
        Phone,
        GenericString,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct StringArtefact {
        pub run_id: String,
        pub artefact_kind: ArtefactKind,
        pub content: String,
        pub encoding: String,
        pub global_start: u64,
        pub global_end: u64,
    }
}
