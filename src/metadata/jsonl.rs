use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Mutex;

use serde::Serialize;

use crate::carve::CarvedFile;
use crate::metadata::{MetadataError, MetadataSink};
use crate::strings::artifacts::StringArtefact;

pub struct JsonlSink {
    tool_version: String,
    config_hash: String,
    evidence_path: String,
    evidence_sha256: String,
    writer: Mutex<BufWriter<File>>,
}

#[derive(Serialize)]
struct CarvedFileRecord<'a> {
    #[serde(flatten)]
    file: &'a CarvedFile,
    tool_version: &'a str,
    config_hash: &'a str,
    evidence_path: &'a str,
    evidence_sha256: &'a str,
}

impl JsonlSink {
    pub fn new(
        _run_id: &str,
        tool_version: &str,
        config_hash: &str,
        evidence_path: &Path,
        evidence_sha256: &str,
        run_output_dir: &Path,
    ) -> Result<Self, MetadataError> {
        let meta_dir = run_output_dir.join("metadata");
        std::fs::create_dir_all(&meta_dir)?;
        let path = meta_dir.join("carved_files.jsonl");
        let file = File::create(path)?;
        Ok(Self {
            tool_version: tool_version.to_string(),
            config_hash: config_hash.to_string(),
            evidence_path: evidence_path.to_string_lossy().to_string(),
            evidence_sha256: evidence_sha256.to_string(),
            writer: Mutex::new(BufWriter::new(file)),
        })
    }
}

impl MetadataSink for JsonlSink {
    fn record_file(&self, file: &CarvedFile) -> Result<(), MetadataError> {
        let record = CarvedFileRecord {
            file,
            tool_version: &self.tool_version,
            config_hash: &self.config_hash,
            evidence_path: &self.evidence_path,
            evidence_sha256: &self.evidence_sha256,
        };
        let mut guard = self.writer.lock().unwrap();
        serde_json::to_writer(&mut *guard, &record)?;
        guard.write_all(b"\n")?;
        Ok(())
    }

    fn record_string(&self, _artefact: &StringArtefact) -> Result<(), MetadataError> {
        Ok(())
    }

    fn record_history(&self, _record: &crate::parsers::browser::BrowserHistoryRecord) -> Result<(), MetadataError> {
        Ok(())
    }

    fn flush(&self) -> Result<(), MetadataError> {
        let mut guard = self.writer.lock().unwrap();
        guard.flush()?;
        Ok(())
    }
}
