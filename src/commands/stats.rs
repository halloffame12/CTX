//! `ctx stats`: index statistics — file/symbol/dependency counts and the
//! on-disk size of the index database.

use std::io::Write;
use std::path::Path;

use serde::Serialize;

use crate::commands::Project;
use crate::errors::CtxResult;
use crate::graph::database::Database;
use crate::output::Term;

#[derive(Debug, Clone, Serialize)]
pub struct StatsReport {
    pub root: String,
    pub files: u64,
    pub symbols: u64,
    pub dependencies: u64,
    pub db_size: u64,
}

pub fn cmd_stats(project: &Project, t: &Term) -> CtxResult<()> {
    let mut out = std::io::stdout();
    write_stats(&mut out, project, t.is_json())?;
    Ok(())
}

/// Render the stats report. Used by the CLI and integration tests.
pub fn write_stats(out: &mut dyn Write, project: &Project, json: bool) -> CtxResult<()> {
    let report = stats(&project.root, &project.db)?;

    if json {
        let v = serde_json::to_value(&report)?;
        writeln!(out, "{}", serde_json::to_string_pretty(&v)?)?;
        return Ok(());
    }

    let t = Term::new(false, false, false);
    writeln!(
        out,
        "{}",
        t.style(crate::output::Default::BOLD, "ctx stats")
    )?;
    writeln!(out)?;
    writeln!(out, "Root:")?;
    writeln!(out, "  {}", report.root)?;
    writeln!(out, "Index:")?;
    writeln!(out, "  {} {} files indexed", t.ok(""), report.files)?;
    writeln!(out, "  {} {} symbols indexed", t.ok(""), report.symbols)?;
    writeln!(
        out,
        "  {} {} dependencies indexed",
        t.ok(""),
        report.dependencies
    )?;
    writeln!(
        out,
        "  {} index.db: {} ({} bytes)",
        t.ok(""),
        human_bytes(report.db_size),
        report.db_size
    )?;
    Ok(())
}

fn stats(root: &Path, db: &Database) -> CtxResult<StatsReport> {
    let (files, symbols, deps) = db.stats()?;
    let db_size = std::fs::metadata(root.join(crate::graph::database::DB_PATH))
        .map(|m| m.len())
        .unwrap_or(0);
    Ok(StatsReport {
        root: root.display().to_string(),
        files: files as u64,
        symbols: symbols as u64,
        dependencies: deps as u64,
        db_size,
    })
}

/// JSON rendering of the stats report, used by the MCP `ctx_stats` tool.
pub fn stats_json(root: &Path, db: &Database) -> CtxResult<String> {
    let report = stats(root, db)?;
    Ok(serde_json::to_string_pretty(&serde_json::to_value(
        &report,
    )?)?)
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
