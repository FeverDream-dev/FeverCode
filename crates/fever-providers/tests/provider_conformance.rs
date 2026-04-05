use fever_providers::ProviderAdapter;
use fever_providers::adapters::anthropic::AnthropicAdapter;
use fever_providers::adapters::gemini::GeminiAdapter;
use fever_providers::adapters::mock::MockProvider;
use fever_providers::adapters::ollama::OllamaAdapter;
use fever_providers::adapters::openai::OpenAiAdapter;
use fever_providers::models::{ChatMessage, ChatRequest, ToolDefinition};
use futures::StreamExt;

/// mockito::Server::new() internally calls tokio::runtime::Handle::block_on(),
/// which panics inside a #[tokio::test] ("Cannot start a runtime from within a runtime").
/// Workaround: spawn it on a blocking thread.
async fn start_mock_server() -> mockito::ServerGuard {
    tokio::task::spawn_blocking(|| mockito::Server::new())
        .await
        .expect("failed to create mock server thread")
}

fn sample_request() -> ChatRequest {
    ChatRequest {
        model: "test-model".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: Some(0.7),
        max_tokens: Some(100),
        stream: false,
    }
}

fn sample_request_with_tools() -> ChatRequest {
    ChatRequest {
        model: "test-model".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "What time is it?".to_string(),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: Some(vec![ToolDefinition {
            name: "get_time".to_string(),
            description: "Get the current time".to_string(),
            parameters: serde_json::json!({"type": "object", "properties": {}}),
        }]),
        temperature: Some(0.0),
        max_tokens: Some(100),
        stream: false,
    }
}

// ── OpenAI-compatible adapter ──

#[tokio::test]
async fn test_openai_chat_response_shape() {
    let mut server = start_mock_server().await;
    let body = serde_json::json!({
        "id": "chatcmpl-abc123",
        "object": "chat.completion",
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "Hello!"},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
    });
    server.mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body.to_string())
        .create();

    let adapter = OpenAiAdapter::custom("test".to_string(), "sk-test", server.url());
    let response = adapter.chat(&sample_request()).await.unwrap();

    assert_eq!(response.id, "chatcmpl-abc123");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].message.role, "assistant");
    assert_eq!(response.choices[0].message.content, "Hello!");
    assert_eq!(response.choices[0].finish_reason, "stop");
    assert!(response.usage.is_some());
    let usage = response.usage.unwrap();
    assert_eq!(usage.prompt_tokens, 10);
    assert_eq!(usage.completion_tokens, 5);
}

#[tokio::test]
async fn test_openai_stream_produces_chunks() {
    let mut server = start_mock_server().await;
    server.mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(
            "data: {\"id\":\"s1\",\"choices\":[{\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n\
             data: {\"id\":\"s1\",\"choices\":[{\"delta\":{\"content\":\" world\"},\"finish_reason\":null}]}\n\n\
             data: {\"id\":\"s1\",\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n\
             data: [DONE]\n\n",
        )
        .create();

    let adapter = OpenAiAdapter::custom("test".to_string(), "sk-test", server.url());
    let mut request = sample_request();
    request.stream = true;
    let mut stream = adapter.chat_stream(&request).await.unwrap();

    let mut chunks = vec![];
    while let Some(result) = stream.next().await {
        chunks.push(result.unwrap());
    }

    assert!(!chunks.is_empty());
    let content_chunks: Vec<_> = chunks.iter().filter_map(|c| c.content.as_ref()).collect();
    assert_eq!(content_chunks.len(), 2);
    assert_eq!(content_chunks[0], "Hello");
    assert_eq!(content_chunks[1], " world");

    let finish_chunks: Vec<_> = chunks.iter().filter_map(|c| c.finish_reason.as_ref()).collect();
    assert!(finish_chunks.contains(&&"stop".to_string()));
}

#[tokio::test]
async fn test_openai_stream_tool_calls_delta() {
    let mut server = start_mock_server().await;
    server.mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_1\",\"type\":\"function\",\"function\":{\"name\":\"get_time\",\"arguments\":\"\"}}]},\"finish_reason\":null}]}\n\n\
             data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"{}\"}}]},\"finish_reason\":null}]}\n\n\
             data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"tool_calls\"}]}\n\n\
             data: [DONE]\n\n",
        )
        .create();

    let adapter = OpenAiAdapter::custom("test".to_string(), "sk-test", server.url());
    let mut request = sample_request_with_tools();
    request.stream = true;
    let mut stream = adapter.chat_stream(&request).await.unwrap();

    let mut tool_chunks = vec![];
    while let Some(result) = stream.next().await {
        let chunk = result.unwrap();
        if let Some(tc) = &chunk.tool_calls {
            tool_chunks.extend(tc.clone());
        }
    }

    assert_eq!(tool_chunks.len(), 1, "Only the first delta chunk has name+id; subsequent argument-only chunks are filtered");
    assert_eq!(tool_chunks[0].name, "get_time");
    assert_eq!(tool_chunks[0].id, "call_1");
}

