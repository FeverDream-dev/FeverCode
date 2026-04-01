mod local_version;

use clap::{Parser, Subcommand};
use fever_providers::ProviderClient;
use fever_providers::adapters::anthropic::AnthropicAdapter;
use fever_providers::adapters::gemini::GeminiAdapter;
use fever_providers::adapters::ollama::OllamaAdapter;
use fever_providers::adapters::openai::OpenAiAdapter;
use fever_providers::models::{ChatMessage, ChatRequest};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[clap(name = "fever", about = "Fever Code — Terminal AI Coding Agent")]
struct Cli {
    #[clap(subcommand)]
    command: Option<Command>,

    /// Run project onboarding (21-question setup)
    #[clap(long, visible_alias = "init")]
    init: bool,

    /// Re-run onboarding with existing profile
    #[clap(long)]
    re_onboard: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Send a one-shot message (no TUI)
    Chat {
        message: Vec<String>,
        #[arg(short, long)]
        model: Option<String>,
    },
    /// List configured providers
    Providers {
        /// Fetch available models from each provider
        #[arg(long)]
        fetch: bool,
    },
    /// Show version
    Version {
        #[arg(long)]
        local: bool,
        #[arg(long)]
        bump: Option<String>,
    },
    /// Run project onboarding wizard
    Init,
}

async fn build_provider_client(fetch_models: bool) -> ProviderClient {
    let mut client = ProviderClient::new();

    if let Ok(key) = env::var("OPENAI_API_KEY") {
        let adapter = OpenAiAdapter::openai(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
    }

    if let Ok(key) = env::var("OPENROUTER_API_KEY") {
        let adapter = OpenAiAdapter::openrouter(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
    }

    if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
        let adapter = AnthropicAdapter::claude(key.as_str());
        client.register(Arc::new(adapter), client.list_providers().is_empty());
    }

    if let Ok(key) = env::var("GEMINI_API_KEY") {
        let adapter = GeminiAdapter::gemini(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
    }

    if let Ok(key) = env::var("GROQ_API_KEY") {
        let adapter = OpenAiAdapter::groq(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
    }

    if let Ok(key) = env::var("TOGETHER_API_KEY") {
        let adapter = OpenAiAdapter::together(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
    }

    if let Ok(key) = env::var("DEEPSEEK_API_KEY") {
        let adapter = OpenAiAdapter::deepseek(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
    }

    if let Ok(key) = env::var("MISTRAL_API_KEY") {
        let adapter = OpenAiAdapter::mistral(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
    }

    if let Ok(key) = env::var("FIREWORKS_API_KEY") {
        let adapter = OpenAiAdapter::fireworks(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
    }

    if let Ok(key) = env::var("PERPLEXITY_API_KEY") {
        let adapter = OpenAiAdapter::perplexity(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
    }

    if let Ok(key) = env::var("MINIMAX_API_KEY") {
        let adapter = OpenAiAdapter::minimax(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
    }

    if let Ok(url) = env::var("OLLAMA_BASE_URL") {
        if url.trim().is_empty() {
            let adapter = OllamaAdapter::local();
            client.register(Arc::new(adapter), client.list_providers().is_empty());
        } else {
            let adapter = OllamaAdapter::with_url(url);
            client.register(Arc::new(adapter), client.list_providers().is_empty());
        }
    } else {
        let adapter = OllamaAdapter::local();
        client.register(Arc::new(adapter), client.list_providers().is_empty());
    }

    if let Ok(key) = env::var("FEVER_ZAI_KEY") {
        let adapter = OpenAiAdapter::openrouter(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
    }

    client
}

fn handle_version(local: bool, bump: Option<String>) {
    let home = env::var("HOME").unwrap_or(".".to_string());
    let store_path = PathBuf::from(home)
        .join(".config")
        .join("fevercode")
        .join("version.json");
    let store = local_version::VersionStore::new(store_path);
    if let Some(b) = bump {
        if let Some(kind) = local_version::parse_bump(&b) {
            let _ = store.bump(&kind);
        }
        if let Ok(v) = store.load() {
            println!("{}", v);
        }
    } else {
        println!("fever {}", env!("CARGO_PKG_VERSION"));
        if local {
            if let Ok(v) = store.load() {
                println!("local: {}", v);
            }
        }
    }
}

fn run_onboard() -> anyhow::Result<()> {
    let project_dir = std::env::current_dir()?;
    let onboarder = fever_onboard::Onboarder::new(&project_dir);
    let result = onboarder.run().map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("{}", result.summary_table);
    for file in &result.generated_files {
        println!("Generated: {}", file.path);
    }
    Ok(())
}

fn run_re_onboard() -> anyhow::Result<()> {
    let project_dir = std::env::current_dir()?;
    let onboarder = fever_onboard::Onboarder::new(&project_dir);
    let result = onboarder.re_onboard().map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("{}", result.summary_table);
    for file in &result.generated_files {
        println!("Generated: {}", file.path);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Handle --init flag or Init subcommand
    if cli.init || matches!(cli.command, Some(Command::Init)) {
        return run_onboard();
    }

    // Handle --re-onboard flag
    if cli.re_onboard {
        return run_re_onboard();
    }

    match cli.command {
        Some(Command::Chat { message, model }) => {
            let client = build_provider_client(false).await;
            let content = message.join(" ");
            let model = model.unwrap_or_else(|| "gpt-4o".to_string());
            let req = ChatRequest {
                model,
                messages: vec![ChatMessage {
                    role: "user".to_string(),
                    content,
                    tool_calls: None,
                    tool_call_id: None,
                }],
                tools: None,
                temperature: None,
                max_tokens: None,
                stream: false,
            };
            match client.chat(&req).await {
                Ok(resp) => {
                    if let Some(choice) = resp.choices.first() {
                        println!("{}", choice.message.content);
                    }
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }
        }
        Some(Command::Providers { fetch }) => {
            let client = build_provider_client(fetch).await;
            let providers = client.list_providers();
            if providers.is_empty() {
                println!("No providers configured");
                return Ok(());
            }
            for name in providers {
                if let Some(adapter) = client.get_provider(&name) {
                    println!("Provider: {}", name);
                    let caps = adapter.capabilities();
                    println!("  supports_chat: {}", caps.supports_chat);
                    let models = adapter.list_models();
                    for (i, m) in models.iter().take(5).enumerate() {
                        println!("  model {}: {}", i + 1, m);
                    }
                    println!();
                }
            }
        }
        Some(Command::Version { local, bump }) => {
            handle_version(local, bump);
        }
        Some(Command::Init) => {
            run_onboard()?;
        }
        None => {
            let mut app = fever_tui::AppState::new();
            app.run()?;
        }
    }

    Ok(())
}
