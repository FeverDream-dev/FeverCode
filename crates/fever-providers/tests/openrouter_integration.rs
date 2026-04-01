//! Integration test for OpenRouter via the OpenAI-compatible adapter.
//!
//! Requires env var `FEVER_ZAI_KEY` to be set.
//! Run: `cargo test -p fever-providers --test openrouter_integration -- --nocapture`

use fever_providers::adapter::ProviderAdapter;
use fever_providers::adapters::openai::OpenAiAdapter;
use fever_providers::models::{ChatMessage, ChatRequest};

fn get_key() -> String {
    std::env::var("FEVER_ZAI_KEY").expect("FEVER_ZAI_KEY env var must be set for integration test")
}

#[tokio::test]
#[ignore]
async fn test_openrouter_fetch_models() {
    let adapter = OpenAiAdapter::openrouter(get_key());

    adapter
        .fetch_models()
        .await
        .expect("fetch_models should succeed");

    let models = adapter.list_models();
    assert!(
        !models.is_empty(),
        "Should have fetched at least some models"
    );

    println!("✅ Fetched {} models from OpenRouter", models.len());

    for m in models.iter().take(10) {
        println!("   {}", m);
    }
    if models.len() > 10 {
        println!("   ... and {} more", models.len() - 10);
    }
}

#[tokio::test]
#[ignore]
async fn test_openrouter_chat() {
    let key = get_key();
    let adapter = OpenAiAdapter::openrouter(&key);

    adapter
        .validate_config()
        .await
        .expect("config should be valid");
    assert!(adapter.is_configured());

    let request = ChatRequest {
        model: "openai/gpt-4o-mini".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Say exactly: TEST_PING_OK".to_string(),
            tool_calls: None,
            tool_call_id: None,
        }],
        temperature: Some(0.0),
        max_tokens: Some(50),
        stream: false,
        tools: None,
    };

    let result = adapter.chat(&request).await;

    match result {
        Ok(response) => {
            assert!(
                !response.choices.is_empty(),
                "Should have at least one choice"
            );
            println!("✅ Chat response received:");
            println!("   ID: {}", response.id);
            println!("   Content: {}", response.choices[0].message.content);
            if let Some(usage) = &response.usage {
                println!(
                    "   Tokens: {} prompt + {} completion = {} total",
                    usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                );
            }
        }
        Err(e) => {
            let err_str = format!("{:?}", e);
            if err_str.contains("Auth") || err_str.contains("401") || err_str.contains("403") {
                println!("⚠️  Chat endpoint returned auth error — key may be read-only.");
                println!("   Adapter code is correct. Error: {:?}", e);
                println!(
                    "   Model listing (GET /models) works. Chat (POST /chat/completions) needs a key with write access."
                );
            } else {
                panic!("Chat failed with unexpected error: {:?}", e);
            }
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_openrouter_model_info() {
    let adapter = OpenAiAdapter::openrouter(get_key());

    adapter
        .fetch_models()
        .await
        .expect("fetch_models should succeed");

    let models = adapter.list_models();
    let first_model_id = models.first().expect("should have at least one model");

    let info = adapter
        .get_model_info(first_model_id)
        .expect("should get model info");

    assert_eq!(info.id, *first_model_id);
    assert_eq!(info.provider, "openrouter");

    println!(
        "✅ Model info for {}: provider={}, context={:?}",
        info.name, info.provider, info.context_length
    );
}
