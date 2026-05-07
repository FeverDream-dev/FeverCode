pub mod chunker;
pub mod embedder;
pub mod ingest;
pub mod mastermind;
pub mod store;

use anyhow::Result;
use std::path::Path;

/// Index a directory recursively into the RAG store.
pub async fn index_directory(
    store: &mut store::VectorStore,
    embedder: &dyn embedder::Embedder,
    root: &Path,
) -> Result<usize> {
    let mut total = 0usize;
    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let text = match ext.as_str() {
            "txt" | "md" | "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | "c"
            | "cpp" | "h" | "hpp" | "json" | "toml" | "yaml" | "yml" | "css" | "html"
            | "htm" | "sql" | "sh" | "bash" | "zsh" | "fish" | "dockerfile" => {
                std::fs::read_to_string(path).ok()
            }
            "pdf" => ingest::extract_pdf_text(path).ok(),
            _ => None,
        };
        let Some(text) = text else { continue };
        let chunks = chunker::chunk_text(&text, 512, 64);
        for (i, chunk) in chunks.iter().enumerate() {
            let embedding = embedder.embed(chunk).await?;
            store.add(store::DocumentChunk {
                id: format!("{}#{}", path.display(), i),
                source: path.display().to_string(),
                text: chunk.clone(),
                embedding,
            });
            total += 1;
        }
    }
    Ok(total)
}
