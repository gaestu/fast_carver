pub mod jsonl;

use std::path::Path;

use thiserror::Error;

use crate::carve::CarvedFile;
use crate::strings::artifacts::StringArtefact;

#[derive(Debug, Clone, Copy)]
pub enum MetadataBackendKind {
    Jsonl,
}

#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("other error: {0}")]
    Other(String),
}

pub trait MetadataSink: Send + Sync {
    fn record_file(&self, file: &CarvedFile) -> Result<(), MetadataError>;
    fn record_string(&self, artefact: &StringArtefact) -> Result<(), MetadataError>;
    fn record_history(&self, _record: &crate::parsers::browser::BrowserHistoryRecord) -> Result<(), MetadataError>;
    fn flush(&self) -> Result<(), MetadataError>;
}

pub fn build_sink(
    backend: MetadataBackendKind,
    run_id: &str,
    tool_version: &str,
    config_hash: &str,
    evidence_path: &Path,
    evidence_sha256: &str,
    run_output_dir: &Path,
) -> Result<Box<dyn MetadataSink>, MetadataError> {
    match backend {
        MetadataBackendKind::Jsonl => Ok(Box::new(jsonl::JsonlSink::new(
            run_id,
            tool_version,
            config_hash,
            evidence_path,
            evidence_sha256,
            run_output_dir,
        )?)),
    }
}
