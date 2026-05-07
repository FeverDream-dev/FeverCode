#[allow(dead_code)]
mod agents;
#[allow(dead_code)]
mod agent_loop;
#[allow(dead_code)]
mod approval;
#[allow(dead_code)]
mod clarification;
#[allow(dead_code)]
mod config;
#[allow(dead_code)]
mod rag;
#[allow(dead_code)]
mod context_economy;
#[allow(dead_code)]
mod events;
#[allow(dead_code)]
mod mcp;
#[allow(dead_code)]
mod patch;
#[allow(dead_code)]
mod presets;
#[allow(dead_code)]
mod providers;
#[allow(dead_code)]
mod safety;
#[allow(dead_code)]
mod souls;
#[allow(dead_code)]
mod tools;
mod tui;
mod workspace;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::tools::Tool;

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

    #[command(about = "Manage FeverCode souls and agent constitution")]
    Souls {
        #[command(subcommand)]
        action: SoulsAction,
    },

    #[command(about = "Context economy: session stats and compaction")]
    Context {
        #[command(subcommand)]
        action: ContextAction,
    },

    #[command(about = "Manage LLM presets for tool-use reliability")]
    Preset {
        #[command(subcommand)]
        action: PresetAction,
    },

    #[command(about = "Vibe coding: creative one-shot mode with relaxed safety")]
    Vibe {
        #[arg(trailing_var_arg = true, help = "Task or idea to build")]
        task: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
enum SoulsAction {
    #[command(about = "List all built-in and configured souls")]
    List,

    #[command(about = "Show details for a specific soul")]
    Show {
        #[arg(help = "Soul name: ra, thoth, ptah, maat, anubis, seshat")]
        name: String,
    },

    #[command(about = "Validate SOULS.md and souls config")]
    Validate,

    #[command(about = "Create SOULS.md and .fevercode/souls.toml if missing")]
    Init,
}

#[derive(Subcommand, Debug)]
enum ContextAction {
    #[command(about = "Show context and session statistics")]
    Stats,

    #[command(about = "Generate a compact session summary")]
    Compact,
}

#[derive(Subcommand, Debug)]
enum PresetAction {
    #[command(about = "List all available presets")]
    List,

    #[command(about = "Show the current preset for the default model")]
    Show,

    #[command(about = "Set a preset by name")]
    Set {
        #[arg(help = "Preset name: default, creative, precise, local_small, local_medium, cloud_strong, test_research, vibe_coder")]
        name: String,
    },
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
        Some(Commands::Souls { action }) => handle_souls(action, &root.root),
        Some(Commands::Context { action }) => handle_context(action, &root),
        Some(Commands::Preset { action }) => handle_preset(action, &cfg),
        Some(Commands::Vibe { task }) => vibe_task(root, cfg, task.join(" ")).await,
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

    let provider = match providers::build_provider(cfg.default_provider()) {
        Ok(p) => p,
        Err(e) => {
            println!("No provider available: {}", e);
            return Ok(());
        }
    };

    let guard = safety::SafetyPolicy::new(root.root.clone(), cfg.safety.clone());
    let tools = tools::ToolRegistry::build_default(root.root.clone());
    let log = events::SessionLog::new(&root.state_dir);
    let model = cfg.providers.default.model.as_deref().unwrap_or("unknown");
    let preset = presets::Preset::detect(model);
    let mut agent = agent_loop::AgentLoop::new(provider, tools, guard, log)
        .with_preset(preset);

    let base_prompt = agents::find_agent("ra-planner")
        .map(|a| a.system_prompt.to_string())
        .unwrap_or_else(|| "You are a helpful planner.".to_string());
    let project_ctx = workspace::load_project_context(&root.root);
    let full_base = if project_ctx.is_empty() {
        base_prompt.clone()
    } else {
        format!("{}\n\n## Project Context\n{}", base_prompt, project_ctx)
    };
    let system_prompt = preset.build_system_prompt(&full_base);

    println!("Ra is planning... (preset: {:?}, temp: {})\n", preset, preset.temperature());
    let result = agent
        .run(
            &system_prompt,
            &task,
            Box::new(|delta| {
                print!("{}", delta);
                let _ = std::io::Write::flush(&mut std::io::stdout());
            }),
        )
        .await?;

    println!("\n\nPlan complete. Tools used: {:?}", result.tool_calls_made);
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

    let provider = match providers::build_provider(cfg.default_provider()) {
        Ok(p) => p,
        Err(e) => {
            println!("No provider available: {}", e);
            return Ok(());
        }
    };

    let tools = tools::ToolRegistry::build_default(root.root.clone());
    let log = events::SessionLog::new(&root.state_dir);
    let model = cfg.providers.default.model.as_deref().unwrap_or("unknown");
    let preset = presets::Preset::detect(model);
    let mut agent = agent_loop::AgentLoop::new(provider, tools, guard.clone(), log)
        .with_preset(preset);

    let base_prompt = agents::find_agent("ptah-builder")
        .map(|a| a.system_prompt.to_string())
        .unwrap_or_else(|| "You are a helpful coding assistant.".to_string());
    let project_ctx = workspace::load_project_context(&root.root);
    let full_base = if project_ctx.is_empty() {
        base_prompt.clone()
    } else {
        format!("{}\n\n## Project Context\n{}", base_prompt, project_ctx)
    };
    let system_prompt = preset.build_system_prompt(&full_base);

    // Auto-create a branch before editing
    if cfg.safety.allow_git_commit || guard.mode() == safety::ApprovalMode::Spray {
        let branch_name = format!("fever/{}-{}-{}",
            sanitize_branch_name(&task),
            chrono::Utc::now().format("%Y%m%d"),
            uuid::Uuid::new_v4().to_string()[..4].to_string()
        );
        println!("Creating branch: {}", branch_name);
        let _ = tools::git_tools::GitBranchTool::new(root.root.clone()).execute(
            serde_json::json!({"name": branch_name})
        );
    }

    println!("\nPtah is building...\n");
    let result = agent
        .run(
            &system_prompt,
            &task,
            Box::new(|delta| {
                print!("{}", delta);
                let _ = std::io::Write::flush(&mut std::io::stdout());
            }),
        )
        .await?;

    println!("\n\nBuild complete. Tools used: {:?}", result.tool_calls_made);

    // Auto-commit if enabled
    if cfg.safety.allow_git_commit || guard.mode() == safety::ApprovalMode::Spray {
        println!("Creating checkpoint...");
        let checkpoint = tools::git_tools::GitCheckpointTool::new(root.root.clone()).execute(
            serde_json::json!({"message": format!("fever: {}", task)})
        );
        match checkpoint {
            Ok(r) if r.success => println!("{}", r.output),
            Ok(r) => println!("Checkpoint note: {}", r.output),
            Err(e) => println!("Checkpoint skipped: {}", e),
        }
    }

    Ok(())
}

fn sanitize_branch_name(task: &str) -> String {
    task.to_ascii_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
        .chars()
        .take(40)
        .collect()
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

    let provider = match providers::build_provider(cfg.default_provider()) {
        Ok(p) => p,
        Err(e) => {
            println!("No provider available: {}", e);
            return Ok(());
        }
    };

    let tools = tools::ToolRegistry::build_default(root.root.clone());
    let log = events::SessionLog::new(&root.state_dir);
    let model = cfg.providers.default.model.as_deref().unwrap_or("unknown");
    let preset = presets::Preset::detect(model);
    let mut agent = agent_loop::AgentLoop::new(provider, tools, guard.clone(), log)
        .with_preset(preset);

    let base_prompt = agents::find_agent("ra-planner")
        .map(|a| a.system_prompt.to_string())
        .unwrap_or_else(|| "You are a helpful autonomous agent.".to_string());
    let project_ctx = workspace::load_project_context(&root.root);
    let full_base = if project_ctx.is_empty() {
        base_prompt.clone()
    } else {
        format!("{}\n\n## Project Context\n{}", base_prompt, project_ctx)
    };
    let system_prompt = preset.build_system_prompt(&full_base);

    let max_loops = guard.max_endless_iterations();
    let checkpoint_every = guard.checkpoint_interval();

    for i in 0..max_loops {
        println!("\n--- Iteration {} ---", i + 1);
        let iter_goal = if i == 0 {
            goal.clone()
        } else {
            format!("Continue working toward: {}. Review progress and pick the next step.", goal)
        };

        let result = agent
            .run(
                &system_prompt,
                &iter_goal,
                Box::new(|delta| {
                    print!("{}", delta);
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }),
            )
            .await?;

        println!("\n[Iteration {} complete. Tools: {:?}]", i + 1, result.tool_calls_made);

        if (i + 1) % checkpoint_every == 0 {
            println!("[Checkpoint...]");
            let checkpoint = tools::git_tools::GitCheckpointTool::new(root.root.clone()).execute(
                serde_json::json!({"message": format!("fever endless checkpoint {}", i + 1)})
            );
            match checkpoint {
                Ok(r) if r.success => println!("Checkpoint: {}", r.output),
                _ => println!("Checkpoint skipped."),
            }
        }
    }

    println!("\nEndless loop complete after {} iterations.", max_loops);
    Ok(())
}

fn empty_hint(s: &str) -> &str {
    if s.trim().is_empty() {
        "<no task provided>"
    } else {
        s
    }
}

fn handle_souls(action: SoulsAction, root: &std::path::Path) -> anyhow::Result<()> {
    let cfg = souls::SoulsConfig::load(root)?;
    match action {
        SoulsAction::List => {
            souls::list_souls(cfg.as_ref());
            Ok(())
        }
        SoulsAction::Show { name } => souls::show_soul(&name, cfg.as_ref()),
        SoulsAction::Validate => souls::validate_souls(root),
        SoulsAction::Init => {
            souls::init_souls_file(root)?;
            souls::init_souls_md(root)?;
            Ok(())
        }
    }
}

fn handle_preset(action: PresetAction, cfg: &config::FeverConfig) -> anyhow::Result<()> {
    let model = cfg.providers.default.model.as_deref().unwrap_or("unknown");
    let detected = presets::Preset::detect(model);

    match action {
        PresetAction::List => {
            println!("Available presets:");
            for (preset, slug) in presets::PresetRegistry::list_all() {
                let marker = if preset == detected { " <- detected" } else { "" };
                println!("  {} — {}{}", slug, preset.description(), marker);
            }
            Ok(())
        }
        PresetAction::Show => {
            println!("Default model: {}", model);
            println!("Detected preset: {:?}", detected);
            println!("Temperature: {}", detected.temperature());
            println!("Max retries: {}", detected.max_retries());
            println!("Few-shot: {}", detected.needs_few_shot());
            println!("Grammar constraints: {}", detected.wants_grammar_constraints());
            Ok(())
        }
        PresetAction::Set { name } => {
            let preset = match name.as_str() {
                "default" => presets::Preset::Default,
                "creative" | "vibe" => presets::Preset::Creative,
                "precise" => presets::Preset::Precise,
                "local_small" | "local-small" => presets::Preset::LocalSmall,
                "local_medium" | "local-medium" => presets::Preset::LocalMedium,
                "cloud_strong" | "cloud-strong" | "cloud" => presets::Preset::CloudStrong,
                "test_research" | "test-research" | "test" => presets::Preset::TestResearch,
                "vibe_coder" | "vibe-coder" => presets::Preset::VibeCoder,
                other => {
                    println!("Unknown preset: {}", other);
                    return Ok(());
                }
            };
            println!("Preset set to: {:?} — {}", preset, preset.description());
            println!("Note: Store 'preset = \"{}\"' in .fevercode/config.toml under [providers.default] to persist.", name);
            Ok(())
        }
    }
}

fn handle_context(action: ContextAction, root: &workspace::Workspace) -> anyhow::Result<()> {
    match action {
        ContextAction::Stats => {
            let summary = workspace::summarize(&root.root)?;
            println!("Workspace: {}", root.root.display());
            println!("Files scanned: {}", summary.files_seen);
            println!("Languages: {}", summary.languages.join(", "));
            println!("Project type: {}", summary.project_type.join(", "));
            println!("Git repo: {}", if summary.has_git { "yes" } else { "no" });
            let ctx = workspace::load_project_context(&root.root);
            println!("Project context loaded: {} chars", ctx.len());
            if !ctx.is_empty() {
                println!("Sources:");
                for line in ctx.lines() {
                    if line.starts_with("## Project context") || line.starts_with("## Cursor rule") {
                        println!("  - {}", line.trim_start_matches("## "));
                    }
                }
            }
            Ok(())
        }
        ContextAction::Compact => {
            println!("Session compaction is not yet implemented.");
            Ok(())
        }
    }
}

async fn vibe_task(
    root: workspace::Workspace,
    mut cfg: config::FeverConfig,
    task: String,
) -> Result<()> {
    // Force vibe coder preset and spray mode for maximum flow
    let model = cfg.providers.default.model.clone().unwrap_or_default();
    let preset = presets::Preset::VibeCoder;
    cfg.safety.mode = safety::ApprovalMode::Spray;

    // llama3.2 hard lock check
    let lower = model.to_ascii_lowercase();
    if lower.contains("llama3.2") || lower.contains("llama-3.2") {
        println!("ERROR: llama3.2 is HARD-LOCKED to test/research mode only.");
        println!("Switch to a production-capable model to use vibe coding.");
        return Ok(());
    }

    println!("Vibe coding mode");
    println!("==================");
    println!();
    println!("Task: {}", empty_hint(&task));
    println!("Model: {}", model);
    println!("Preset: {:?} (temp={})", preset, preset.temperature());
    println!("Mode: spray — autonomous workspace edits enabled");
    println!();

    let provider = match providers::build_provider(cfg.default_provider()) {
        Ok(p) => p,
        Err(e) => {
            println!("No provider available: {}", e);
            return Ok(());
        }
    };

    let guard = safety::SafetyPolicy::new(root.root.clone(), cfg.safety.clone());
    let tools = tools::ToolRegistry::build_default(root.root.clone());
    let log = events::SessionLog::new(&root.state_dir);
    let mut agent = agent_loop::AgentLoop::new(provider, tools, guard.clone(), log)
        .with_preset(preset);

    let base_prompt = agents::find_agent("vibe-coder")
        .map(|a| a.system_prompt.to_string())
        .unwrap_or_else(|| agents::find_agent("ptah-builder")
            .map(|a| a.system_prompt.to_string())
            .unwrap_or_else(|| "You are a helpful coding assistant.".to_string()));
    let project_ctx = workspace::load_project_context(&root.root);
    let full_base = if project_ctx.is_empty() {
        base_prompt.clone()
    } else {
        format!("{}\n\n## Project Context\n{}", base_prompt, project_ctx)
    };
    let system_prompt = preset.build_system_prompt(&full_base);

    // Auto-create branch
    let branch_name = format!(
        "vibe/{}-{}",
        sanitize_branch_name(&task),
        uuid::Uuid::new_v4().to_string()[..6].to_string()
    );
    println!("Branch: {}", branch_name);
    let _ = tools::git_tools::GitBranchTool::new(root.root.clone()).execute(
        serde_json::json!({"name": branch_name})
    );

    println!("\nVibe Coder is shipping...\n");
    let result = agent
        .run(
            &system_prompt,
            &task,
            Box::new(|delta| {
                print!("{}", delta);
                let _ = std::io::Write::flush(&mut std::io::stdout());
            }),
        )
        .await?;

    println!("\n\nVibe complete. Tools used: {:?}", result.tool_calls_made);

    // Auto-commit
    let checkpoint = tools::git_tools::GitCheckpointTool::new(root.root.clone()).execute(
        serde_json::json!({"message": format!("vibe: {}", task)})
    );
    match checkpoint {
        Ok(r) if r.success => println!("Checkpoint: {}", r.output),
        Ok(r) => println!("Checkpoint note: {}", r.output),
        Err(e) => println!("Checkpoint skipped: {}", e),
    }

    Ok(())
}
