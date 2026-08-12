use crate::commands::{Project, normalize_rel_path};
use crate::errors::CtxResult;
use crate::graph::impact::{ImpactReport, impact, resolve_target};
use crate::output::{Default, Term, emit_json};

pub fn cmd_impact(project: &Project, target: &str, depth: u32, t: &Term) -> CtxResult<()> {
    let (path, file_id, target_symbol) = {
        if let Some(found) = resolve_target(&project.db, target)? {
            (found.0, found.1, found.2)
        } else {
            // maybe the user gave a file path that isn't in the graph yet
            let rel = normalize_rel_path(&project.root, target)?;
            if let Some(f) = project.db.file_by_path(&rel)? {
                (f.path, f.id, None)
            } else {
                return Err(crate::errors::CtxError::Other(format!(
                    "target `{target}` not found in the graph (it's neither a symbol nor an indexed file)"
                )));
            }
        }
    };

    let report = impact(&project.db, &path, file_id, target_symbol.as_deref(), depth)?;

    if t.is_json() {
        emit_json(&serde_json::to_value(&report)?);
        return Ok(());
    }

    let total_files = report.direct.len() + report.indirect.len() + report.tests.len();
    let total_symbols = report
        .direct
        .iter()
        .chain(report.indirect.iter())
        .chain(report.tests.iter())
        .map(|f| f.symbols.len())
        .sum::<usize>();

    println!("IMPACT ANALYSIS");
    println!();
    println!("Changed:");
    println!("  {}", t.style(Default::BOLD, &report.target));
    if let Some(sym) = &report.target_symbol {
        println!("  symbol: {sym}");
    }
    println!();

    if !report.direct.is_empty() {
        println!("Direct dependents:");
        for f in &report.direct {
            println!("  {}", f.path);
        }
        println!();
    }
    if !report.indirect.is_empty() {
        println!("Indirect dependents:");
        for f in &report.indirect {
            println!("  {} (depth {})", f.path, f.distance);
        }
        println!();
    }
    if !report.tests.is_empty() {
        println!("Tests:");
        for f in &report.tests {
            println!("  {}", f.path);
        }
        println!();
    }

    println!("Potentially affected:");
    println!("  {total_files} files");
    println!("  {total_symbols} symbols");
    println!("  (depth {})", report.depth);
    println!();

    if !report.unknown.is_empty() {
        println!("UNKNOWN (unresolvable imports in affected files):");
        for u in &report.unknown {
            println!("  {} → {}", u.source, u.source_raw);
        }
        println!();
    }

    if report.direct.is_empty() && report.indirect.is_empty() && report.tests.is_empty() {
        println!("No dependents found in the graph.");
    }
    let _ = Default::DIM;
    Ok(())
}

pub fn report_stats(report: &ImpactReport) -> (usize, usize) {
    let files = report.direct.len() + report.indirect.len() + report.tests.len();
    let symbols = report
        .direct
        .iter()
        .chain(report.indirect.iter())
        .chain(report.tests.iter())
        .map(|f| f.symbols.len())
        .sum();
    (files, symbols)
}
