#[derive(Debug, Clone)]
pub struct ScanChunk {
    pub id: u64,
    pub start: u64,
    pub length: u64,
    pub valid_length: u64,
}

pub fn build_chunks(total_len: u64, chunk_size: u64, overlap: u64) -> Vec<ScanChunk> {
    if chunk_size == 0 {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut start = 0u64;
    let mut id = 0u64;

    while start < total_len {
        let remaining = total_len - start;
        let length = remaining.min(chunk_size.saturating_add(overlap));
        let valid_length = remaining.min(chunk_size);

        chunks.push(ScanChunk {
            id,
            start,
            length,
            valid_length,
        });

        start = start.saturating_add(chunk_size);
        id += 1;
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_chunks_with_overlap() {
        let chunks = build_chunks(100, 40, 10);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].start, 0);
        assert_eq!(chunks[0].length, 50);
        assert_eq!(chunks[0].valid_length, 40);
        assert_eq!(chunks[1].start, 40);
        assert_eq!(chunks[1].length, 50);
        assert_eq!(chunks[1].valid_length, 40);
        assert_eq!(chunks[2].start, 80);
        assert_eq!(chunks[2].length, 20);
        assert_eq!(chunks[2].valid_length, 20);
    }
}
