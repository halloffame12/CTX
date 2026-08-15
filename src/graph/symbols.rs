//! Symbol-level queries: lookups, search, and related symbols.

use crate::errors::CtxResult;
use crate::graph::database::{Database, FileRecord, SymbolRow};

/// Resolve a symbol by exact name; prefer exported/class-level matches when
/// several symbols share a name. Accepts a bare name (`updateUser`) or a
/// qualified name (`UserService.updateUser`). Returns the file it lives in
/// with row.
pub fn resolve_symbol(
    db: &Database,
    name: &str,
    file_id: Option<i64>,
) -> CtxResult<Vec<LocatedSymbol>> {
    let mut out = Vec::new();

    // Qualified lookup: `Parent.member`. Prefer parent+name matches so the
    // display name (Parent.member) used by `ctx search` round-trips.
    if let Some((parent, member)) = name.split_once('.') {
        for row in db.symbols_by_parent_and_name(parent, member, 200)? {
            let Some(file) = db.file_by_id(row.file_id)? else {
                continue;
            };
            if let Some(fid) = file_id
                && fid != row.file_id
            {
                continue;
            }
            out.push(LocatedSymbol { symbol: row, file });
        }
        if !out.is_empty() {
            return Ok(out);
        }
    }

    let rows = db.symbols_by_name(name, 200)?;
    for row in rows {
        let Some(file) = db.file_by_id(row.file_id)? else {
            continue;
        };
        if let Some(fid) = file_id
            && fid != row.file_id
        {
            continue;
        }
        out.push(LocatedSymbol { symbol: row, file });
    }
    Ok(out)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LocatedSymbol {
    pub symbol: SymbolRow,
    pub file: FileRecord,
}

/// Methods on a symbol's own file.
pub fn methods_of(db: &Database, class_file: i64, class_name: &str) -> CtxResult<Vec<SymbolRow>> {
    use crate::graph::database::SymbolRow;
    let mut stmt = db.conn().prepare(
        "SELECT id, file_id, name, kind, signature, parent, visibility, exported,
                start_line, end_line, start_byte, end_byte
         FROM symbols
         WHERE file_id = ?1 AND parent = ?2 AND kind IN ('method','field')
         ORDER BY start_line",
    )?;
    let rows = stmt
        .query_map(rusqlite::params![class_file, class_name], |r| {
            Ok(SymbolRow {
                id: r.get(0)?,
                file_id: r.get(1)?,
                name: r.get(2)?,
                kind: r.get(3)?,
                signature: r.get(4)?,
                parent: r.get(5)?,
                visibility: r.get(6)?,
                exported: r.get::<_, i64>(7)? != 0,
                start_line: r.get::<_, i64>(8)? as u32,
                end_line: r.get::<_, i64>(9)? as u32,
                start_byte: r.get(10)?,
                end_byte: r.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub struct SymbolDetail {
    pub symbol: SymbolRow,
    pub file: FileRecord,
    pub methods: Vec<SymbolRow>,
    pub references: Vec<(String, Option<String>)>,
    pub dependencies: Vec<(String, Option<String>)>,
}

pub fn symbol_detail(db: &Database, name: &str) -> CtxResult<Vec<SymbolDetail>> {
    let mut out = Vec::new();
    for located in resolve_symbol(db, name, None)? {
        // For a class/struct, `parent` is its containing scope and methods are
        // keyed on the symbol's own name. For a member lookup such as
        // "UserService.updateUser", the resolved symbol is the method and the
        // enclosing class name is `parent`. Never pass the full qualified query
        // string here — it cannot match `parent` in the methods query.
        let class_name = located
            .symbol
            .parent
            .as_deref()
            .filter(|p| !p.is_empty())
            .unwrap_or(&located.symbol.name);
        let methods = methods_of(db, located.file.id, class_name)?;
        let references = db.dependents_of(located.file.id)?;
        let dependencies = db.internal_dependencies_of(located.file.id)?;
        out.push(SymbolDetail {
            symbol: located.symbol,
            file: located.file,
            methods,
            references,
            dependencies,
        });
    }
    Ok(out)
}
