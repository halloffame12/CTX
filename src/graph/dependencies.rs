//! Dependency queries: what a file imports (outgoing) and what imports it
//! (incoming).

use crate::errors::CtxResult;
use crate::graph::database::Database;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Dependency {
    /// Project-relative file path of the dependency (when internal).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imported_symbol: Option<String>,
    pub dependency_type: String,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: i64,
    pub to: i64,
}

/// Direct outgoing internal dependencies of a file.
pub fn outgoing(db: &Database, file_id: i64) -> CtxResult<Vec<Dependency>> {
    let mut out = Vec::new();
    for dep in db.dependencies_of(file_id)? {
        let path = if let Some(tid) = dep.target_file_id {
            Some(db.file_by_id(tid)?.map(|f| f.path).unwrap_or_default())
        } else {
            None
        };
        out.push(Dependency {
            path: path.filter(|p| !p.is_empty()),
            imported_symbol: dep.imported_symbol,
            dependency_type: dep.dependency_type,
        });
    }
    Ok(out)
}

/// Direct incoming dependents (files that reference this file).
pub fn incoming(db: &Database, file_id: i64) -> CtxResult<Vec<Dependency>> {
    let mut out = Vec::new();
    for (path, imported_symbol) in db.dependents_of(file_id)? {
        out.push(Dependency {
            path: Some(path),
            imported_symbol,
            dependency_type: "import".to_string(),
        });
    }
    Ok(out)
}

/// All dependency edges in the graph (source -> target file ids).
pub fn all_edges(db: &Database) -> CtxResult<Vec<Edge>> {
    let mut stmt = db.conn().prepare(
        "SELECT source_file_id, target_file_id FROM dependencies WHERE target_file_id IS NOT NULL",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Edge {
                from: r.get(0)?,
                to: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
