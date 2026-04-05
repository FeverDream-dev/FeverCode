mod agent_handle;
mod local_version;

use clap::{Parser, Subcommand};
use fever_core::ToolRegistry;
use fever_providers::ProviderClient;
use fever_providers::adapters::anthropic::AnthropicAdapter;
use fever_providers::adapters::gemini::GeminiAdapter;
use fever_providers::adapters::ollama::OllamaAdapter;
use fever_providers::adapters::openai::OpenAiAdapter;
use fever_providers::models::{ChatMessage, ChatRequest};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use agent_handle::FeverAgentHandle;
use fever_agent::AgentConfig;

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

    /// Increase verbosity (-v, -vv, -vvv)
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Run in mock mode (no real API keys required)
    #[clap(long)]
    mock: bool,
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
        /// Test a specific provider's connectivity
        #[arg(long, value_name = "NAME")]
        test: Option<String>,
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
    /// Check system health and configuration
    Doctor,
    /// Show or manage configuration
    Config {
        /// Show full config file path
        #[arg(long)]
        path: bool,
        /// Show current configuration values
        #[arg(long)]
        show: bool,
        /// Validate configuration without making changes
        #[arg(long)]
        validate: bool,
        /// Open config file in $EDITOR
        #[arg(long)]
        edit: bool,
    },
    /// List available models from all configured providers
    Models {
        /// Filter models by provider name
        #[arg(short, long)]
        provider: Option<String>,
    },
    /// Run a prompt non-interactively and print the response
    Run {
        /// The prompt to send
        prompt: Vec<String>,
        /// Model to use (provider/model format)
        #[arg(short, long)]
        model: Option<String>,
        /// Output raw JSON response
        #[arg(long)]
        json: bool,
    },
    /// Manage chat sessions
    Session {
        /// Action: list, show, clear
        #[arg(default_value = "list")]
        action: String,
        /// Session ID (for show/clear)
        #[arg(short, long)]
        id: Option<String>,
    },
}

