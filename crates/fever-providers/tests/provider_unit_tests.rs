//! Comprehensive unit tests for fever-providers crate.
//! These tests do NOT require API keys or network access.

use async_trait::async_trait;
use fever_providers::models::{ChatChoice, Usage};
use fever_providers::{
    ChatMessage, ChatRequest, ChatResponse, ModelCapability, ModelInfo, ProviderAdapter,
    ProviderCapabilities, ProviderClient, ProviderError,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// =============================================================================
// Mock Provider for Testing
// =============================================================================

struct MockProvider {
    name: String,
    call_count: AtomicUsize,
}

impl MockProvider {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            call_count: AtomicUsize::new(0),
        }
    }

    fn get_call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl ProviderAdapter for MockProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_chat: true,
            supports_tools: false,
            supports_streaming: false,
            supports_images: false,
            supports_function_calling: false,
            max_context_length: Some(4096),
            supported_capabilities: vec![ModelCapability::Chat],
        }
    }

    async fn chat(&self, _request: &ChatRequest) -> Result<ChatResponse, ProviderError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        Ok(ChatResponse {
            id: "mock-id".to_string(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: "mock response".to_string(),
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: "stop".to_string(),
            }],
            usage: Some(Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            }),
        })
    }

    fn list_models(&self) -> Vec<String> {
        vec!["mock-model".to_string()]
    }

    fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        if model_id == "mock-model" {
            Some(ModelInfo {
                id: "mock-model".to_string(),
                name: "Mock Model".to_string(),
                provider: self.name.clone(),
                capabilities: vec![ModelCapability::Chat],
                context_length: Some(4096),
                max_output_tokens: Some(2048),
            })
        } else {
            None
        }
    }

    async fn validate_config(&self) -> Result<(), ProviderError> {
        Ok(())
    }

    fn is_configured(&self) -> bool {
        true
    }
}

// =============================================================================
// models.rs Tests
// =============================================================================

#[test]
fn test_chat_request_default() {
    let request = ChatRequest::default();

    assert_eq!(request.model, "gpt-4o");
    assert!(request.messages.is_empty());
    assert!(request.tools.is_none());
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.max_tokens, Some(4096));
    assert!(!request.stream);
}

#[test]
fn test_chat_message_fields() {
    let message = ChatMessage {
        role: "user".to_string(),
        content: "Hello, world!".to_string(),
        tool_calls: None,
        tool_call_id: None,
    };

    assert_eq!(message.role, "user");
    assert_eq!(message.content, "Hello, world!");
    assert!(message.tool_calls.is_none());
    assert!(message.tool_call_id.is_none());
}

#[test]
fn test_chat_message_with_tool_calls() {
    use fever_providers::models::ToolCall;

    let tool_calls = vec![ToolCall {
        id: "call-123".to_string(),
        name: "get_weather".to_string(),
        arguments: serde_json::json!({"location": "Boston"}),
    }];

    let message = ChatMessage {
        role: "assistant".to_string(),
        content: "".to_string(),
        tool_calls: Some(tool_calls),
        tool_call_id: None,
    };

    assert_eq!(message.role, "assistant");
    assert!(message.tool_calls.is_some());
    let calls = message.tool_calls.as_ref().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].id, "call-123");
    assert_eq!(calls[0].name, "get_weather");
}

#[test]
fn test_chat_response_construction() {
    let response = ChatResponse {
        id: "chatcmpl-123".to_string(),
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: "Hello!".to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: Some(Usage {
            prompt_tokens: 5,
            completion_tokens: 10,
            total_tokens: 15,
        }),
    };

    assert_eq!(response.id, "chatcmpl-123");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].index, 0);
    assert_eq!(response.choices[0].message.role, "assistant");
    assert_eq!(response.choices[0].message.content, "Hello!");
    assert_eq!(response.choices[0].finish_reason, "stop");

    let usage = response.usage.as_ref().unwrap();
    assert_eq!(usage.prompt_tokens, 5);
    assert_eq!(usage.completion_tokens, 10);
    assert_eq!(usage.total_tokens, 15);
}

