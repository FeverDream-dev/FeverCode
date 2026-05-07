# FeverCode Providers — Setup Guide

Current status: Working. Five provider configurations are supported: Z.ai, OpenAI, Ollama Local, Ollama Cloud, and Gemini CLI. The system handles env vars, base URLs, model names, and **per-LLM presets**.

Provider overview
- The FeverCode client can route prompts to different providers via a unified config.
- Each provider exposes its own environment variables, base URL, and model name.
- A custom OpenAI-compatible endpoint can be added by configuring a provider with custom base_url and model.
- **Per-LLM presets** automatically detect the model and inject obedience rules, temperature, retry logic, and few-shot examples.

Table: providers, env vars, base URLs, and models

| Provider | Environment Variables | Base URL (default) | Model name(s) | Notes |
| --- | --- | --- | --- | --- |
| Z.ai | ZAI_API_BASE, ZAI_API_KEY, ZAI_DEFAULT_MODEL | https://api.zaicore.example/v1 | zai-llm-v1 | Local provider with optional default model; configurable base URL |
| OpenAI | OPENAI_API_KEY, OPENAI_ORGANIZATION (optional) | https://api.openai.com/v1 | gpt-4o, code-davinci-002 (example) | Requires key; supports multiple models |
| Ollama Local | OLLAMA_HOST, OLLAMA_MODEL | http://localhost:11434 | ollama-llm-1 | Local containerized LLM gateway |
| Ollama Cloud | OLLAMA_CLOUD_API_KEY, OLLAMA_CLOUD_BASE_URL | https://api.ollama.cloud/v1 | ollama-cloud-llm | Cloud-hosted Ollama API |
| Gemini CLI | GEMINI_CLI_API_TOKEN, GEMINI_CLI_BASE_URL | https:// Gemini CLI base | gemini-llm-v1 | Gemini via CLI transport |

## Per-LLM Presets

FeverCode automatically selects a preset based on the model name. You can override with `fever preset set NAME`, but **llama3.2 cannot be overridden** — it is hard-locked to `TestResearch`.

| Preset | Detected Models | Temperature | Behavior |
|--------|----------------|-------------|----------|
| CloudStrong | claude, gpt-4/5, gemini-2, glm-5 | 0.3 | Minimal constraints |
| LocalMedium | qwen2.5/3, llama3.1/3.3, mistral, mixtral, gemma2/3, deepseek, phi4 | 0.2 | CoT + few-shot |
| LocalSmall | gemma, phi3, qwen2 | 0.1 | Heavy few-shot, 3 retries |
| Precise | Any (manual) | 0.0 | Exact mode |
| VibeCoder | Any (manual for `fever vibe`) | 0.85 | Ship fast, assume intent |
| TestResearch | **llama3.2 ONLY** (hard-locked) | 0.15 | Test/research/internet only |

Custom OpenAI-compatible endpoint
- You can add a custom OpenAI-compatible endpoint by configuring a provider entry with base_url and model that match the endpoint's API.
- Example configuration (YAML):

```
providers:
  - name: custom-openai-compat
    type: openai_compat
    base_url: http://localhost:4000/v1
    api_key: ${OPENAI_COMPAT_API_KEY}
    model: gpt-4o
```

Configuration format
- YAML is supported for provider configuration blocks. A typical snippet may look like:

```
providers:
  - name: openai
    type: openai
    base_url: https://api.openai.com/v1
    api_key: ${OPENAI_API_KEY}
    model: gpt-4o
```
