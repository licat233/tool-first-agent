mod config;
mod detect;
mod file_store;
mod mcp;
mod memory;
mod registry;
mod resolver;

use clap::{Parser, Subcommand};

/// tool-first: fast local runtime core for tool-first-agent.
///
/// SKILL.md = canonical execution rule source
/// Rust runtime = CLI + MCP for shared file-based tool-memory
/// tool-memory = shared runtime infrastructure resolved by TOOL_FIRST_MEMORY_HOME
#[derive(Parser)]
#[command(name = "tool-first", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Resolve the canonical tool-memory home directory.
    Memory {
        #[command(subcommand)]
        action: MemoryCommands,
    },
    /// Query the local tool registry.
    Registry {
        #[command(subcommand)]
        action: RegistryCommands,
    },
    /// Detect installed tools.
    Tools {
        #[command(subcommand)]
        action: ToolsCommands,
    },
    /// Run diagnostic checks.
    Doctor,
    /// Start the MCP stdio server.
    Mcp {
        #[command(subcommand)]
        action: Option<McpCommands>,
    },
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// Resolve and show the canonical memory home path.
    Resolve {
        #[arg(long)]
        json: bool,
    },
    /// Search retained tool-memory records.
    Recall {
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long, default_value = "10")]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    /// Persist a tool-memory record.
    Record {
        record_json: String,
        #[arg(long)]
        json: bool,
    },
    /// Check for multiple memory home candidates and conflicts.
    CheckConflicts {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum RegistryCommands {
    /// Query candidate tools from the registry.
    Query {
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum ToolsCommands {
    /// Detect which registered candidate tools are installed.
    Detect {
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        tool: Vec<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum McpCommands {
    /// Start the MCP stdio server.
    Serve,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Memory { action } => match action {
            MemoryCommands::Resolve { json } => cmd_memory_resolve(json),
            MemoryCommands::Recall {
                task,
                category,
                limit,
                json,
            } => cmd_memory_recall(task.as_deref(), category.as_deref(), limit, json),
            MemoryCommands::Record { record_json, json } => cmd_memory_record(&record_json, json),
            MemoryCommands::CheckConflicts { json } => cmd_memory_check_conflicts(json),
        },
        Commands::Registry { action } => match action {
            RegistryCommands::Query {
                category,
                task,
                json,
            } => cmd_registry_query(category.as_deref(), task.as_deref(), json),
        },
        Commands::Tools { action } => match action {
            ToolsCommands::Detect {
                category,
                tool,
                json,
            } => cmd_tools_detect(category.as_deref(), &tool, json),
        },
        Commands::Doctor => cmd_doctor(),
        Commands::Mcp { action } => match action {
            Some(McpCommands::Serve) | None => {
                eprintln!("tool-first MCP server starting on stdio...");
                mcp::run_stdio_server()
            }
        },
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

// ── CLI commands ────────────────────────────────────────────────────────

fn cmd_memory_resolve(json_output: bool) -> Result<(), String> {
    let cfg = config::load();
    let memory_home = resolver::resolve_memory_home(&cfg);
    let has_marker = resolver::has_marker(&memory_home);
    let env_home =
        std::env::var("TOOL_FIRST_MEMORY_HOME").unwrap_or_else(|_| "(not set)".to_string());

    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "memory_home": memory_home.to_string_lossy(),
                "TOOL_FIRST_MEMORY_HOME": env_home,
                "has_marker": has_marker,
            }))
            .unwrap()
        );
    } else {
        println!("memory_home:          {}", memory_home.display());
        println!("TOOL_FIRST_MEMORY_HOME: {env_home}");
        println!("has .tool-memory-home:  {has_marker}");
    }
    Ok(())
}

fn cmd_memory_recall(
    task: Option<&str>,
    category: Option<&str>,
    limit: usize,
    json_output: bool,
) -> Result<(), String> {
    let query = task.ok_or("--task is required for recall")?;
    let cfg = config::load();
    let memory_home = resolver::resolve_memory_home(&cfg);
    file_store::ensure_ready(&memory_home)?;

    let results = file_store::recall(&memory_home, query, category, limit);

    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "query": query,
                "results": results,
            }))
            .unwrap()
        );
    } else {
        if results.is_empty() {
            println!("No matching records found.");
        } else {
            for r in &results {
                let cat = r.category.as_deref().unwrap_or("?");
                let tool = r.tool.as_deref().unwrap_or("?");
                let task = r.task.as_deref().unwrap_or("");
                let status = r.status.as_deref().unwrap_or("?");
                let cmd = r
                    .command_template
                    .as_deref()
                    .or(r.command.as_deref())
                    .unwrap_or("");
                let mut line = format!("  {cat}/{tool}");
                if !task.is_empty() {
                    line.push_str(&format!(" [{task}]"));
                }
                line.push_str(&format!(" -> {status}"));
                if !cmd.is_empty() {
                    line.push_str(&format!("  $ {cmd}"));
                }
                println!("{line}");
            }
        }
    }
    Ok(())
}

