use std::fs::File;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvidenceError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unsupported evidence type: {0}")]
    Unsupported(String),
}

pub trait EvidenceSource: Send + Sync {
    fn len(&self) -> u64;
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, EvidenceError>;
}

pub struct RawFileSource {
    file: File,
    len: u64,
    #[cfg(not(unix))]
    lock: std::sync::Mutex<()>,
}

impl RawFileSource {
    pub fn open(path: &std::path::Path) -> Result<Self, EvidenceError> {
        let file = File::open(path)?;
        let len = file.metadata()?.len();
        Ok(Self {
            file,
            len,
            #[cfg(not(unix))]
            lock: std::sync::Mutex::new(()),
        })
    }
}

impl EvidenceSource for RawFileSource {
    fn len(&self) -> u64 {
        self.len
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, EvidenceError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileExt;
            Ok(self.file.read_at(buf, offset)?)
        }
        #[cfg(not(unix))]
        {
            use std::io::{Read, Seek, SeekFrom};
            let _guard = self.lock.lock().unwrap();
            let mut f = &self.file;
            f.seek(SeekFrom::Start(offset))?;
            Ok(f.read(buf)?)
        }
    }
}

use crate::cli::CliOptions;

pub fn open_source(opts: &CliOptions) -> Result<Box<dyn EvidenceSource>, EvidenceError> {
    let src = RawFileSource::open(&opts.input)?;
    Ok(Box::new(src))
}
