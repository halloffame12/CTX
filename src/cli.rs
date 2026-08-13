use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::commands::{self, Project};
use crate::errors::CtxResult;
use crate::output::{Default, Term};

#[derive(Parser)]
#[command(
    name = "ctx",
    version,
    about = "Codebase intelligence and context engine for AI coding agents",
    after_help = "Run `ctx` with no arguments for a quick overview, or `ctx <command> --help` for details."
)]
pub struct Cli {
    /// Project root (defaults to the nearest directory containing .ctx)
    #[arg(short = 'R', long, global = true, value_name = "DIR")]
    root: Option<PathBuf>,

    /// Emit machine-readable JSON instead of human text
    #[arg(short = 'j', long, global = true)]
    json: bool,

    /// Suppress non-essential output
    #[arg(short = 'q', long, global = true)]
    quiet: bool,

    /// Enable verbose diagnostics on stderr
    #[arg(short = 'v', long, global = true)]
    verbose: bool,

    /// Disable ANSI colors
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Create .ctx, write a default config and index the project
    Init {
        /// Directory to initialize (default: current directory)
        path: Option<PathBuf>,
        /// Recreate the index and config even if they exist
        #[arg(long)]
        force: bool,
    },

    /// Show a body-less structural skeleton of a source file
    Skeleton {
        /// Path to the file (project-relative or absolute)
        path: String,
        /// Include sizes and symbol counts
        #[arg(long)]
        stats: bool,
    },

    /// Search the graph for symbols or files
    Search {
        /// Case-insensitive name query
        query: String,
        /// Restrict to a symbol kind
        #[arg(long)]
        kind: Option<String>,
        /// Search file paths instead of symbols
        #[arg(long)]
        files: bool,
        /// Maximum number of results
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },

    /// Details about a symbol: definition, references, dependencies
    Symbol { name: String },

    /// Show what a file imports and what imports it
    Deps {
        /// Path to the file
        path: String,
        /// Only show outgoing dependencies
        #[arg(long)]
        outgoing: bool,
        /// Only show incoming dependents
        #[arg(long)]
        incoming: bool,
    },

    /// Analyze impact of changing a symbol or file
    Impact {
        /// Symbol name or file path to change
        target: String,
        /// How deep to traverse dependent graphs
        #[arg(long, default_value_t = 3)]
        depth: u32,
    },

    /// Build a relevance-ranked context package for a task
    Context {
        /// Natural-language description of the task
        task: String,
        /// Include full function/type bodies in the suggested context
        #[arg(long)]
        include_bodies: bool,
        /// Token budget for the suggested context (overrides config)
        #[arg(long = "max-tokens", value_name = "N")]
        max_tokens: Option<usize>,
        /// Ignore working-tree git changes when ranking files
        #[arg(long)]
        no_git: bool,
    },

    /// Show symbols changed in the working tree or between refs
    Changed {
        /// Git ref to diff against (default: working tree vs HEAD)
        #[arg(long = "ref", value_name = "REF")]
        since: Option<String>,
        /// Update the graph with current files before comparing
        #[arg(long)]
        sync: bool,
    },

    /// Semantic diff of symbols between two git refs
    Diff {
        /// Base ref (default: git merge-base with HEAD)
        base: Option<String>,
        /// Head ref (default: HEAD)
        head: Option<String>,
    },

    /// Print the SQLite graph schema
    Schema,

    /// Re-run an index pass and print incremental timing
    Benchmark,

    /// Watch the project and keep the graph in sync
    Watch,

    /// Run the Model Context Protocol server over stdio
    Mcp,

    /// Inspect the project and report the health of the ctx index
    Doctor,

    /// Print version information
    Version,
}

