use std::time::Instant;

use crate::config::Config;
use crate::errors::CtxResult;
use crate::indexing::incremental::run_index;
use crate::output::{Default, Term, emit_json};

pub fn cmd_init(
    cwd: &std::path::Path,
    force: bool,
    root_override: Option<&std::path::Path>,
    t: &Term,
) -> CtxResult<()> {
    let start = Instant::now();
    let root = match root_override {
        Some(r) => r.to_path_buf(),
        None => cwd.to_path_buf(),
    };
    let config = if force {
        let mut c = Config::default();
        c.index.exclude = Config::default().index.exclude.clone();
        c
    } else {
        Config::load(&root)?
    };

    if t.is_json() {
        // ensure .ctx dir + config
        std::fs::create_dir_all(root.join(".ctx"))?;
        write_default_config(&root);
        let report = run_index(&root, &config)?;
        emit_json(&serde_json::to_value(report)?);
        return Ok(());
    }

    let was_initialized = crate::graph::database::Database::exists(&root);
    t.p(&t.style(Default::BOLD, "ctx initializing project..."));
    t.p("");
    std::fs::create_dir_all(root.join(".ctx"))?;
    write_default_config(&root);

    let probe = run_index(&root, &config)?;

    t.p(&format!(
        "{} Project detected",
        t.ok(&root.display().to_string())
    ));
    t.p(&format!(
        "{} {} files discovered",
        t.ok(""),
        probe.total_files
    ));
    t.p(&format!(
        "{} {} supported files",
        t.ok(""),
        probe.supported_files
    ));
    t.p(&format!("{} {} parsed", t.ok(""), probe.parsed_files));
    if probe.unchanged_files > 0 {
        t.p(&format!(
            "{} {} unchanged (incremental)",
            t.style(Default::DIM, "·"),
            probe.unchanged_files
        ));
    }
    if probe.metadata_only > 0 {
        t.p(&format!(
            "{} {} metadata refreshed",
            t.style(Default::DIM, "·"),
            probe.metadata_only
        ));
    }
    if probe.deleted_files > 0 {
        t.p(&format!(
            "{} {} files removed from graph",
            t.style(Default::YELLOW, "·"),
            probe.deleted_files
        ));
    }
    if probe.skipped > 0 {
        t.p(&format!(
            "{} {} skipped ({})",
            t.style(Default::YELLOW, "·"),
            probe.skipped,
            probe.skipped_reason.as_deref().unwrap_or("")
        ));
    }
    t.p(&format!(
        "{} {} symbols indexed",
        t.ok(""),
        probe.symbols_indexed
    ));
    t.p(&format!(
        "{} {} dependencies discovered",
        t.ok(""),
        probe.dependencies_indexed
    ));
    t.p(&format!("{} SQLite graph ready", t.ok("")));

    for e in &probe.parse_errors {
        t.p(&format!(
            "{} parse issues in {}",
            t.style(Default::YELLOW, "⚠"),
            e
        ));
    }

    t.p("");
    if was_initialized {
        t.p(&t.style(Default::GREEN, "Index updated."));
    } else {
        t.p(&t.style(Default::GREEN, "Index ready."));
    }
    t.p(&format!("Time: {}ms", probe.elapsed_ms));
    let _ = start;
    Ok(())
}

fn write_default_config(root: &std::path::Path) {
    let path = root.join(".ctx").join("config.toml");
    if !path.exists() {
        let _ = std::fs::write(
            &path,
            r#"# ctx configuration
[index]
exclude = [
    "node_modules",
    "target",
    ".git",
    ".ctx",
    "dist",
    "build",
    "vendor",
    "coverage",
    "__pycache__",
    ".next",
]

[context]
max_tokens = 12000
max_files = 25
include_bodies = false

[watch]
enabled = true
debounce_ms = 200
"#,
        );
    }
}
