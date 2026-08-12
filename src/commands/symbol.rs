use crate::commands::Project;
use crate::errors::CtxResult;
use crate::graph::symbols::symbol_detail;
use crate::output::{Default, Term, emit_json};

pub fn cmd_symbol(project: &Project, name: &str, t: &Term) -> CtxResult<()> {
    let details = symbol_detail(&project.db, name)?;

    if t.is_json() {
        let v: Vec<serde_json::Value> = details
            .iter()
            .map(|d| {
                serde_json::json!({
                    "name": d.symbol.name,
                    "kind": d.symbol.kind,
                    "signature": d.symbol.signature,
                    "file": d.file.path,
                    "line": d.symbol.start_line,
                    "methods": d.methods,
                    "references": d.references,
                    "dependencies": d.dependencies,
                })
            })
            .collect();
        emit_json(&serde_json::Value::Array(v));
        return Ok(());
    }

    if details.is_empty() {
        println!("No symbol named `{name}` in the graph");
        return Ok(());
    }

    for d in details {
        println!("{}", t.style(Default::BOLD, &d.symbol.name));
        println!("\nDefined:");
        println!("  {}:{}", d.file.path, d.symbol.start_line);
        println!("\nKind:");
        println!("  {}", d.symbol.kind);
        if let Some(sig) = &d.symbol.signature {
            println!("\nSignature:");
            println!("  {sig}");
        }
        if let Some(vis) = &d.symbol.visibility {
            println!("\nVisibility:");
            println!("  {vis}");
        }
        if !d.methods.is_empty() {
            println!("\nMethods:");
            for m in &d.methods {
                println!("  {}", m.name);
            }
        }
        if !d.references.is_empty() {
            println!("\nReferences:");
            for (path, _sym) in &d.references {
                println!("  {path}");
            }
        }
        if !d.dependencies.is_empty() {
            println!("\nDependencies:");
            for (path, _sym) in &d.dependencies {
                println!("  {path}");
            }
        }
        println!();
    }
    Ok(())
}
