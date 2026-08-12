use crate::commands::Project;
use crate::errors::CtxResult;
use crate::git::changed::changed_symbols;
use crate::output::{Default, Term, emit_json};

/// `ctx changed [ref]` — files and symbols changed since ref / working tree.
pub fn cmd_changed(project: &Project, since: Option<&str>, sync: bool, t: &Term) -> CtxResult<()> {
    let Some(git) = &project.git else {
        return Err(crate::errors::CtxError::Git(
            "not a git repository (or git is unavailable)".to_string(),
        ));
    };

    if sync {
        // keep graph in sync with disk before listing
        let mut db = crate::graph::database::Database::open(&project.root)?;
        crate::git::changed::sync_changed(git, &mut db, &project.config)?;
        let report = changed_symbols(git, &db, since)?;
        return render_changed(project, git, &report, t);
    }
    let report = changed_symbols(git, &project.db, since)?;
    render_changed(project, git, &report, t)
}

fn render_changed(
    project: &Project,
    git: &crate::git::GitRepo,
    report: &crate::git::changed::ChangedReport,
    t: &Term,
) -> CtxResult<()> {
    if t.is_json() {
        emit_json(&serde_json::to_value(report)?);
        return Ok(());
    }
    let _ = project;
    let _ = git;
    if report.files.is_empty() {
        println!("No changes since {}", report.since);
        return Ok(());
    }
    println!("CHANGED FILES");
    for f in &report.files {
        let status = f.status.as_str();
        let color = if status == "A" {
            Default::GREEN
        } else if status == "D" {
            Default::RED
        } else {
            Default::YELLOW
        };
        let label = match status {
            "A" => "added",
            "D" => "deleted",
            "R" => "renamed",
            _ => "modified",
        };
        println!(
            "  {} {}  {}",
            t.style(color, &format!("[{label}]")),
            f.path,
            t.style(Default::DIM, "")
        );
    }
    if !report.symbols.is_empty() {
        println!("\nCHANGED SYMBOLS");
        for s in &report.symbols {
            println!(
                "  {}  {}:{}  ({})",
                t.style(Default::BOLD, &s.name),
                s.file,
                s.line,
                s.kind
            );
        }
    } else {
        println!("\nCHANGED SYMBOLS");
        println!("  (none indexed for changed files — run `ctx init`)");
    }
    Ok(())
}