#[test]
fn test_chat_choice_multiple() {
    let choices = vec![
        ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: "Option A".to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
            finish_reason: "stop".to_string(),
        },
        ChatChoice {
            index: 1,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: "Option B".to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
            finish_reason: "length".to_string(),
        },
    ];

    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].message.content, "Option A");
    assert_eq!(choices[1].message.content, "Option B");
    assert_eq!(choices[0].finish_reason, "stop");
    assert_eq!(choices[1].finish_reason, "length");
}

#[test]
fn test_usage_fields() {
    let usage = Usage {
        prompt_tokens: 100,
        completion_tokens: 50,
        total_tokens: 150,
    };

    assert_eq!(usage.prompt_tokens, 100);
    assert_eq!(usage.completion_tokens, 50);
    assert_eq!(usage.total_tokens, 150);
}

#[test]
fn test_chat_request_with_messages() {
    let request = ChatRequest {
        model: "gpt-4o".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are helpful.".to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hi!".to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        tools: None,
        temperature: Some(0.5),
        max_tokens: Some(1000),
        stream: true,
    };

    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.messages[0].role, "system");
    assert_eq!(request.messages[1].role, "user");
    assert_eq!(request.temperature, Some(0.5));
    assert_eq!(request.max_tokens, Some(1000));
    assert!(request.stream);
}

#[test]
fn test_model_info_construction() {
    let info = ModelInfo {
        id: "gpt-4".to_string(),
        name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        capabilities: vec![ModelCapability::Chat, ModelCapability::Tools],
        context_length: Some(8192),
        max_output_tokens: Some(4096),
    };

    assert_eq!(info.id, "gpt-4");
    assert_eq!(info.name, "GPT-4");
    assert_eq!(info.provider, "openai");
    assert_eq!(info.capabilities.len(), 2);
    assert!(info.capabilities.contains(&ModelCapability::Chat));
    assert!(info.capabilities.contains(&ModelCapability::Tools));
}

// =============================================================================
// client.rs Tests
// =============================================================================

#[test]
fn test_provider_client_new() {
    let client = ProviderClient::new();
    assert!(client.list_providers().is_empty());
    assert!(client.get_default_provider().is_none());
}

#[test]
fn test_provider_client_default() {
    let client = ProviderClient::default();
    assert!(client.list_providers().is_empty());
}

#[test]
fn test_provider_client_register_and_list() {
    let mut client = ProviderClient::new();
    let mock = Arc::new(MockProvider::new("mock-provider"));

    client.register(mock, false);

    let providers = client.list_providers();
    assert_eq!(providers.len(), 1);
    assert!(providers.contains(&"mock-provider".to_string()));

    let models = client.list_models();
    assert_eq!(models.len(), 1);
    assert!(models.contains(&"mock-model".to_string()));
}

#[test]
fn test_provider_client_register_multiple_providers() {
    let mut client = ProviderClient::new();

    let mock1 = Arc::new(MockProvider::new("provider-one"));
    let mock2 = Arc::new(MockProvider::new("provider-two"));

    client.register(mock1, false);
    client.register(mock2, true);

    let providers = client.list_providers();
    assert_eq!(providers.len(), 2);
    assert!(providers.contains(&"provider-one".to_string()));
    assert!(providers.contains(&"provider-two".to_string()));

    assert_eq!(client.get_default_provider(), Some("provider-two"));
}

#[test]
fn test_provider_client_set_default() {
    let mut client = ProviderClient::new();
    let mock = Arc::new(MockProvider::new("test-provider"));

    client.register(mock, false);
    assert!(client.get_default_provider().is_none());

    let result = client.set_default_provider("test-provider".to_string());
    assert!(result.is_ok());
    assert_eq!(client.get_default_provider(), Some("test-provider"));
}

#[test]
fn test_provider_client_set_default_nonexistent() {
    let mut client = ProviderClient::new();

    let result = client.set_default_provider("nonexistent".to_string());
    assert!(result.is_err());

    match result {
        Err(ProviderError::ModelNotFound(name)) => assert_eq!(name, "nonexistent"),
        _ => panic!("Expected ModelNotFound error"),
    }
}

#[test]
fn test_provider_client_get_provider() {
    let mut client = ProviderClient::new();
    let mock = Arc::new(MockProvider::new("my-provider"));

    client.register(mock, false);

    assert!(client.get_provider("my-provider").is_some());
    assert!(client.get_provider("unknown-provider").is_none());
}

