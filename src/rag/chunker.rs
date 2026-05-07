/// Naive character-level sliding window chunker.
/// Splits on paragraph boundaries when possible, falls back to fixed-size windows.
pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let text = text.trim();
    if text.len() <= chunk_size {
        return vec![text.to_string()];
    }

    // Try paragraph splitting first
    let paragraphs: Vec<&str> = text
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .collect();
    let mut current = String::new();
    for para in paragraphs {
        let trimmed = para.trim();
        if current.len() + trimmed.len() + 2 > chunk_size {
            if !current.is_empty() {
                chunks.push(current.trim().to_string());
            }
            current = trimmed.to_string();
        } else {
            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(trimmed);
        }
    }
    if !current.is_empty() {
        chunks.push(current.trim().to_string());
    }

    // If any chunk is still too big, split by sentences / fixed windows
    let mut final_chunks = Vec::new();
    for chunk in chunks {
        if chunk.len() <= chunk_size {
            final_chunks.push(chunk);
        } else {
            let mut start = 0usize;
            while start < chunk.len() {
                let end = (start + chunk_size).min(chunk.len());
                let slice = &chunk[start..end];
                final_chunks.push(slice.trim().to_string());
                start += chunk_size - overlap;
            }
        }
    }

    final_chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_small_text() {
        let text = "hello world";
        let chunks = chunk_text(text, 100, 10);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "hello world");
    }

    #[test]
    fn chunks_large_text() {
        let text = "a".repeat(1000);
        let chunks = chunk_text(&text, 300, 50);
        assert!(chunks.len() >= 3);
        for c in &chunks {
            assert!(c.len() <= 300);
        }
    }
}
