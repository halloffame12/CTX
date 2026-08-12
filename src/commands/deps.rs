use crate::commands::{Project, normalize_rel_path, resolve_file};
use crate::errors::CtxResult;
use crate::output::{Default, Term, emit_json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Both,
    Outgoing,
    Incoming,
}

pub fn cmd_deps(project: &Project, path: &str, direction: Direction, t: &Term) -> CtxResult<()> {
    let rel = normalize_rel_path(&project.root, path)?;
    let file = resolve_file(project, &rel)?;

    let outgoing = project.db.internal_dependencies_of(file.id)?;
    let incoming = project.db.dependents_of(file.id)?;

    if t.is_json() {
        let out: Vec<serde_json::Value> = outgoing
            .iter()
            .map(|(p, sym)| {
                serde_json::json!({
                    "target": p,
                    "imported_symbol": sym,
                })
            })
            .collect();
        let inc: Vec<serde_json::Value> = incoming
            .iter()
            .map(|(p, sym)| {
                serde_json::json!({
                    "source": p,
                    "imported_symbol": sym,
                })
            })
            .collect();
        emit_json(&serde_json::json!({
            "file": file.path,
            "depends_on": out,
            "used_by": inc,
        }));
        return Ok(());
    }

    println!("{}", t.style(Default::BOLD, &file.path));

    match direction {
        Direction::Outgoing | Direction::Both => {
            println!("\nDEPENDS ON");
            if outgoing.is_empty() {
                println!("  (no internal dependencies)");
            } else {
                let last = outgoing.len() - 1;
                for (i, (target, sym)) in outgoing.iter().enumerate() {
                    let prefix = if i == last { "└──" } else { "├──" };
                    let s = match sym {
                        Some(s) => format!("{target} (via {s})"),
                        None => target.clone(),
                    };
                    println!("  {prefix} ./{}", s);
                }
            }
        }
        _ => {}
    }
    match direction {
        Direction::Incoming | Direction::Both => {
            println!("\nUSED BY");
            if incoming.is_empty() {
                println!("  (nothing imports this file)");
            } else {
                let last = incoming.len() - 1;
                for (i, (source, sym)) in incoming.iter().enumerate() {
                    let prefix = if i == last { "└──" } else { "├──" };
                    let s = match sym {
                        Some(s) => format!("{source} (via {s})"),
                        None => source.clone(),
                    };
                    println!("  {prefix} ./{}", s);
                }
            }
            println!();
        }
        _ => println!(),
    }
    Ok(())
}
