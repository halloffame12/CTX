use crate::commands::Project;
use crate::errors::CtxResult;
use crate::graph::database::SymbolRow;
use crate::output::{Default, Term, emit_json};

pub fn cmd_search(
    project: &Project,
    query: &str,
    kind: Option<&str>,
    files_only: bool,
    limit: usize,
    t: &Term,
) -> CtxResult<()> {
    let limit = limit.clamp(1, 500);

    if files_only {
        let files = project.db.files_like(query, limit)?;
        if t.is_json() {
            let v: Vec<serde_json::Value> = files
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "path": f.path,
                        "language": f.language,
                        "size": f.size,
                    })
                })
                .collect();
            emit_json(&serde_json::Value::Array(v));
            return Ok(());
        }
        println!("Found {} files", files.len());
        for (i, f) in files.iter().enumerate() {
            let rel = f.path.clone();
            let lang = f.language.as_deref().unwrap_or("?");
            println!(
                "{}. {}  {}",
                i + 1,
                t.style(Default::CYAN, &rel),
                t.style(Default::DIM, lang)
            );
        }
        return Ok(());
    }

    let symbols = project.db.search(query, kind, limit)?;
    if t.is_json() {
        let v: Vec<serde_json::Value> = symbols.iter().map(symbol_json).collect();
        emit_json(&serde_json::Value::Array(v));
        return Ok(());
    }

    if symbols.is_empty() {
        println!("No symbols matching `{query}`");
        return Ok(());
    }
    println!("Found {} matches\n", symbols.len());
    for (i, s) in symbols.iter().enumerate() {
        let path = project
            .db
            .file_by_id(s.file_id)?
            .map(|f| f.path)
            .unwrap_or_default();
        let display_name = match &s.parent {
            Some(p) if !p.is_empty() => format!("{p}.{}", s.name),
            _ => s.name.clone(),
        };
        let kind_str = s.kind.as_str();
        println!("{}. {}", i + 1, t.style(Default::BOLD, &display_name));
        println!(
            "   {}:{}  {}",
            path,
            s.start_line,
            t.style(Default::DIM, kind_str)
        );
    }
    Ok(())
}

pub fn symbol_json(s: &SymbolRow) -> serde_json::Value {
    serde_json::json!({
        "id": s.id,
        "name": s.name,
        "kind": s.kind,
        "signature": s.signature,
        "parent": s.parent,
        "visibility": s.visibility,
        "exported": s.exported,
        "start_line": s.start_line,
        "end_line": s.end_line,
    })
}