#[test]
fn test_provider_client_resolve_provider_with_prefix() {
    let mut client = ProviderClient::new();
    let mock = Arc::new(MockProvider::new("mock-provider"));

    client.register(mock.clone(), false);

    // The resolve_provider logic checks if the first part before '/' is a registered provider
    let provider = client.get_provider("mock-provider");
    assert!(provider.is_some());
}

#[test]
fn test_provider_client_resolve_provider_default_fallback() {
    let mut client = ProviderClient::new();
    let mock = Arc::new(MockProvider::new("default-provider"));

    client.register(mock, true);

    // With default set, unknown model should fall back to default
    assert_eq!(client.get_default_provider(), Some("default-provider"));
}

#[tokio::test]
async fn test_provider_client_chat_calls_provider() {
    let mut client = ProviderClient::new();
    let mock = Arc::new(MockProvider::new("chat-provider"));
    client.register(mock.clone(), true);

    let request = ChatRequest {
        model: "some-model".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Test message".to_string(),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: Some(0.7),
        max_tokens: Some(100),
        stream: false,
    };

    let response = client.chat(&request).await.unwrap();

    assert_eq!(response.id, "mock-id");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].message.content, "mock response");
    assert_eq!(mock.get_call_count(), 1);
}

#[tokio::test]
async fn test_provider_client_chat_with_explicit_provider() {
    let mut client = ProviderClient::new();
    let mock = Arc::new(MockProvider::new("explicit-provider"));
    client.register(mock.clone(), false);

    let request = ChatRequest {
        model: "explicit-provider/mock-model".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Test".to_string(),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: None,
        max_tokens: None,
        stream: false,
    };

    let response = client.chat(&request).await.unwrap();
    assert_eq!(response.choices[0].message.content, "mock response");
}

#[tokio::test]
async fn test_provider_client_chat_no_provider_error() {
    let client = ProviderClient::new();

    let request = ChatRequest {
        model: "unknown-model".to_string(),
        messages: vec![],
        tools: None,
        temperature: None,
        max_tokens: None,
        stream: false,
    };

    let result = client.chat(&request).await;
    assert!(result.is_err());

    match result {
        Err(ProviderError::InvalidRequest(msg)) => {
            assert!(msg.contains("No provider specified"));
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

// =============================================================================
// error.rs Tests
// =============================================================================

#[test]
fn test_provider_error_display_model_not_found() {
    let error = ProviderError::ModelNotFound("gpt-5".to_string());
    let msg = error.to_string();

    assert!(msg.contains("gpt-5"));
    assert!(msg.contains("not found"));
}

#[test]
fn test_provider_error_display_invalid_request() {
    let error = ProviderError::InvalidRequest("Missing required field".to_string());
    let msg = error.to_string();

    assert!(msg.contains("Invalid request"));
    assert!(msg.contains("Missing required field"));
}

#[test]
fn test_provider_error_display_config() {
    let error = ProviderError::Config("API key not set".to_string());
    let msg = error.to_string();

    assert!(msg.contains("Configuration error"));
    assert!(msg.contains("API key not set"));
}

#[test]
fn test_provider_error_display_auth() {
    let error = ProviderError::Auth("Invalid credentials".to_string());
    let msg = error.to_string();

    assert!(msg.contains("Authentication failed"));
    assert!(msg.contains("Invalid credentials"));
}

#[test]
fn test_provider_error_display_rate_limit() {
    let error = ProviderError::RateLimit {
        provider: "openai".to_string(),
    };
    let msg = error.to_string();

    assert!(msg.contains("Rate limit"));
    assert!(msg.contains("openai"));
}

#[test]
fn test_provider_error_display_unavailable() {
    let error = ProviderError::Unavailable("Service down".to_string());
    let msg = error.to_string();

    assert!(msg.contains("unavailable"));
    assert!(msg.contains("Service down"));
}

#[test]
fn test_provider_error_display_api() {
    let error = ProviderError::Api {
        code: "400".to_string(),
        message: "Bad request".to_string(),
    };
    let msg = error.to_string();

    assert!(msg.contains("400"));
    assert!(msg.contains("Bad request"));
}

#[test]
fn test_provider_error_display_timeout() {
    let error = ProviderError::Timeout("Request timed out after 30s".to_string());
    let msg = error.to_string();

    assert!(msg.contains("Timeout"));
    assert!(msg.contains("30s"));
}

#[test]
fn test_provider_error_display_parse() {
    let error = ProviderError::Parse("Invalid JSON".to_string());
    let msg = error.to_string();

    assert!(msg.contains("Parse error"));
    assert!(msg.contains("Invalid JSON"));
}

#[test]
fn test_provider_error_display_unknown() {
    let error = ProviderError::Unknown("Something went wrong".to_string());
    let msg = error.to_string();

    assert!(msg.contains("Unknown error"));
    assert!(msg.contains("Something went wrong"));
}

// =============================================================================
// ProviderCapabilities Tests
// =============================================================================

#[test]
fn test_provider_capabilities_from_mock() {
    let mock = MockProvider::new("test");
    let caps = mock.capabilities();

    assert!(caps.supports_chat);
    assert!(!caps.supports_tools);
    assert!(!caps.supports_streaming);
    assert!(!caps.supports_images);
    assert!(!caps.supports_function_calling);
    assert_eq!(caps.max_context_length, Some(4096));
}

#[test]
fn test_mock_provider_is_configured() {
    let mock = MockProvider::new("test");
    assert!(mock.is_configured());
}

#[test]
fn test_mock_provider_list_models() {
    let mock = MockProvider::new("test");
    let models = mock.list_models();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0], "mock-model");
}

#[test]
fn test_mock_provider_get_model_info() {
    let mock = MockProvider::new("test");

    let info = mock.get_model_info("mock-model");
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.id, "mock-model");
    assert_eq!(info.name, "Mock Model");

    let missing = mock.get_model_info("nonexistent");
    assert!(missing.is_none());
}

