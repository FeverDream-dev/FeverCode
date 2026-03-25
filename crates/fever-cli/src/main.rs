use clap::{Parser, Subcommand};
use fever_tui::{FeverTui, TuiConfig};
use fever_config::ConfigManager;
use fever_providers::ProviderClient;
use fever_agent::{FeverAgent, AgentConfig};
use fever_core::ToolRegistry;
use fever_tools::{ShellTool, FilesystemTool, GrepTool, GitTool};
use fever_browser::BrowserTool;
use std::sync::Arc;
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "fever")]
#[command(about = "Fever Code - Terminal-first AI coding platform", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start interactive TUI")]
    Code,
    #[command(about = "Show version information")]
    Version,
    #[command(about = "List available roles")]
    Roles,
    #[command(about = "Show configuration")]
    Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Code) => run_tui().await,
        Some(Commands::Version) => show_version(),
        Some(Commands::Roles) => list_roles(),
        Some(Commands::Config) => show_config(),
        None => run_tui().await,
    }
}

async fn run_tui() -> anyhow::Result<()> {
    info!("Starting Fever Code TUI");

    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?;

    let mut tool_registry = ToolRegistry::new();

    tool_registry.register(Box::new(ShellTool::new()))?;
    tool_registry.register(Box::new(FilesystemTool::new()))?;
    tool_registry.register(Box::new(GrepTool::new()))?;
    tool_registry.register(Box::new(GitTool::new()))?;
    tool_registry.register(Box::new(BrowserTool::new()))?;

    let provider = Arc::new(ProviderClient::new());

    let agent_config = AgentConfig {
        default_model: config.defaults.model.clone().unwrap_or_else(|| "openai/gpt-4o".to_string()),
        default_temperature: config.defaults.temperature.unwrap_or(0.7),
        max_tokens: config.defaults.max_tokens.unwrap_or(4096),
        stream: false,
    };

    let agent = Arc::new(FeverAgent::new(provider.clone(), agent_config));

    let tui_config = TuiConfig::default();

    let mut tui = FeverTui::new(tui_config);

    tui.set_status("Fever Code v0.1.0 - Ready".to_string());

    info!("Running TUI");
    tui.run()?;

    info!("TUI exited");
    Ok(())
}

fn show_version() -> anyhow::Result<()> {
    println!("Fever Code {}", env!("CARGO_PKG_VERSION"));
    println!("A terminal-first AI coding platform");
    println!();
    println!("Built with Rust, for Linux.");
    Ok(())
}

fn list_roles() -> anyhow::Result<()> {
    use fever_agent::RoleRegistry;

    let registry = RoleRegistry::new();

    println!("Available Roles:");
    println!();

    for role in registry.list() {
        println!("  {} - {}", role.name, role.description);
        if !role.capabilities.is_empty() {
            println!("    Capabilities: {}", role.capabilities.join(", "));
        }
    }

    Ok(())
}

fn show_config() -> anyhow::Result<()> {
    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?;

    println!("Configuration:");
    println!();
    println!("Config directory: {}", config_manager.config_path().display());
    println!("Data directory: {}", config_manager.data_dir().display());
    println!("Cache directory: {}", config_manager.cache_dir().display());
    println!();
    println!("Default provider: {:?}", config.defaults.provider);
    println!("Default model: {:?}", config.defaults.model);
    println!("Temperature: {:?}", config.defaults.temperature);
    println!("Max tokens: {:?}", config.defaults.max_tokens);
    println!();
    println!("Search engine: {:?}", config.search.engine);
    println!("SearXNG URL: {:?}", config.search.searxng_url);
    println!();
    println!("Providers configured: {}", config.providers.len());

    Ok(())
}
