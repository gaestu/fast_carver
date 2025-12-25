use crate::chunk::ScanChunk;
use crate::strings::{StringSpan, StringScanner};

pub struct CpuStringScanner {
    min_len: usize,
    max_len: usize,
}

impl CpuStringScanner {
    pub fn new(min_len: usize, max_len: usize) -> Self {
        let max_len = if max_len == 0 { usize::MAX } else { max_len };
        Self { min_len, max_len }
    }
}

impl StringScanner for CpuStringScanner {
    fn scan_chunk(&self, chunk: &ScanChunk, data: &[u8]) -> Vec<StringSpan> {
        let mut spans = Vec::new();
        let mut i = 0usize;

        while i < data.len() {
            if !is_printable(data[i]) {
                i += 1;
                continue;
            }

            let start = i;
            let mut len = 0usize;
            while i < data.len() && is_printable(data[i]) {
                i += 1;
                len += 1;
                if len >= self.max_len {
                    break;
                }
            }

            if len >= self.min_len {
                spans.push(StringSpan {
                    chunk_id: chunk.id,
                    local_start: start as u64,
                    length: len as u32,
                    flags: 0,
                });
            }

            if len >= self.max_len {
                continue;
            }
        }

        spans
    }
}

fn is_printable(byte: u8) -> bool {
    matches!(byte, b'\t' | 0x20..=0x7E)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_printable_runs() {
        let scanner = CpuStringScanner::new(4, 1024);
        let chunk = ScanChunk {
            id: 1,
            start: 0,
            length: 12,
            valid_length: 12,
        };
        let data = b"abc\0defg\nxyz";
        let spans = scanner.scan_chunk(&chunk, data);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].local_start, 4);
        assert_eq!(spans[0].length, 4);
    }

    #[test]
    fn splits_long_strings() {
        let scanner = CpuStringScanner::new(2, 4);
        let chunk = ScanChunk {
            id: 1,
            start: 0,
            length: 8,
            valid_length: 8,
        };
        let data = b"abcdefgh";
        let spans = scanner.scan_chunk(&chunk, data);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].length, 4);
        assert_eq!(spans[1].length, 4);
        assert_eq!(spans[1].local_start, 4);
    }
}
