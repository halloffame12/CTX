use crate::commands::Project;
use crate::errors::CtxResult;
use crate::git::diff::symbol_diff;
use crate::graph::impact::resolve_target;
use crate::output::{Default, Term, emit_json};
use std::collections::BTreeSet;

pub fn cmd_diff(
    project: &Project,
    base: Option<&str>,
    head: Option<&str>,
    t: &Term,
) -> CtxResult<()> {
    let Some(git) = &project.git else {
        return Err(crate::errors::CtxError::Git(
            "not a git repository".to_string(),
        ));
    };
    let diff = symbol_diff(git, base, head, Some(&project.root))?;

    if t.is_json() {
        emit_json(&serde_json::to_value(&diff)?);
        return Ok(());
    }

    println!("CHANGED SYMBOLS ({} → {})", diff.base, diff.head);
    println!();

    for f in &diff.files {
        if f.symbols.is_empty() {
            println!(
                "  {}  {} (no symbol changes)",
                t.style(Default::DIM, f.status.as_str()),
                f.path
            );
            continue;
        }
        println!("{}", t.style(Default::BOLD, &f.path));
        let mut added: Vec<_> = f.symbols.iter().filter(|s| s.status == "Added").collect();
        let mut modified: Vec<_> = f
            .symbols
            .iter()
            .filter(|s| s.status == "Modified")
            .collect();
        let mut removed: Vec<_> = f.symbols.iter().filter(|s| s.status == "Removed").collect();
        added.sort_by_key(|s| s.name.clone());
        modified.sort_by_key(|s| s.name.clone());
        removed.sort_by_key(|s| s.name.clone());
        if !added.is_empty() {
            println!("  {}", t.style(Default::GREEN, "Added:"));
            for s in added {
                println!("    {}", s.name);
            }
        }
        if !modified.is_empty() {
            println!("  {}", t.style(Default::YELLOW, "Modified:"));
            for s in modified {
                println!("    {}", s.name);
            }
        }
        if !removed.is_empty() {
            println!("  {}", t.style(Default::RED, "Removed:"));
            for s in removed {
                println!("    {}", s.name);
            }
        }
    }

    // potential impact
    let mut impacted: BTreeSet<String> = BTreeSet::new();
    for f in &diff.files {
        if (f.status == "A" || f.status == "D" || !f.symbols.is_empty())
            && let Some((path, id, _symbol)) = resolve_target(&project.db, &f.path)?
        {
            let _ = path;
            for (dep_path, _sym) in project.db.dependents_of(id)? {
                impacted.insert(dep_path);
            }
        }
    }
    println!();
    if impacted.is_empty() {
        println!("Potential impact: none detected");
    } else {
        println!("Potential impact ({})", impacted.len());
        for p in impacted.iter().take(20) {
            println!("  {p}");
        }
        if impacted.len() > 20 {
            println!("  … and {} more", impacted.len() - 20);
        }
    }
    Ok(())
}
