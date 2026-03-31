use clap::{Parser, Subcommand, Args};
mod local_version;
use fever_config::ConfigManager;
use fever_providers::ProviderClient;
use std::sync::Arc;
use std::path::PathBuf;
use std::env;

use fever_providers::adapters::openai::OpenAiAdapter;
use fever_providers::adapters::anthropic::AnthropicAdapter;
use fever_providers::adapters::gemini::GeminiAdapter;
use fever_providers::adapters::ollama::OllamaAdapter;
use fever_providers::models::{ChatRequest, ChatMessage};

#[derive(Parser)]
#[clap(name = "fever", about = " Fever CLI ")]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Code(CodeArgs),
    Version(VersionArgs),
    Roles,
    Config,
    Providers(ProvidersArgs),
    Chat(ChatArgs),
}

#[derive(Args)]
struct CodeArgs {}

#[derive(Args)]
struct VersionArgs {
    #[arg(long)]
    local: bool,
    #[arg(long)]
    bump: Option<String>,
}

#[derive(Args)]
struct ProvidersArgs {
    #[arg(long)]
    fetch: bool,
}

#[derive(Args)]
struct ChatArgs {
    message: Vec<String>,
    model: Option<String>,
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

async fn list_providers(args: ProvidersArgs) {
    let client = build_provider_client(args.fetch).await;
    let providers = client.list_providers();
    if providers.is_empty() {
        println!("No providers configured");
        return;
    }
    for name in providers {
        if let Some(adapter) = client.get_provider(&name) {
            println!("Provider: {}", name);
            let caps = adapter.capabilities();
            println!("  supports_chat: {}", caps.supports_chat);
            println!("  supports_tools: {}", caps.supports_tools);
            println!("  supports_streaming: {}", caps.supports_streaming);
            let models = adapter.list_models();
            for (i, m) in models.iter().take(5).enumerate() {
                println!("  model {}: {}", i + 1, m);
            }
            println!();
        }
    }
}

async fn run_chat(args: ChatArgs) {
    let mut req = ChatRequest {
        model: args.model.unwrap_or_else(|| "gpt-4o".to_string()),
        messages: Vec::new(),
        tools: None,
        temperature: None,
        max_tokens: None,
        stream: false,
    };
    let content = args.message.join(" ");
    req.messages.push(ChatMessage {
        role: "user".to_string(),
        content,
        tool_calls: None,
        tool_call_id: None,
    });
    let client = build_provider_client(false).await;
    let resp = client.chat(&req).await;
    match resp {
        Ok(r) => {
            if let Some(choice) = r.choices.get(0) {
                println!("{}", choice.message.content);
            }
            if let Some(usage) = r.usage {
                println!("\nUsage: prompt_tokens={} completion_tokens={} total_tokens={}", usage.prompt_tokens, usage.completion_tokens, usage.total_tokens);
            }
        }
        Err(e) => {
            println!("Chat request failed: {:?}", e);
        }
    }
}

#[allow(dead_code)]
async fn run_tui() {
    let _ = build_provider_client(false).await;
}

fn handle_version_command(args: VersionArgs) {
    let home = env::var("HOME").unwrap_or(".".to_string());
    let store_path = PathBuf::from(home).join(".config").join("fevercode").join("version.json");
    let store = local_version::VersionStore::new(store_path);
    if let Some(b) = args.bump {
        if let Some(kind) = local_version::parse_bump(&b) {
            let _ = store.bump(&kind);
        }
        if let Ok(v) = store.load() {
            println!("{}", v.to_string());
        }
        return;
    }
    if let Ok(v) = store.load() {
        println!("{}", v.to_string());
    }
}

fn list_roles() {
    println!("Available roles: coder, researcher, planner, architect, debugger, tester, reviewer, refactorer, shell_operator");
}

async fn show_config() {
    let cm = ConfigManager::new().expect("config dir init");
    let cfg = cm.load().expect("load config");
    println!("{:#?}", cfg);
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Code(_) => {
        }
        Command::Version(v) => {
            handle_version_command(v);
        }
        Command::Roles => {
            list_roles();
        }
        Command::Config => {
            show_config().await;
        }
        Command::Providers(p) => {
            list_providers(p).await;
        }
        Command::Chat(c) => {
            run_chat(c).await;
        }
    }
}