async fn build_provider_client(fetch_models: bool) -> ProviderClient {
    tracing::debug!("Building provider client (fetch_models={fetch_models})");
    let mut client = ProviderClient::new();

    if let Ok(key) = env::var("OPENAI_API_KEY") {
        let adapter = OpenAiAdapter::openai(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
        tracing::info!("Registered provider: openai");
    }

    if let Ok(key) = env::var("OPENROUTER_API_KEY") {
        let adapter = OpenAiAdapter::openrouter(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
        tracing::info!("Registered provider: openrouter");
    }

    if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
        let adapter = AnthropicAdapter::claude(key.as_str());
        client.register(Arc::new(adapter), client.list_providers().is_empty());
        tracing::info!("Registered provider: anthropic");
    }

    if let Ok(key) = env::var("GEMINI_API_KEY") {
        let adapter = GeminiAdapter::gemini(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
        tracing::info!("Registered provider: gemini");
    }

    if let Ok(key) = env::var("GROQ_API_KEY") {
        let adapter = OpenAiAdapter::groq(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
        tracing::info!("Registered provider: groq");
    }

    if let Ok(key) = env::var("TOGETHER_API_KEY") {
        let adapter = OpenAiAdapter::together(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
        tracing::info!("Registered provider: together");
    }

    if let Ok(key) = env::var("DEEPSEEK_API_KEY") {
        let adapter = OpenAiAdapter::deepseek(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
        tracing::info!("Registered provider: deepseek");
    }

    if let Ok(key) = env::var("MISTRAL_API_KEY") {
        let adapter = OpenAiAdapter::mistral(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
        tracing::info!("Registered provider: mistral");
    }

    if let Ok(key) = env::var("FIREWORKS_API_KEY") {
        let adapter = OpenAiAdapter::fireworks(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
        tracing::info!("Registered provider: fireworks");
    }

    if let Ok(key) = env::var("PERPLEXITY_API_KEY") {
        let adapter = OpenAiAdapter::perplexity(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
        tracing::info!("Registered provider: perplexity");
    }

    if let Ok(key) = env::var("MINIMAX_API_KEY") {
        let adapter = OpenAiAdapter::minimax(key);
        if fetch_models {
            let _ = adapter.fetch_models().await;
        }
        client.register(Arc::new(adapter), client.list_providers().is_empty());
        tracing::info!("Registered provider: minimax");
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
    tracing::info!("Registered provider: ollama");

    if let Ok(key) = env::var("ZAI_API_KEY") {
        let adapter = OpenAiAdapter::zai(key);
        client.register(Arc::new(adapter), client.list_providers().is_empty());
        tracing::info!("Registered provider: zai");
    }

    // Register providers from config.toml
    if let Ok(cm) = fever_config::ConfigManager::new() {
        if let Ok(config) = cm.load() {
            for (name, prov_config) in &config.providers {
                // Skip disabled providers or those without API keys
                if !prov_config.enabled {
                    continue;
                }
                let Some(api_key) = &prov_config.api_key else {
                    continue;
                };

                // Skip if already registered from env vars
                if client.get_provider(name).is_some() {
                    tracing::debug!("Provider '{}' already registered, skipping config", name);
                    continue;
                }

                // Create custom adapter with the provider's base_url or default
                let base_url = prov_config
                    .base_url
                    .clone()
                    .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
                let adapter = OpenAiAdapter::custom(name.clone(), api_key.clone(), base_url);

                if fetch_models {
                    let _ = adapter.fetch_models().await;
                }

                client.register(Arc::new(adapter), false);
                tracing::info!("Registered provider from config: {}", name);
            }

            // Set default provider from config if specified
            if let Some(default_provider) = &config.defaults.provider {
                let _ = client.set_default_provider(default_provider.clone());
            }
        }
    }

    client
}

fn build_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    let _ = registry.register(Box::new(fever_tools::ShellTool::new()));
    let _ = registry.register(Box::new(fever_tools::FilesystemTool::new()));
    let _ = registry.register(Box::new(fever_tools::GitTool::new()));
    let _ = registry.register(Box::new(fever_tools::GrepTool::new()));
    registry
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
    let result = onboarder
        .re_onboard()
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("{}", result.summary_table);
    for file in &result.generated_files {
        println!("Generated: {}", file.path);
    }
    Ok(())
}

fn run_doctor() -> anyhow::Result<()> {
    let mut checks = Vec::new();

    checks.push(("Config directory", {
        let cm = fever_config::ConfigManager::new();
        match cm {
            Ok(_) => ("ok".to_string(), true),
            Err(e) => (format!("failed: {e}"), false),
        }
    }));

    let cm = fever_config::ConfigManager::new().ok();
    let config = cm.as_ref().and_then(|c| c.load().ok());

    checks.push(("Config file", {
        if let Some(ref cm) = cm {
            let path = cm.config_path();
            if path.exists() {
                ("found".to_string(), true)
            } else {
                ("missing (run `fever init` to create)".to_string(), false)
            }
        } else {
            ("unknown (config dir failed)".to_string(), false)
        }
    }));

    let provider_env_vars = [
        ("OPENAI_API_KEY", "OpenAI"),
        ("OPENROUTER_API_KEY", "OpenRouter"),
        ("ANTHROPIC_API_KEY", "Anthropic"),
        ("GEMINI_API_KEY", "Gemini"),
        ("GROQ_API_KEY", "Groq"),
        ("DEEPSEEK_API_KEY", "DeepSeek"),
        ("MISTRAL_API_KEY", "Mistral"),
    ];

    let mut provider_count = 0usize;
    for (var, name) in &provider_env_vars {
        if std::env::var(var).is_ok() {
            provider_count += 1;
            checks.push((name, (format!("key set ({var})"), true)));
        }
    }
    if provider_count == 0 {
        checks.push(("Providers", ("none configured".to_string(), false)));
    }

    if let Some(ref config) = config {
        let enabled: Vec<_> = config
            .providers
            .iter()
            .filter(|(_, p)| p.enabled)
            .map(|(n, _)| n.as_str())
            .collect();
        checks.push((
            "Config providers",
            (
                if enabled.is_empty() {
                    "none enabled in config".to_string()
                } else {
                    format!("{}: {}", enabled.len(), enabled.join(", "))
                },
                !enabled.is_empty(),
            ),
        ));
        if let Some(ref provider) = config.defaults.provider {
            checks.push(("Default provider", (provider.clone(), true)));
        }
        if let Some(ref model) = config.defaults.model {
            checks.push(("Default model", (model.clone(), true)));
        }
    }

    checks.push(("Terminal (TTY)", {
        use std::io::IsTerminal;
        if std::io::stdout().is_terminal() {
            ("interactive".to_string(), true)
        } else {
            ("non-interactive (piped)".to_string(), true)
        }
    }));

    checks.push(("Working directory", {
        match std::env::current_dir() {
            Ok(cwd) => (format!("{}", cwd.display()), true),
            Err(e) => (format!("cannot read: {e}"), false),
        }
    }));

    checks.push((".git directory", {
        let git_dir = std::env::current_dir().ok().map(|d| d.join(".git"));
        match git_dir {
            Some(ref d) if d.exists() => ("found".to_string(), true),
            Some(_) => ("not found (not a git repo)".to_string(), false),
            None => ("cannot check".to_string(), false),
        }
    }));

    println!("\n  Fever Doctor — System Health Check\n");
    let all_pass = checks.iter().all(|(_, (_, pass))| *pass);
    for (name, (status, pass)) in &checks {
        let icon = if *pass { "\u{2713}" } else { "\u{2717}" };
        let style = if *pass { "\x1b[32m" } else { "\x1b[33m" };
        let reset = "\x1b[0m";
        println!("  {style}{icon}{reset}  {:<24} {}", name, status);
    }

    println!();
    if all_pass {
        println!("  All checks passed. Fever is ready to run.");
    } else {
        println!("  Some checks failed. See above for details.");
        println!("  Run `fever init` to set up project configuration.");
    }
    println!();

    Ok(())
}

fn run_config(path: bool, show: bool, validate: bool, edit: bool) -> anyhow::Result<()> {
    let cm = fever_config::ConfigManager::new()
        .map_err(|e| anyhow::anyhow!("Cannot access config directory: {e}"))?;

    if edit {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
        let config_path = cm.config_path();
        if !config_path.exists() {
            let config = fever_config::Config::default();
            cm.save(&config)?;
            println!("Created default config at: {}", config_path.display());
        }
        let status = std::process::Command::new(&editor)
            .arg(config_path)
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to run editor '{}': {}", editor, e))?;
        if !status.success() {
            eprintln!("Editor exited with non-zero status");
        }
        return Ok(());
    }

    if path {
        println!("{}", cm.config_path().display());
        return Ok(());
    }

    if show {
        let config = cm.load()?;
        println!("Config path: {}\n", cm.config_path().display());
        println!("Config dir:  {}", cm.config_dir().display());
        println!("Data dir:   {}", cm.data_dir().display());
        println!("Cache dir:  {}\n", cm.cache_dir().display());
        println!("{}", toml::to_string_pretty(&config)?);
        return Ok(());
    }

    if validate {
        let config = cm.load()?;
        let mut warnings: Vec<String> = Vec::new();

        if config.providers.is_empty() {
            warnings.push("No providers configured in config.toml".to_string());
        }
        if config.defaults.provider.is_none() && config.providers.is_empty() {
            warnings.push("No default provider set".to_string());
        }

        let provider_env_vars = [
            "OPENAI_API_KEY",
            "OPENROUTER_API_KEY",
            "ANTHROPIC_API_KEY",
            "GEMINI_API_KEY",
            "GROQ_API_KEY",
            "DEEPSEEK_API_KEY",
            "MISTRAL_API_KEY",
        ];
        let has_any_key = provider_env_vars.iter().any(|v| std::env::var(v).is_ok());
        if !has_any_key && config.providers.is_empty() {
            warnings.push("No API keys found in environment variables".to_string());
        }

        let enabled: Vec<_> = config.providers.iter().filter(|(_, p)| p.enabled).collect();
        for (name, prov) in &enabled {
            if prov.api_key.is_none() {
                warnings.push(format!(
                    "Provider '{name}' is enabled but has no api_key set"
                ));
            }
        }

        if warnings.is_empty() {
            println!("Configuration is valid.");
        } else {
            for w in &warnings {
                println!("  \u{26a0}  {w}");
            }
        }
        return Ok(());
    }

    println!("Usage: fever config [OPTIONS]");
    println!("  --path      Show config file path");
    println!("  --show      Show current configuration");
    println!("  --validate  Validate configuration");
    println!("  --edit      Open config in $EDITOR");

    Ok(())
}

fn run_models(provider_filter: Option<String>) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        let client = build_provider_client(true).await;
        let providers = client.list_providers();

        if providers.is_empty() {
            println!("No providers configured. Set API keys or run `fever init`.");
            return Ok(());
        }

        for name in providers {
            if let Some(ref filter) = provider_filter {
                if !name.eq_ignore_ascii_case(filter) {
                    continue;
                }
            }

            if let Some(adapter) = client.get_provider(&name) {
                let caps = adapter.capabilities();
                let models = adapter.list_models();

                if models.is_empty() {
                    println!("{} (0 models — API key may be invalid)", name);
                } else {
                    println!(
                        "{} ({} models{})",
                        name,
                        models.len(),
                        if caps.supports_chat { "" } else { ", no chat" }
                    );
                    for m in &models {
                        println!("  {}", m);
                    }
                }
                println!();
            }
        }

        Ok(())
    })
}

async fn run_run(
    prompt: Vec<String>,
    model: Option<String>,
    json_output: bool,
) -> anyhow::Result<()> {
    let content = prompt.join(" ");
    if content.trim().is_empty() {
        anyhow::bail!("No prompt provided. Usage: fever run \"your prompt\"");
    }

    let client = build_provider_client(false).await;
    let model = model.unwrap_or_else(|| {
        client
            .get_default_provider()
            .map(|p| format!("{}/gpt-4o", p))
            .unwrap_or_else(|| "openai/gpt-4o".to_string())
    });

    let req = ChatRequest {
        model: model.clone(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content,
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        temperature: None,
        max_tokens: None,
        stream: !json_output,
    };

    let start = std::time::Instant::now();

    if json_output {
        match client.chat(&req).await {
            Ok(resp) => {
                let elapsed = start.elapsed();
                println!("{}", serde_json::to_string_pretty(&resp)?);
                if let Some(usage) = &resp.usage {
                    eprintln!(
                        "  ({:.1}s, {} prompt + {} completion tokens)",
                        elapsed.as_secs_f64(),
                        usage.prompt_tokens,
                        usage.completion_tokens
                    );
                }
            }
            Err(e) => {
                tracing::error!("Run failed: {:#}", e);
                anyhow::bail!("Provider error: {:#}", e);
            }
        }
    } else {
        use futures::StreamExt;
        match client.chat_stream(&req).await {
            Ok(stream) => {
                let mut stream = stream;
                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            if let Some(text) = &chunk.content {
                                print!("{text}");
                                use std::io::Write;
                                std::io::stdout().flush().ok();
                            }
                        }
                        Err(e) => {
                            eprintln!("\nStream error: {e}");
                            break;
                        }
                    }
                }
                let elapsed = start.elapsed();
                eprintln!("\n  ({:.1}s)", elapsed.as_secs_f64());
            }
            Err(e) => {
                tracing::error!("Run failed: {:#}", e);
                anyhow::bail!("Provider error: {:#}", e);
            }
        }
    }

    Ok(())
}

fn run_session(action: &str, _id: Option<&str>) -> anyhow::Result<()> {
    let cm = fever_config::ConfigManager::new()?;
    let data_dir = cm.data_dir();
    let sessions_dir = data_dir.join("sessions");

    match action {
        "list" => {
            if !sessions_dir.exists() {
                println!("No sessions found.");
                return Ok(());
            }
            let mut sessions: Vec<_> = std::fs::read_dir(&sessions_dir)?
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.ends_with(".json") {
                        let stem = name.trim_end_matches(".json").to_string();
                        let meta = e.metadata().ok()?;
                        let modified = meta.modified().ok()?;
                        Some((stem, modified))
                    } else {
                        None
                    }
                })
                .collect();
            sessions.sort_by(|a, b| b.1.cmp(&a.1));

            if sessions.is_empty() {
                println!("No sessions found.");
            } else {
                println!("  Sessions ({}):\n", sessions.len());
                for (id, modified) in sessions {
                    let time = chrono::DateTime::<chrono::Local>::from(modified);
                    println!("  {}  {}", id, time.format("%Y-%m-%d %H:%M"));
                }
            }
        }
        "clear" => {
            if sessions_dir.exists() {
                let count = std::fs::read_dir(&sessions_dir)?
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext == "json")
                            .unwrap_or(false)
                    })
                    .count();
                if count == 0 {
                    println!("No sessions to clear.");
                } else {
                    std::fs::remove_dir_all(&sessions_dir)?;
                    std::fs::create_dir_all(&sessions_dir)?;
                    println!("Cleared {} session(s).", count);
                }
            } else {
                println!("No sessions to clear.");
            }
        }
        _ => {
            println!("Usage: fever session <list|clear>");
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .with_target(false)
        .init();
    tracing::info!("fever v{} starting", env!("CARGO_PKG_VERSION"));

    let config_manager = fever_config::ConfigManager::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize config: {e}"))?;
    let config = config_manager
        .load()
        .map_err(|e| anyhow::anyhow!("Failed to load config: {e}"))?;

    tracing::debug!(config_path = ?config_manager.config_path(), "Loaded configuration");
    if let Some(provider) = &config.defaults.provider {
        tracing::info!("Default provider from config: {provider}");
    }
    if let Some(model) = &config.defaults.model {
        tracing::info!("Default model from config: {model}");
    }

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
                stream: true,
            };
            use futures::StreamExt;
            match client.chat_stream(&req).await {
                Ok(stream) => {
                    let mut stream = stream;
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                if let Some(text) = &chunk.content {
                                    print!("{text}");
                                    use std::io::Write;
                                    std::io::stdout().flush().ok();
                                }
                            }
                            Err(e) => {
                                eprintln!("\nStream error: {e}");
                                break;
                            }
                        }
                    }
                    println!();
                }
                Err(e) => {
                    tracing::error!("Chat failed: {:#}", e);
                    eprintln!("Error: {:#}", e);
                }
            }
        }
        Some(Command::Providers { fetch, test }) => {
            if let Some(provider_name) = test {
                let client = build_provider_client(false).await;
                let provider = match client.get_provider(&provider_name) {
                    Some(p) => p,
                    None => {
                        eprintln!("Provider '{}' not found", provider_name);
                        eprintln!("Available: {}", client.list_providers().join(", "));
                        return Ok(());
                    }
                };
                println!("Testing provider: {}...", provider_name);
                match provider.validate_config().await {
                    Ok(()) => println!("  Config: valid"),
                    Err(e) => {
                        println!("  Config: INVALID - {}", e);
                        return Ok(());
                    }
                }
                let request = ChatRequest {
                    model: format!("{}/default", provider_name),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "ping".to_string(),
                        tool_calls: None,
                        tool_call_id: None,
                    }],
                    temperature: Some(0.0),
                    max_tokens: Some(5),
                    tools: None,
                    stream: false,
                };
                match provider.chat(&request).await {
                    Ok(response) => {
                        println!("  Chat: OK (id: {})", response.id);
                        if let Some(usage) = &response.usage {
                            println!(
                                "  Tokens: {} prompt + {} completion = {} total",
                                usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                            );
                        }
                    }
                    Err(e) => {
                        println!("  Chat: FAILED - {}", e);
                    }
                }
                return Ok(());
            }

            let registry = fever_providers::ProviderRegistry::builtin();
            let profiles = registry.list();

            let tier_label = |tier: &fever_providers::ProviderTier| match tier {
                fever_providers::ProviderTier::FirstClass => "first-class",
                fever_providers::ProviderTier::Compatible => "compatible",
                fever_providers::ProviderTier::Community => "community",
            };

            let mut configured_count = 0usize;
            for p in &profiles {
                let configured = p.is_configured();
                if configured {
                    configured_count += 1;
                }
                let status = if configured { "✓" } else { "○" };
                let tier = tier_label(&p.tier);
                println!(
                    "{} {:<20} {:<12} {} (default: {})",
                    status, p.id, tier, p.display_name, p.default_model
                );
            }

            println!();
            println!(
                "{} configured / {} total profiles",
                configured_count,
                profiles.len()
            );

            let client = build_provider_client(fetch).await;
            let registered = client.list_providers();
            if !registered.is_empty() {
                println!("{} live adapters: {}", registered.len(), registered.join(", "));
            }
        }
        Some(Command::Version { local, bump }) => {
            handle_version(local, bump);
        }
        Some(Command::Init) => {
            run_onboard()?;
        }
        Some(Command::Doctor) => {
            run_doctor()?;
        }
        Some(Command::Config {
            path,
            show,
            validate,
            edit,
        }) => {
            run_config(path, show, validate, edit)?;
        }
        Some(Command::Models { provider }) => {
            run_models(provider).await??;
        }
        Some(Command::Run {
            prompt,
            model,
            json,
        }) => {
            run_run(prompt, model, json).await?;
        }
        Some(Command::Session { action, id }) => {
            run_session(&action, id.as_deref())?;
        }
        None => {
            // If --mock is enabled, bootstrap a provider client with MockProvider only.
            let provider = if cli.mock {
                let mut mock_client = fever_providers::ProviderClient::new();
                let mock_provider = fever_providers::MockProvider::new();
                mock_client.register(
                    Arc::new(mock_provider),
                    mock_client.list_providers().is_empty(),
                );
                Arc::new(mock_client)
            } else {
                Arc::new(build_provider_client(false).await)
            };

            let mut app = fever_tui::AppState::new();
            if cli.mock {
                app.is_mock_mode = true;
            }
            let tools = Arc::new(build_tool_registry());

            let mut guard = fever_core::PermissionGuard::new();
            guard.grant(fever_core::PermissionScope::ShellExec);
            guard.grant(fever_core::PermissionScope::FilesystemRead);
            guard.grant(fever_core::PermissionScope::FilesystemWrite);
            guard.grant(fever_core::PermissionScope::FilesystemDelete);
            guard.grant(fever_core::PermissionScope::GitOperations);
            if let Ok(cwd) = std::env::current_dir() {
                guard.allow_path(&cwd);
            }
            let guard = Arc::new(std::sync::RwLock::new(guard));

            let default_model = provider
                .get_default_provider()
                .map(|p| format!("{}/gpt-4o", p))
                .unwrap_or_else(|| "openai/gpt-4o".to_string());

            let config = AgentConfig {
                default_model: default_model.clone(),
                ..AgentConfig::default()
            };

            let handle = FeverAgentHandle::new(provider, tools, config, guard);
            app.provider_name = handle
                .default_model()
                .split('/')
                .next()
                .unwrap_or("none")
                .to_string();
            app.model_name = handle
                .default_model()
                .split('/')
                .nth(1)
                .unwrap_or("none")
                .to_string();
            app.agent = Some(Arc::new(handle));

            app.run().await?;
        }
    }

    Ok(())
}