// ── Anthropic adapter ──

#[tokio::test]
async fn test_anthropic_chat_response_shape() {
    let mut server = start_mock_server().await;
    let body = serde_json::json!({
        "id": "msg_abc",
        "type": "message",
        "role": "assistant",
        "content": [{"type": "text", "text": "Hi there!"}],
        "model": "claude-sonnet-4-20250514",
        "stop_reason": "end_turn",
        "usage": {"input_tokens": 10, "output_tokens": 5}
    });
    server.mock("POST", "/v1/messages")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_header("x-api-key", "test")
        .with_header("anthropic-version", "2023-06-01")
        .with_body(body.to_string())
        .create();

    let adapter = AnthropicAdapter::custom("test", "sk-test", &server.url());
    let response = adapter.chat(&sample_request()).await.unwrap();

    assert!(!response.id.is_empty());
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].message.content, "Hi there!");
    assert_eq!(response.choices[0].finish_reason, "end_turn");
}

#[tokio::test]
async fn test_anthropic_stream_finish_reason_tool_use() {
    let mut server = start_mock_server().await;
    server.mock("POST", "/v1/messages")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_header("x-api-key", "test")
        .with_header("anthropic-version", "2023-06-01")
        .with_body(
            "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\"}}\n\n\
             event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"get_time\"}}\n\n\
             event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"}}\n\n\
             event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n",
        )
        .create();

    let adapter = AnthropicAdapter::custom("test", "sk-test", &server.url());
    let mut request = sample_request_with_tools();
    request.stream = true;
    let mut stream = adapter.chat_stream(&request).await.unwrap();

    let mut finish_reasons = vec![];
    let mut tool_call_chunks = vec![];
    while let Some(result) = stream.next().await {
        let chunk = result.unwrap();
        if let Some(reason) = &chunk.finish_reason {
            finish_reasons.push(reason.clone());
        }
        if let Some(tc) = &chunk.tool_calls {
            tool_call_chunks.extend(tc.clone());
        }
    }

    assert!(
        finish_reasons.contains(&"tool_calls".to_string()),
        "Anthropic stream should emit finish_reason=tool_calls when stop_reason=tool_use"
    );
    assert_eq!(tool_call_chunks.len(), 1);
    assert_eq!(tool_call_chunks[0].name, "get_time");
    assert_eq!(tool_call_chunks[0].id, "toolu_1");
}

// ── Gemini adapter (OpenAI-compatible) ──

#[tokio::test]
async fn test_gemini_chat_response_shape() {
    let mut server = start_mock_server().await;
    let body = serde_json::json!({
        "id": "gemini-abc",
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "Hey!"},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 8, "completion_tokens": 3, "total_tokens": 11}
    });
    server.mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body.to_string())
        .create();

    let adapter = GeminiAdapter::custom(
        "test",
        "key".to_string(),
        fever_providers::adapters::gemini::GeminiConfig {
            api_key: "key".to_string(),
            base_url: server.url(),
            default_model: None,
        },
    );
    let response = adapter.chat(&sample_request()).await.unwrap();

    assert_eq!(response.id, "gemini-abc");
    assert_eq!(response.choices[0].message.content, "Hey!");
    assert_eq!(response.choices[0].finish_reason, "stop");
}

// ── Ollama adapter (NDJSON) ──

#[tokio::test]
async fn test_ollama_chat_response_shape() {
    let mut server = start_mock_server().await;
    let body = serde_json::json!({
        "id": "chatcmpl-ollama",
        "object": "chat.completion",
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "Ollama says hi"},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
    });
    server.mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body.to_string())
        .create();

    let adapter = OllamaAdapter::with_url(server.url());
    let response = adapter.chat(&sample_request()).await.unwrap();

    assert!(!response.id.is_empty());
    assert_eq!(response.choices[0].message.content, "Ollama says hi");
}

#[tokio::test]
async fn test_ollama_stream_ndjson() {
    let mut server = start_mock_server().await;
    server.mock("POST", "/api/chat")
        .with_status(200)
        .with_header("content-type", "application/x-ndjson")
        .with_body(
            "{\"message\":{\"role\":\"assistant\",\"content\":\"Hi\"},\"done\":false}\n\
             {\"message\":{\"role\":\"assistant\",\"content\":\" there\"},\"done\":false}\n\
             {\"message\":{\"role\":\"assistant\",\"content\":\"!\"},\"done\":true}\n",
        )
        .create();

    let adapter = OllamaAdapter::with_url(server.url());
    let mut request = sample_request();
    request.stream = true;
    let mut stream = adapter.chat_stream(&request).await.unwrap();

    let mut all_content = String::new();
    let mut has_stop = false;
    while let Some(result) = stream.next().await {
        let chunk = result.unwrap();
        if let Some(c) = &chunk.content {
            all_content.push_str(c);
        }
        if chunk.finish_reason.as_deref() == Some("stop") {
            has_stop = true;
        }
    }

    assert_eq!(all_content, "Hi there!");
    assert!(has_stop);
}