pub fn run() -> CtxResult<()> {
    let cli = Cli::parse();
    let t = Term::new(cli.json, cli.no_color, cli.quiet);

    if cli.verbose {
        init_tracing(true);
    }

    let cwd = std::env::current_dir()?;

    let Some(command) = cli.command else {
        welcome();
        return Ok(());
    };

    match command {
        Command::Init { path, force } => {
            let root = cli.root.or(path);
            commands::init::cmd_init(&cwd, force, root.as_deref(), &t)?;
        }
        Command::Skeleton { path, stats } => {
            let root = resolve_root(&cwd, cli.root.as_deref(), true)?;
            commands::skeleton::cmd_skeleton(&root, &path, stats, &t)?;
        }
        Command::Search {
            query,
            kind,
            files,
            limit,
        } => {
            let project = open(&cwd, cli.root.as_deref(), true)?;
            commands::search::cmd_search(&project, &query, kind.as_deref(), files, limit, &t)?;
        }
        Command::Symbol { name } => {
            let project = open(&cwd, cli.root.as_deref(), true)?;
            commands::symbol::cmd_symbol(&project, &name, &t)?;
        }
        Command::Deps {
            path,
            outgoing,
            incoming,
        } => {
            let project = open(&cwd, cli.root.as_deref(), true)?;
            let direction = match (outgoing, incoming) {
                (true, false) => commands::deps::Direction::Outgoing,
                (false, true) => commands::deps::Direction::Incoming,
                _ => commands::deps::Direction::Both,
            };
            commands::deps::cmd_deps(&project, &path, direction, &t)?;
        }
        Command::Impact { target, depth } => {
            let project = open(&cwd, cli.root.as_deref(), true)?;
            commands::impact::cmd_impact(&project, &target, depth.clamp(1, 20), &t)?;
        }
        Command::Context {
            task,
            include_bodies,
            max_tokens,
            no_git,
        } => {
            let project = open(&cwd, cli.root.as_deref(), true)?;
            commands::context::cmd_context(
                &project,
                &task,
                include_bodies,
                max_tokens,
                no_git,
                &t,
            )?;
        }
        Command::Changed { since, sync } => {
            let project = open(&cwd, cli.root.as_deref(), true)?;
            commands::changed::cmd_changed(&project, since.as_deref(), sync, &t)?;
        }
        Command::Diff { base, head } => {
            let project = open(&cwd, cli.root.as_deref(), true)?;
            commands::diff::cmd_diff(&project, base.as_deref(), head.as_deref(), &t)?;
        }
        Command::Schema => {
            let project = open(&cwd, cli.root.as_deref(), true)?;
            let schema = project.db.schema_sql()?;
            if t.is_json() {
                crate::output::emit_json(&serde_json::json!({ "schema": schema }));
            } else {
                println!("{schema}");
            }
        }
        Command::Benchmark => {
            let project = open(&cwd, cli.root.as_deref(), true)?;
            commands::benchmark::cmd_benchmark(&project, &t)?;
        }
        Command::Watch => {
            let project = Project::open(&cwd, cli.root.as_deref())?;
            commands::watch::cmd_watch(&project, &t)?;
        }
        Command::Mcp => {
            let project = Project::open(&cwd, cli.root.as_deref()).or_else(|_| {
                // MCP can run without an initialized project; use cwd as root fallback
                let root = cli.root.clone().unwrap_or(cwd);
                Project::open(&root, Some(&root))
            })?;
            commands::mcp::cmd_mcp(&project, cli.root.as_deref())?;
        }
        Command::Doctor => {
            commands::doctor::cmd_doctor(&cwd, cli.root.as_deref(), &t)?;
        }
        Command::Version => {
            let v = concat!("ctx ", env!("CARGO_PKG_VERSION"));
            if t.is_json() {
                crate::output::emit_json(
                    &serde_json::json!({ "name": "ctx", "version": env!("CARGO_PKG_VERSION") }),
                );
            } else {
                println!("{v}");
            }
        }
    }
    Ok(())
}

fn welcome() {
    println!("ctx — Context Intelligence Engine");
    println!();
    println!("Understand your codebase.");
    println!("Give AI coding agents the context they actually need.");
    println!();
    println!("Usage:");
    println!("  ctx init");
    println!("  ctx context");
    println!("  ctx search");
    println!("  ctx symbol");
    println!("  ctx impact");
    println!("  ctx mcp");
    println!();
    println!("Run:");
    println!("  ctx init");
    println!();
    println!("See `ctx --help` for all commands.");
}

fn init_tracing(verbose: bool) {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(if verbose {
            tracing_subscriber::EnvFilter::new("ctx=trace")
        } else {
            tracing_subscriber::EnvFilter::new("ctx=info")
        })
        .with_writer(std::io::stderr)
        .try_init();
}

/// Resolve the project root for commands that only need a path (skeleton).
fn resolve_root(
    cwd: &std::path::Path,
    override_root: Option<&std::path::Path>,
    require_init: bool,
) -> CtxResult<std::path::PathBuf> {
    let root = match override_root {
        Some(r) => r.to_path_buf(),
        None => commands::discover_root(cwd)?,
    };
    if require_init && !crate::graph::database::Database::exists(&root) {
        return Err(crate::errors::CtxError::NotInitialized(
            root.display().to_string(),
        ));
    }
    Ok(root)
}

fn open(
    cwd: &std::path::Path,
    override_root: Option<&std::path::Path>,
    _git: bool,
) -> CtxResult<Project> {
    let project = Project::open(cwd, override_root)?;
    project.require_initialized()?;
    Ok(project)
}

// re-exported color codes used by some commands for consistent styling
#[allow(unused)]
fn _colors() -> Vec<u8> {
    vec![
        Default::BOLD,
        Default::DIM,
        Default::UNDERLINE,
        Default::RED,
        Default::GREEN,
        Default::YELLOW,
        Default::BLUE,
        Default::MAGENTA,
        Default::CYAN,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn version_parses_as_subcommand() {
        let cli = Cli::try_parse_from(["ctx", "version"]).expect("parse");
        assert!(matches!(cli.command, Some(Command::Version)));
    }

    #[test]
    fn version_flag_and_subcommand_agree() {
        // clap's `version` flag derives from the same Cargo package version.
        let v = clap::Command::new("ctx")
            .version(env!("CARGO_PKG_VERSION"))
            .render_version();
        assert!(
            v.trim_end().ends_with(env!("CARGO_PKG_VERSION")),
            "clap flag version {v} does not end with package version"
        );
    }

    #[test]
    fn no_command_is_ok() {
        let cli = Cli::try_parse_from(["ctx"]).expect("parse with no subcommand");
        assert!(cli.command.is_none());
    }

    #[test]
    fn all_expected_subcommands_exist() {
        let cmd = Cli::command();
        for name in [
            "init",
            "skeleton",
            "search",
            "symbol",
            "deps",
            "impact",
            "context",
            "changed",
            "diff",
            "schema",
            "benchmark",
            "watch",
            "mcp",
            "doctor",
            "version",
        ] {
            assert!(
                cmd.get_subcommands().any(|c| c.get_name() == name),
                "missing subcommand: {name}"
            );
        }
    }
}
