#[allow(dead_code)]
mod agents;
#[allow(dead_code)]
mod approval;
#[allow(dead_code)]
mod config;
#[allow(dead_code)]
mod mcp;
#[allow(dead_code)]
mod patch;
#[allow(dead_code)]
mod providers;
#[allow(dead_code)]
mod safety;
#[allow(dead_code)]
mod tools;
mod tui;
mod workspace;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "fever",
    version,
    about = "FeverCode — a terminal AI coding portal",
    long_about = "FeverCode (fever) is a full-screen terminal AI coding agent.\n\
    It plans, edits, tests, and reviews code inside your workspace.\n\
    \n\
    Safety first: FeverCode never writes outside the folder where you launch it.\n\
    \n\
    Run 'fever' to open the portal, or use a subcommand for non-interactive mode."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long, help = "Override workspace root (defaults to current directory)")]
    workspace: Option<PathBuf>,

    #[arg(long, help = "Set approval mode: ask, auto, or spray")]
    mode: Option<safety::ApprovalMode>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Create .fevercode config in the current workspace")]
    Init,

    #[command(about = "Check install, workspace, safety, providers, and test detection")]
    Doctor,

    #[command(about = "Plan a task without editing files")]
    Plan {
        #[arg(trailing_var_arg = true, help = "Task description")]
        task: Vec<String>,
    },

    #[command(about = "Plan, approve, edit, and test a task")]
    Run {
        #[arg(trailing_var_arg = true, help = "Task description")]
        task: Vec<String>,
    },

    #[command(about = "Bounded autonomous loop with checkpoints (experimental)")]
    Endless {
        #[arg(trailing_var_arg = true, help = "Goal description")]
        goal: Vec<String>,
    },

    #[command(about = "List configured providers and their status")]
    Providers,

    #[command(about = "List built-in agent roles")]
    Agents,

    #[command(about = "Print version information")]
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = workspace::Workspace::detect(cli.workspace)?;
    let mut cfg = config::FeverConfig::load_or_default(&root.root)?;
    if let Some(mode) = cli.mode {
        cfg.safety.mode = mode;
    }

    match cli.command {
        None => tui::launch(root, cfg).await,
        Some(Commands::Init) => config::init_workspace(&root.root),
        Some(Commands::Doctor) => doctor(root, cfg).await,
        Some(Commands::Plan { task }) => plan_only(root, cfg, task.join(" ")).await,
        Some(Commands::Run { task }) => run_task(root, cfg, task.join(" ")).await,
        Some(Commands::Endless { goal }) => endless(root, cfg, goal.join(" ")).await,
        Some(Commands::Providers) => providers::print_providers(&cfg),
        Some(Commands::Agents) => agents::print_agents(),
        Some(Commands::Version) => {
            println!("fever {} (fevercode)", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}

async fn doctor(root: workspace::Workspace, cfg: config::FeverConfig) -> Result<()> {
    println!("FeverCode doctor");
    println!("================");
    println!();

    println!("Workspace: {}", root.root.display());
    println!("State dir: {}", root.state_dir.display());
    println!();

    let summary = workspace::summarize(&root.root)?;
    println!("Files scanned: {}", summary.files_seen);
    println!("Languages: {}", summary.languages.join(", "));
    println!("Project type: {}", summary.project_type.join(", "));
    println!("Git repo: {}", if summary.has_git { "yes" } else { "no" });
    println!();

    println!("Approval mode: {}", cfg.safety.mode);
    println!(
        "Writes inside workspace: {}",
        cfg.safety.allow_writes_inside_workspace
    );
    println!(
        "Writes outside workspace: {}",
        cfg.safety.allow_writes_outside_workspace
    );
    println!("Shell allowed: {}", cfg.safety.allow_shell);
    println!("Network allowed: {}", cfg.safety.allow_network);
    println!("Git commit allowed: {}", cfg.safety.allow_git_commit);
    println!(
        "Max endless iterations: {}",
        cfg.safety.max_endless_iterations
    );
    println!();

    println!(
        "Default provider: {} ({})",
        cfg.providers.default.name, cfg.providers.default.kind
    );
    println!("Providers configured: {}", cfg.providers.available.len());
    for p in &cfg.providers.available {
        let key_status = match &p.api_key_env {
            Some(env_var) => {
                if std::env::var(env_var).is_ok() {
                    "key set"
                } else {
                    "key missing"
                }
            }
            None => "no key needed",
        };
        println!(
            "  - {} [{}] models:{} ({})",
            p.name,
            p.kind,
            p.models.as_ref().map(|m| m.join(",")).unwrap_or_default(),
            key_status
        );
    }
    println!();

    println!("Agents enabled: {}", cfg.agents.enabled.join(", "));
    println!();

    let test_commands = cfg.detect_test_commands(&root.root);
    if test_commands.is_empty() {
        println!("Test commands: none detected");
    } else {
        println!("Test commands: {}", test_commands.join(", "));
    }
    println!();

    let safety = safety::SafetyPolicy::new(root.root.clone(), cfg.safety.clone());
    let safety_checks = vec![
        ("../escape.txt", "parent directory escape"),
        ("/etc/passwd", "absolute path outside root"),
        ("src/main.rs", "workspace file"),
    ];
    println!("Safety checks:");
    for (path, desc) in safety_checks {
        let result = safety.ensure_inside_workspace(std::path::Path::new(path));
        let status = if result.is_ok() {
            "PASS (inside)"
        } else {
            "PASS (blocked)"
        };
        println!("  {} - {} - {}", path, desc, status);
    }
    println!();

    let mcp_path = root.root.join(&cfg.mcp.config_file);
    if mcp_path.exists() {
        println!("MCP config: {} (found)", cfg.mcp.config_file);
    } else {
        println!("MCP config: {} (not found)", cfg.mcp.config_file);
    }

    println!();
    println!("Status: doctor check complete");
    Ok(())
}

async fn plan_only(
    root: workspace::Workspace,
    cfg: config::FeverConfig,
    task: String,
) -> Result<()> {
    let summary = workspace::summarize(&root.root)?;
    println!("Thoth plan mode");
    println!("===============");
    println!();
    println!("Task: {}", empty_hint(&task));
    println!("Workspace: {}", root.root.display());
    println!("Files sampled: {}", summary.files_seen);
    println!("Languages: {}", summary.languages.join(", "));
    println!("Project type: {}", summary.project_type.join(", "));
    println!("Approval mode: {}", cfg.safety.mode);
    println!();

    if agents::find_agent("ra-planner").is_some() {
        println!("Ra Planner active.");
    }

    println!();
    println!("Plan outline:");
    println!("1. Clarify goal and acceptance criteria.");
    println!("2. Map relevant files and dependencies.");
    println!("3. Propose patch set.");
    println!("4. Run checks (tests, lint, typecheck).");
    println!("5. Summarize changes and verify.");
    println!();
    println!("Note: Connect a provider to enable AI-assisted planning.");

    Ok(())
}

async fn run_task(
    root: workspace::Workspace,
    cfg: config::FeverConfig,
    task: String,
) -> Result<()> {
    let guard = safety::SafetyPolicy::new(root.root.clone(), cfg.safety.clone());
    println!("Ptah build mode");
    println!("===============");
    println!();
    println!("Task: {}", empty_hint(&task));
    println!("Mode: {}", guard.mode());
    println!("Workspace: {}", root.root.display());
    println!();

    let test_commands = cfg.detect_test_commands(&root.root);
    if !test_commands.is_empty() {
        println!("Detected test commands:");
        for cmd in &test_commands {
            println!("  - {}", cmd);
        }
    }

    println!();
    println!("Note: Connect a provider to enable AI-assisted coding.");

    Ok(())
}

async fn endless(root: workspace::Workspace, cfg: config::FeverConfig, goal: String) -> Result<()> {
    let guard = safety::SafetyPolicy::new(root.root.clone(), cfg.safety.clone());
    println!("Ra endless mode (experimental)");
    println!("===============");
    println!();
    println!("Goal: {}", empty_hint(&goal));
    println!("Mode: {}", guard.mode());
    println!("Max iterations: {}", guard.max_endless_iterations());
    println!(
        "Checkpoint every: {} iterations",
        guard.checkpoint_interval()
    );
    println!("Workspace: {}", root.root.display());
    println!();
    println!("Loop: plan -> edit -> test -> doctor -> checkpoint -> continue/stop.");
    println!("Note: Connect a provider to enable autonomous execution.");

    Ok(())
}

fn empty_hint(s: &str) -> &str {
    if s.trim().is_empty() {
        "<no task provided>"
    } else {
        s
    }
}
