# Provider Expansion Implementation Plan

**Date**: 2026-04-05
**Based on**: REALITY_AUDIT.md v2

---

## Phase 0: Truth Audit ✅ DONE

Complete audit written to `REALITY_AUDIT.md`. Key findings:
- 5 adapters, not 12. OpenAiAdapter has 9 factory methods (profiles).
- Z.ai BROKEN (wrong base URL), Ollama streaming BROKEN, Anthropic tools NOT WIRED
- Browser/Search/Telegram not wired into agent runtime
- 190 tests, 0 in fever-browser, 0 in fever-search

---

## Phase 1: Provider Platform Refactor

### 1A. Fix Broken Providers (P0)

**Z.ai** (`crates/fever-cli/src/main.rs:226-233`):
- Change env var from `FEVER_ZAI_KEY` to `ZAI_API_KEY`
- Change from `OpenAiAdapter::openrouter(key)` to `OpenAiAdapter::custom("zai", key, "https://api.z.ai/api/paas/v4")`
- Add `ZAI_API_KEY` to env var table in README

**Ollama streaming** (`crates/fever-providers/src/adapters/ollama.rs:301-308`):
- Implement NDJSON streaming using native `/api/chat` endpoint (not `/v1/chat/completions`)
- Ollama native format: `{"message":{"role":"assistant","content":"..."}` repeated
- Add health check: `GET /` → "Ollama is running"
- Add real model listing via `GET /api/tags`

**Anthropic tools** (`crates/fever-providers/src/adapters/anthropic.rs`):
- Add `tools: Option<Vec<AnthropicTool>>` to `AnthropicRequestBody`
- Add `AnthropicTool` struct matching Anthropic API format
- Parse `tool_use` content blocks from response
- Map to our `ToolCall` format

### 1B. ProviderProfile + ProviderRegistry

**New file**: `crates/fever-providers/src/profile.rs`
```rust
pub struct ProviderProfile {
    pub id: String,              // e.g. "openai", "zai", "ollama"
    pub display_name: String,    // e.g. "OpenAI", "Z.ai (GLM)"
    pub adapter_type: AdapterType, // OpenAi, Anthropic, Gemini, Ollama, Native
    pub base_url: String,
    pub env_var: String,         // e.g. "OPENAI_API_KEY"
    pub default_model: String,   // e.g. "gpt-4o"
    pub models: Vec<String>,     // known models (can be fetched)
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub requires_auth: bool,
    pub tier: ProviderTier,      // FirstClass, Compatible, Community
}

pub enum AdapterType { OpenAi, Anthropic, Gemini, Ollama }
pub enum ProviderTier { FirstClass, Compatible, Community }
```

**New file**: `crates/fever-providers/src/registry.rs`
```rust
pub struct ProviderRegistry {
    profiles: HashMap<String, ProviderProfile>,
}

impl ProviderRegistry {
    pub fn builtin() -> Self;  // loads all known profiles
    pub fn register(&mut self, profile: ProviderProfile);
    pub fn get(&self, id: &str) -> Option<&ProviderProfile>;
    pub fn list(&self) -> Vec<&ProviderProfile>;
    pub fn list_configured(&self) -> Vec<&ProviderProfile>;  // env var present
    pub fn create_adapter(&self, id: &str) -> Result<Arc<dyn ProviderAdapter>>;
}
```

### 1C. Fix TUI Provider Switching

Currently `app.provider_name`/`app.model_name` are display-only strings that don't affect the actual ProviderClient. Need to:
- Store the ProviderClient in AppState (or make it accessible)
- When `/provider` or `/model` changes, update the actual routing
- This is blocked by the current architecture where ProviderClient is moved into FeverAgentHandle

---

## Phase 2: Provider Catalog Expansion

### Target: 50+ profiles using ProviderProfile

All OpenAI-compatible providers use `OpenAiAdapter::custom()` with different base_url/env_var.

**Batch 1 — First-Class (10)**:
OpenAI, Anthropic, Gemini, Ollama, Z.ai, Groq, DeepSeek, Mistral, Together, OpenRouter

**Batch 2 — Compatible (20+)**:
Fireworks, Perplexity, MiniMax, Cohere, AI21, Anthropic (AWS Bedrock), Azure OpenAI, Hugging Face, Replicate, Anyscale, Cerebras, SambaNova, AI21 Labs, Writer, Voyage AI, Jina AI, Together (embedded), Cloudflare Workers AI, Lepton AI, Segmind, Monster API

**Batch 3 — Community/Emerging (20+)**:
Novita AI, DeepInfra, OctoAI, Gradient AI, Lambda Labs, Modal, Bananadev, Petals, Ollama (cloud), LM Studio (local), KoboldAI, TabbyAPI, vLLM, TextGen WebUI, LocalAI, Ollama (OpenAI-compat), Mistral (Codestral), CodeLlama, StarCoder, Phi, Gemma

### Z.ai specific:
- Base URL: `https://api.z.ai/api/paas/v4`
- Env var: `ZAI_API_KEY`
- Models: glm-5-turbo, glm-5, glm-4.7, glm-4.7-flash, glm-4.6, glm-4.5
- Capabilities: chat ✅, streaming ✅, tools ✅, vision ❌
- No model listing endpoint → hardcoded

### Ollama specific:
- Base URL: `http://localhost:11434` (configurable via `OLLAMA_BASE_URL`)
- No auth (local only)
- Health check: `GET /`
- Model listing: `GET /api/tags`
- Native streaming: NDJSON via `POST /api/chat`
- Capabilities: chat ✅, streaming ✅, tools ✅ (since May 2025), vision ⚠️

---

## Phase 3: Harden Streaming + Tools

- Unify SSE parsing (OpenAI, Anthropic share pattern)
- Implement NDJSON parsing (Ollama native)
- Ensure tool results re-enter loop correctly in all adapters
- Add streaming tool call support (delta chunks for tool calls)
- Add fallback when provider lacks tools/streaming

---

## Phase 4: Feature Gap Closure

- Wire `discover_instructions()` into `prepare_request()`
- Wire `load_with_workspace()` into CLI startup
- Register SearchClient as a tool (or mark as TODO)
- Mark BrowserTool as placeholder in docs
- Mark MemoryStore, EventBus, ExecutionEngine, Telemetry as stubs
- Fix TUI provider switching to actually change routing

---

## Phase 5: Tests + Docs

- Provider conformance test: send ChatRequest, verify ChatResponse shape
- Streaming conformance: verify StreamChunk sequence
- Registration tests: verify env vars → correct adapter
- Update README: "5 adapters, 50+ profiles"
- Fix architecture diagram
- Update env var table
