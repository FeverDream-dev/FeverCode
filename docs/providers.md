# FeverCode Providers — Setup Guide

Current status: Working. Five provider configurations are supported: Z.ai, OpenAI, Ollama Local, Ollama Cloud, and Gemini CLI. The system handles env vars, base URLs, and model names.

Provider overview
- The FeverCode client can route prompts to different providers via a unified config.
- Each provider exposes its own environment variables, base URL, and model name.
- A custom OpenAI-compatible endpoint can be added by configuring a provider with custom base_url and model.

Table: providers, env vars, base URLs, and models

| Provider | Environment Variables | Base URL (default) | Model name(s) | Notes |
| --- | --- | --- | --- | --- |
| Z.ai | ZAI_API_BASE, ZAI_API_KEY, ZAI_DEFAULT_MODEL | https://api.zaicore.example/v1 | zai-llm-v1 | Local provider with optional default model; configurable base URL |
| OpenAI | OPENAI_API_KEY, OPENAI_ORGANIZATION (optional) | https://api.openai.com/v1 | gpt-4o, code-davinci-002 (example) | Requires key; supports multiple models
| Ollama Local | OLLAMA_HOST, OLLAMA_MODEL | http://localhost:11434 | ollama-llm-1 | Local containerized LLM gateway
| Ollama Cloud | OLLAMA_CLOUD_API_KEY, OLLAMA_CLOUD_BASE_URL | https://api.ollama.cloud/v1 | ollama-cloud-llm | Cloud-hosted Ollama API
| Gemini CLI | GEMINI_CLI_API_TOKEN, GEMINI_CLI_BASE_URL | https:// Gemini CLI base | gemini-llm-v1 | Gemini via CLI transport

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
