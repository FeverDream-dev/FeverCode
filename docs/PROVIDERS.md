# Provider System

FeverCode should support providers through a small internal trait and config-driven registry.

## Provider kinds

### `openai_compatible`

Use for OpenAI, Z.ai, Ollama local, Ollama Cloud, OpenRouter, Groq, DeepSeek, Mistral, xAI, and any endpoint that exposes an OpenAI-style chat/completions or responses API.

Required config:

```toml
[[providers.available]]
name = "my-provider"
kind = "openai_compatible"
base_url = "https://example.com/v1"
api_key_env = "MY_PROVIDER_API_KEY"
models = ["my-coding-model"]
```

### `external_cli`

Use for Gemini CLI or other tools that expose a local command.

```toml
[[providers.available]]
name = "gemini-cli"
kind = "external_cli"
command = "gemini"
models = ["gemini-default"]
```

## Day-one providers requested

- Z.ai / GLM Coding Plan.
- ChatGPT / OpenAI / Codex-compatible OpenAI endpoints.
- Gemini CLI as an external CLI bridge first, native Gemini API later.
- Ollama local through `http://localhost:11434/v1`.
- Ollama Cloud through an OpenAI-compatible cloud endpoint when configured.

## Add a new provider

1. Add provider config to `.fevercode/config.toml`.
2. Export the API key in your shell.
3. Run `fever providers`.
4. Run `fever doctor`.
5. Use `/provider my-provider/my-model` in the TUI once implemented.

## Implementation plan

- Create `ProviderClient` with streaming support.
- Normalize message input and tool calls.
- Normalize usage metadata and token budget.
- Add retries with exponential backoff.
- Add provider health checks in `fever doctor`.
- Never log raw API keys.