#[tokio::test]
async fn test_ollama_stream_tool_calls() {
    let mut server = start_mock_server().await;
    server.mock("POST", "/api/chat")
        .with_status(200)
        .with_header("content-type", "application/x-ndjson")
        .with_body(
            "{\"message\":{\"role\":\"assistant\",\"content\":\"\",\"tool_calls\":[{\"function\":{\"name\":\"get_time\",\"arguments\":{\"timezone\":\"UTC\"}}}]},\"done\":true}\n",
        )
        .create();

    let adapter = OllamaAdapter::with_url(server.url());
    let mut request = sample_request_with_tools();
    request.stream = true;
    let mut stream = adapter.chat_stream(&request).await.unwrap();

    let mut tool_chunks = vec![];
    while let Some(result) = stream.next().await {
        let chunk = result.unwrap();
        if let Some(tc) = &chunk.tool_calls {
            tool_chunks.extend(tc.clone());
        }
    }

    assert_eq!(tool_chunks.len(), 1);
    assert_eq!(tool_chunks[0].name, "get_time");
}

// ── Mock adapter ──

#[tokio::test]
async fn test_mock_chat_response_shape() {
    let adapter = MockProvider::new();
    let response = adapter.chat(&sample_request()).await.unwrap();

    assert!(!response.id.is_empty());
    assert_eq!(response.choices.len(), 1);
    assert!(!response.choices[0].message.content.is_empty());
    assert_eq!(response.choices[0].finish_reason, "stop");
}

#[tokio::test]
async fn test_mock_stream_produces_chunks() {
    let adapter = MockProvider::new();
    let mut request = sample_request();
    request.stream = true;
    let mut stream = adapter.chat_stream(&request).await.unwrap();

    let mut chunks = vec![];
    while let Some(result) = stream.next().await {
        chunks.push(result.unwrap());
    }

    assert!(!chunks.is_empty());
    assert!(chunks.last().unwrap().finish_reason.is_some());
}

// ── Registry conformance ──

#[test]
fn test_registry_profiles_have_valid_adapter_types() {
    let registry = fever_providers::ProviderRegistry::builtin();
    for profile in registry.list() {
        match profile.adapter_type {
            fever_providers::AdapterType::OpenAi
            | fever_providers::AdapterType::Anthropic
            | fever_providers::AdapterType::Gemini
            | fever_providers::AdapterType::Ollama => {}
        }
    }
}

#[test]
fn test_registry_all_profiles_have_nonempty_models() {
    let registry = fever_providers::ProviderRegistry::builtin();
    for profile in registry.list() {
        assert!(
            !profile.models.is_empty(),
            "Profile '{}' has empty models list",
            profile.id
        );
    }
}

#[test]
fn test_registry_all_profiles_have_valid_env_vars() {
    let registry = fever_providers::ProviderRegistry::builtin();
    for profile in registry.list() {
        if profile.requires_auth {
            assert!(
                !profile.env_var.is_empty(),
                "Profile '{}' requires auth but has empty env_var",
                profile.id
            );
            // Env vars must be UPPER_SNAKE_CASE (letters, digits, underscores)
            assert!(
                profile
                    .env_var
                    .chars()
                    .all(|c| c.is_uppercase() || c == '_' || c.is_ascii_digit()),
                "Profile '{}' env_var '{}' should be UPPER_SNAKE_CASE",
                profile.id,
                profile.env_var
            );
        }
    }
}

#[test]
fn test_registry_no_duplicate_ids() {
    let registry = fever_providers::ProviderRegistry::builtin();
    let mut seen = std::collections::HashSet::new();
    for profile in registry.list() {
        assert!(
            seen.insert(profile.id.clone()),
            "Duplicate profile id: '{}'",
            profile.id
        );
    }
}

#[test]
fn test_registry_all_first_class_have_tools() {
    let registry = fever_providers::ProviderRegistry::builtin();
    for profile in registry.list_by_tier(fever_providers::ProviderTier::FirstClass) {
        assert!(
            profile.supports_tools,
            "First-class provider '{}' should support tools",
            profile.id
        );
        assert!(
            profile.supports_streaming,
            "First-class provider '{}' should support streaming",
            profile.id
        );
    }
}
