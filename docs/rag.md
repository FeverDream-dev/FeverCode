# Local Mastermind RAG

The Local Mastermind is FeverCode's retrieval-augmented generation system. It lets small local models (Phi4 1.5B, Qwen2.5 3B, etc.) match top cloud LLMs by reasoning iteratively over your workspace documents.

## Why

Small models lack broad world knowledge. But they reason well. If you give them the right documents — chunked, embedded, and searched — they can answer complex questions about your codebase, PDFs, and notes through multiple retrieval and reasoning steps.

## How it works

1. **Ingest** — `/index` walks your workspace, reads `.txt`, `.md`, `.rs`, `.py`, `.pdf`, etc., splits into chunks (512 chars, 64 overlap), and embeds each chunk with `nomic-embed-text` via Ollama.
2. **Store** — Embeddings saved to `.fevercode/rag_store.json` with cosine similarity search.
3. **Query** — `/mastermind "your question"` embeds the query, finds top-5 similar chunks, feeds them to the LLM.
4. **Iterate** — The LLM decides: `SEARCH: <follow-up>` / `PARTIAL: <reasoning>` / `ANSWER: <final>`. Up to 6 iterations.
5. **Synthesize** — Final answer with source citations.

## Requirements

- Ollama running locally (default: `http://localhost:11434`)
- `nomic-embed-text` model pulled: `ollama pull nomic-embed-text`
- Any small LLM for reasoning: `ollama pull phi4` or `ollama pull qwen2.5`
- `pdftotext` (poppler-utils) for PDF ingestion

## TUI Commands

| Command | Description |
|---|---|
| `/index` | Index all workspace documents into the RAG store |
| `/mastermind "question"` | Run iterative RAG query |
| `/rag-status` | Show chunk count and indexed sources |
| `/rag-clear` | Empty the store |

## Example

```
/index
> Indexed 1,247 chunks.

/mastermind how does authentication work in this project?
> Local Mastermind finished in 4 iterations (3 queries)
> Sources: src/auth.rs, docs/auth.md
> Answer: The project uses JWT tokens stored in HttpOnly cookies...
```

## Architecture

- `src/rag/chunker.rs` — paragraph-aware sliding window chunker
- `src/rag/embedder.rs` — Ollama and OpenAI-compatible embedding clients
- `src/rag/store.rs` — in-memory vector store with cosine similarity, JSON persistence
- `src/rag/ingest.rs` — PDF and text extraction
- `src/rag/mastermind.rs` — multi-step retrieve-generate agent loop
- `src/rag/mod.rs` — `index_directory()` helper

## Limitations

- Embedding requires a running Ollama or compatible endpoint
- Large workspaces may take time to index (one embedding call per chunk)
- Store is in-memory + JSON; no vector DB like Qdrant or Pinecone
- PDF extraction requires `pdftotext` CLI tool
