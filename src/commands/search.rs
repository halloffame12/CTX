use crate::commands::Project;
use crate::errors::{CtxError, CtxResult};
use crate::graph::database::SymbolRow;
use crate::output::{Default, Term, emit_json};
use crate::parser::SymbolKind;

pub fn cmd_search(
    project: &Project,
    query: &str,
    kind: Option<&str>,
    files_only: bool,
    limit: usize,
    t: &Term,
) -> CtxResult<()> {
    let limit = limit.clamp(1, 500);

    let kind = normalize_kind(kind)?;

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
        let v: Vec<serde_json::Value> = symbols
            .iter()
            .map(|s| {
                symbol_json(
                    s,
                    project
                        .db
                        .file_by_id(s.file_id)
                        .ok()
                        .flatten()
                        .map(|f| f.path),
                )
            })
            .collect();
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

pub fn symbol_json(s: &SymbolRow, path: Option<String>) -> serde_json::Value {
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
        "file": path,
        "path": path,
    })
}

fn normalize_kind(kind: Option<&str>) -> CtxResult<Option<&'static str>> {
    match kind {
        Some(k) => match SymbolKind::from_str(k) {
            Some(sk) => Ok(Some(sk.as_str())),
            None => Err(CtxError::Usage(format!(
                "invalid symbol kind `{k}`; expected one of: {}",
                SymbolKind::ALL_NAMES.join(", ")
            ))),
        },
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_aliases_normalize_to_canonical_names() {
        assert_eq!(normalize_kind(Some("fn")).unwrap(), Some("function"));
        assert_eq!(normalize_kind(Some("const")).unwrap(), Some("constant"));
        assert_eq!(normalize_kind(Some("alias")).unwrap(), Some("type"));
        assert_eq!(normalize_kind(None).unwrap(), None);
    }

    #[test]
    fn invalid_kind_is_rejected() {
        let err = normalize_kind(Some("notakind")).unwrap_err();
        assert!(matches!(err, CtxError::Usage(_)), "got {err:?}");
    }
}