fn cmd_memory_record(record_json: &str, json_output: bool) -> Result<(), String> {
    let mut record: memory::MemoryRecord =
        serde_json::from_str(record_json).map_err(|e| format!("Invalid JSON: {e}"))?;
    record.enrich();

    let cfg = config::load();
    let memory_home = resolver::resolve_memory_home(&cfg);
    file_store::ensure_ready(&memory_home)?;

    let result = file_store::retain(&memory_home, &record)?;

    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "saved": result.saved,
            }))
            .unwrap()
        );
    } else {
        println!("Saved: {}", result.saved);
    }
    Ok(())
}

fn cmd_memory_check_conflicts(json_output: bool) -> Result<(), String> {
    let candidates = resolver::detect_memory_homes();
    let conflict = candidates.iter().filter(|c| c.path.exists()).count() > 1;

    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "candidates": candidates,
                "conflict": conflict,
            }))
            .unwrap()
        );
    } else {
        println!("Memory home candidates:");
        for c in &candidates {
            let exists = if c.path.exists() {
                "exists"
            } else {
                "not found"
            };
            let canonical = if c.is_canonical { " (canonical)" } else { "" };
            let marker = if c.has_marker { " [has marker]" } else { "" };
            println!(
                "  {} - {} - {}{}{}",
                c.path.display(),
                c.source,
                exists,
                canonical,
                marker
            );
        }
        if conflict {
            println!(
                "\nWARNING: Multiple memory homes found. Set TOOL_FIRST_MEMORY_HOME to resolve."
            );
        } else {
            println!("\nNo conflicts detected.");
        }
    }
    Ok(())
}

fn cmd_registry_query(
    category: Option<&str>,
    task: Option<&str>,
    json_output: bool,
) -> Result<(), String> {
    let reg = registry::load_registry()?;
    let results = registry::query(&reg, category, task);

    if json_output {
        println!("{}", serde_json::to_string_pretty(&results).unwrap());
    } else {
        if results.is_empty() {
            println!("No candidates found.");
        } else {
            for row in &results {
                let marker = if row.is_match { "*" } else { "-" };
                println!(
                    "{} {} / {} (priority {})",
                    marker, row.category, row.tool, row.priority
                );
                if !row.handles.is_empty() {
                    println!("  handles: {}", row.handles.join("; "));
                }
                for (key, cmd) in &row.commands {
                    println!("  {key}: {cmd}");
                }
                if !row.detect_names.is_empty() {
                    println!("  detect: {}", row.detect_names.join(", "));
                }
            }
        }
    }
    Ok(())
}

fn cmd_tools_detect(
    category: Option<&str>,
    tools: &[String],
    json_output: bool,
) -> Result<(), String> {
    if category.is_none() && tools.is_empty() {
        return Err("Provide --category or --tool".to_string());
    }

    let reg = registry::load_registry()?;
    let results = detect::detect(&reg, category, tools);

    if json_output {
        println!("{}", serde_json::to_string_pretty(&results).unwrap());
    } else {
        for item in &results {
            let loc = if item.path.is_empty() {
                "-"
            } else {
                &item.path
            };
            let ver = if item.version.is_empty() {
                String::new()
            } else {
                format!(" ({})", item.version)
            };
            println!(
                "{:18} {:10} {:16} {}{}",
                item.status, item.category, item.tool, loc, ver
            );
        }
    }
    Ok(())
}

fn cmd_doctor() -> Result<(), String> {
    let cfg = config::load();
    let memory_home = resolver::resolve_memory_home(&cfg);
    let has_marker = resolver::has_marker(&memory_home);
    let conflicts = resolver::detect_memory_homes();
    let conflict_count = conflicts.iter().filter(|c| c.path.exists()).count();

    let ready = file_store::ensure_ready(&memory_home).is_ok();
    let backend_info = file_store::backend_info(&memory_home);

    let reg = registry::load_registry().unwrap_or_default();

    println!("=== tool-first doctor ===");
    println!();
    println!("memory_home:            {}", memory_home.display());
    println!(
        "TOOL_FIRST_MEMORY_HOME: {}",
        std::env::var("TOOL_FIRST_MEMORY_HOME").unwrap_or_else(|_| "(not set)".to_string())
    );
    println!(".tool-memory-home:      {has_marker}");
    println!("adapter:                file");
    println!("adapter ready:          {ready}");
    println!("registry categories:    {}", reg.len());
    println!("memory home conflicts:  {conflict_count}");
    println!();
    println!("backend info:");
    println!("{}", serde_json::to_string_pretty(&backend_info).unwrap());
    println!();
    if conflict_count > 1 {
        println!("WARNING: Multiple memory home candidates found. Run `tool-first memory check-conflicts`.");
    }
    println!("=== doctor complete ===");
    Ok(())
}