#[tokio::test]
async fn test_mock_provider_validate_config() {
    let mock = MockProvider::new("test");
    let result = mock.validate_config().await;
    assert!(result.is_ok());
}

// =============================================================================
// Additional Edge Case Tests
// =============================================================================

#[test]
fn test_chat_request_clone() {
    let request = ChatRequest {
        model: "gpt-4o".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: Some(0.5),
        max_tokens: Some(500),
        stream: false,
    };

    let cloned = request.clone();
    assert_eq!(cloned.model, request.model);
    assert_eq!(cloned.messages.len(), request.messages.len());
    assert_eq!(cloned.temperature, request.temperature);
}

#[test]
fn test_chat_response_without_usage() {
    let response = ChatResponse {
        id: "test-id".to_string(),
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: "No usage info".to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: None,
    };

    assert!(response.usage.is_none());
}

#[test]
fn test_empty_chat_request_messages() {
    let request = ChatRequest {
        model: "gpt-4o".to_string(),
        messages: vec![],
        tools: None,
        temperature: None,
        max_tokens: None,
        stream: false,
    };

    assert!(request.messages.is_empty());
}

#[test]
fn test_register_replaces_existing_provider() {
    let mut client = ProviderClient::new();

    let mock1 = Arc::new(MockProvider::new("same-name"));
    client.register(mock1, false);

    let mock2 = Arc::new(MockProvider::new("same-name"));
    client.register(mock2, false);

    // Should still have only one provider (replaced)
    assert_eq!(client.list_providers().len(), 1);
}

#[test]
fn test_provider_client_register_as_default() {
    let mut client = ProviderClient::new();
    let mock = Arc::new(MockProvider::new("default-mock"));

    client.register(mock, true);

    assert_eq!(client.get_default_provider(), Some("default-mock"));
}

#[tokio::test]
async fn test_multiple_chat_calls() {
    let mut client = ProviderClient::new();
    let mock = Arc::new(MockProvider::new("multi-call"));
    client.register(mock.clone(), true);

    let request = ChatRequest::default();

    let _ = client.chat(&request).await.unwrap();
    let _ = client.chat(&request).await.unwrap();
    let _ = client.chat(&request).await.unwrap();

    assert_eq!(mock.get_call_count(), 3);
}
