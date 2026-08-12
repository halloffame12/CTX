//! Impact analysis: walk the inverted dependency graph from a changed symbol
//! or file, collecting affected files and symbols.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::errors::CtxResult;
use crate::graph::database::{Database, SymbolRow};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImpactReport {
    pub target: String,
    /// When the target resolved to a symbol (not just a file), its name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_symbol: Option<String>,
    pub depth: u32,
    pub direct: Vec<ImpactedFile>,
    pub indirect: Vec<ImpactedFile>,
    pub tests: Vec<ImpactedFile>,
    /// Imports among the affected set that could not be mapped to a concrete
    /// file. Their ripple is unknowable statically, so they are surfaced
    /// explicitly instead of being silently treated as internal edges.
    pub unknown: Vec<UnknownDep>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UnknownDep {
    pub source: String,
    pub dependency_type: String,
    pub source_raw: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImpactedFile {
    pub path: String,
    pub distance: u32,
    pub symbols: Vec<SymbolRow>,
}

#[derive(Debug, Clone)]
pub struct Graph {
    /// file_id -> set of file_ids that depend on it
    dependents: HashMap<i64, HashSet<i64>>,
    /// file_id -> path
    paths: HashMap<i64, String>,
}

impl Graph {
    pub fn build(db: &Database) -> CtxResult<Graph> {
        let mut dependents: HashMap<i64, HashSet<i64>> = HashMap::new();
        let mut paths = HashMap::new();
        for file in db.all_files()? {
            paths.insert(file.id, file.path);
        }
        let mut stmt = db.conn().prepare(
            "SELECT source_file_id, target_file_id FROM dependencies WHERE target_file_id IS NOT NULL",
        )?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        for (source, target) in rows {
            dependents.entry(target).or_default().insert(source);
        }
        Ok(Graph { dependents, paths })
    }

    /// Files at each BFS distance from `start` (file id). Returns
    /// Vec of (file_id, distance).
    pub fn reachable(&self, start: i64, max_depth: u32) -> Vec<(i64, u32)> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut out = Vec::new();
        visited.insert(start);
        queue.push_back((start, 0u32));
        while let Some((node, dist)) = queue.pop_front() {
            if let Some(deps) = self.dependents.get(&node) {
                for &d in deps {
                    if visited.insert(d) {
                        let nd = dist + 1;
                        if nd <= max_depth {
                            out.push((d, nd));
                            queue.push_back((d, nd));
                        }
                    }
                }
            }
        }
        out
    }

    /// Total count of files reachable at any depth.
    pub fn reachable_counts(&self, start: i64, max_depth: u32) -> (Vec<(i64, u32)>, usize) {
        let all = self.reachable(start, max_depth);
        let count = all.len();
        (all, count)
    }
}

pub fn is_test_file(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    if lower.contains("__tests__/") || lower.contains("/__tests__") {
        return true;
    }
    if lower.contains("/test") || lower.contains("/tests") || lower.contains("_test") {
        return true;
    }
    lower.ends_with(".test.ts")
        || lower.ends_with(".test.tsx")
        || lower.ends_with(".test.js")
        || lower.ends_with(".test.jsx")
        || lower.ends_with(".spec.ts")
        || lower.ends_with(".spec.js")
        || lower.ends_with("_test.py")
        || lower.ends_with("_test.go")
        || lower.starts_with("test_")
}

/// Run impact analysis. `file_id` is the starting point; files are grouped by
/// BFS distance. Traversal is cycle-safe (each file is visited once) and
/// bounded by `depth`.
pub fn impact(
    db: &Database,
    target_path: &str,
    file_id: i64,
    target_symbol: Option<&str>,
    depth: u32,
) -> CtxResult<ImpactReport> {
    let graph = Graph::build(db)?;
    let reach = graph.reachable(file_id, depth);
    let mut direct = Vec::new();
    let mut indirect = Vec::new();
    let mut tests = Vec::new();

    for &(fid, dist) in &reach {
        let Some(path) = graph.paths.get(&fid) else {
            continue;
        };
        let path = path.clone();
        let symbols = db.symbols_for_file(fid).unwrap_or_default();
        let entry = ImpactReportEntry {
            path,
            distance: dist,
            symbols,
        };
        if is_test_file(&entry.path) {
            tests.push(entry);
        } else if dist == 1 {
            direct.push(entry);
        } else {
            indirect.push(entry);
        }
    }

    // Collect statically-unresolvable imports across the whole affected set
    // (target file + every reachable dependent). These edges can't be walked,
    // so their downstream impact is UNKNOWN by definition.
    let mut unknown: Vec<UnknownDep> = Vec::new();
    let mut seen_unknown: std::collections::HashSet<(i64, i64)> = std::collections::HashSet::new();
    let mut affected: Vec<i64> = vec![file_id];
    affected.extend(reach.iter().map(|(fid, _)| *fid));
    for fid in affected {
        let Some(path) = graph.paths.get(&fid) else {
            continue;
        };
        for dep in db.dependencies_of(fid)? {
            if dep.target_file_id.is_some() {
                continue;
            }
            if !seen_unknown.insert((fid, dep.id)) {
                continue;
            }
            unknown.push(UnknownDep {
                source: path.clone(),
                dependency_type: dep.dependency_type,
                source_raw: dep.source_raw,
            });
        }
    }

    Ok(ImpactReport {
        target: target_path.to_string(),
        target_symbol: target_symbol.map(str::to_string),
        depth,
        direct: direct
            .into_iter()
            .map(ImpactedFile::from)
            .collect::<Vec<_>>(),
        indirect: indirect
            .into_iter()
            .map(ImpactedFile::from)
            .collect::<Vec<_>>(),
        tests: tests.into_iter().map(ImpactedFile::from).collect(),
        unknown,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
struct ImpactReportEntry {
    path: String,
    distance: u32,
    symbols: Vec<SymbolRow>,
}

impl From<ImpactReportEntry> for ImpactedFile {
    fn from(e: ImpactReportEntry) -> Self {
        ImpactedFile {
            path: e.path,
            distance: e.distance,
            symbols: e.symbols,
        }
    }
}

/// Resolve a target string (symbol or file path) to a file record. Returns
/// `(project-relative path, file id, resolved symbol name)`.
pub fn resolve_target(
    db: &Database,
    target: &str,
) -> CtxResult<Option<(String, i64, Option<String>)>> {
    // exact file match first
    if let Some(f) = db.file_by_path(target)? {
        return Ok(Some((target.to_string(), f.id, None)));
    }
    // file path via LIKE (partial)
    for f in db.files_like(target, 10)? {
        if f.path.contains(target) {
            return Ok(Some((f.path, f.id, None)));
        }
    }
    // symbol
    for row in db.symbols_by_name(target, 10)? {
        if row.name == target
            && let Some(file) = db.file_by_id(row.file_id)?
        {
            return Ok(Some((file.path, file.id, Some(row.name))));
        }
    }
    Ok(None)
}
