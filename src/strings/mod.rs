pub mod cpu;

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

pub fn build_string_scanner(cfg: &Config) -> Result<Box<dyn StringScanner>> {
    Ok(Box::new(cpu::CpuStringScanner::new(
        cfg.string_min_len,
        cfg.string_max_len,
    )))
}

pub mod artifacts {
    use once_cell::sync::Lazy;
    use regex::Regex;
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

    static URL_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?i)\b(?:https?://|www\.)[^\s"'<>]+"#).expect("url regex")
    });
    static EMAIL_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b").expect("email regex")
    });
    static PHONE_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"\b\+?\d[\d\s().-]{6,}\d\b").expect("phone regex")
    });

    pub fn extract_artefacts(
        run_id: &str,
        chunk_start: u64,
        local_start: u64,
        data: &[u8],
    ) -> Vec<StringArtefact> {
        let mut out = Vec::new();
        let text = String::from_utf8_lossy(data);

        for mat in URL_RE.find_iter(&text) {
            out.push(build_artefact(
                run_id,
                ArtefactKind::Url,
                mat.as_str(),
                chunk_start + local_start + mat.start() as u64,
            ));
        }

        for mat in EMAIL_RE.find_iter(&text) {
            out.push(build_artefact(
                run_id,
                ArtefactKind::Email,
                mat.as_str(),
                chunk_start + local_start + mat.start() as u64,
            ));
        }

        for mat in PHONE_RE.find_iter(&text) {
            out.push(build_artefact(
                run_id,
                ArtefactKind::Phone,
                mat.as_str(),
                chunk_start + local_start + mat.start() as u64,
            ));
        }

        out
    }

    fn build_artefact(run_id: &str, kind: ArtefactKind, content: &str, global_start: u64) -> StringArtefact {
        let len = content.as_bytes().len() as u64;
        let global_end = if len == 0 { global_start } else { global_start + len - 1 };
        StringArtefact {
            run_id: run_id.to_string(),
            artefact_kind: kind,
            content: content.to_string(),
            encoding: "ascii".to_string(),
            global_start,
            global_end,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{extract_artefacts, ArtefactKind};

        #[test]
        fn extracts_basic_artefacts() {
            let data = b"visit https://example.com and mail test@example.com";
            let out = extract_artefacts("run1", 100, 0, data);
            assert!(out.iter().any(|a| matches!(a.artefact_kind, ArtefactKind::Url)));
            assert!(out.iter().any(|a| matches!(a.artefact_kind, ArtefactKind::Email)));
        }
    }
}
