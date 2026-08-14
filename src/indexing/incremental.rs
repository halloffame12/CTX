//! Incremental indexing: only re-parse files whose hash changed, remove
//! deleted files, batch writes in a single transaction. Safe to interrupt.

use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use rayon::prelude::*;

use crate::config::Config;
use crate::errors::CtxResult;
use crate::graph::database::{Database, FileRecord};
use crate::indexing::hasher::hash_bytes;
use crate::indexing::scanner::{DiscoveredFile, scan};
use crate::lang::LanguageId;
use crate::parser::{ParsedFile, parse_source};

pub const OVERSIZED_SKIPPED: &str = "file exceeds configured max_file_size; skipped";

#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexReport {
    pub total_files: usize,
    pub supported_files: usize,
    pub parsed_files: usize,
    pub unchanged_files: usize,
    pub metadata_only: usize,
    pub deleted_files: usize,
    pub skipped: usize,
    pub skipped_reason: Option<String>,
    pub symbols_indexed: usize,
    pub dependencies_indexed: usize,
    pub parse_errors: Vec<String>,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedOutcome {
    pub rel_path: String,
    pub language: LanguageId,
    pub parsed: ParsedFile,
}

pub fn run_index(root: &Path, config: &Config) -> CtxResult<IndexReport> {
    run_index_inner(root, config, false)
}

pub fn force_reindex(root: &Path, config: &Config) -> CtxResult<IndexReport> {
    run_index_inner(root, config, true)
}

fn run_index_inner(root: &Path, config: &Config, force: bool) -> CtxResult<IndexReport> {
    let start = Instant::now();
    let mut db = Database::open(root)?;
    if force {
        // wipe all indexed state so every file is re-parsed from scratch
        db.wipe()?;
    }
    let mut report = IndexReport {
        total_files: 0,
        supported_files: 0,
        parsed_files: 0,
        unchanged_files: 0,
        metadata_only: 0,
        deleted_files: 0,
        skipped: 0,
        skipped_reason: None,
        symbols_indexed: 0,
        dependencies_indexed: 0,
        parse_errors: Vec::new(),
        elapsed_ms: 0,
    };

    let discovered = scan(root, config)?;
    report.total_files = discovered.len();

    let existing = db.all_files()?;
    let existing_by_path: HashMap<String, FileRecord> =
        existing.into_iter().map(|f| (f.path.clone(), f)).collect();

    let discovered_set: std::collections::HashSet<&str> =
        discovered.iter().map(|d| d.rel_path.as_str()).collect();

    // Classify
    let mut to_parse: Vec<(&DiscoveredFile, Option<&FileRecord>)> = Vec::new();
    let mut unchanged: Vec<&FileRecord> = Vec::new();
    let mut metadata_only: Vec<(&DiscoveredFile, &FileRecord)> = Vec::new();
    let mut skipped: Vec<&DiscoveredFile> = Vec::new();
    let mut supported = 0usize;

    for d in &discovered {
        supported += 1;
        match existing_by_path.get(&d.rel_path) {
            None => {
                if d.size > config.index.max_file_size as i64 {
                    skipped.push(d);
                } else {
                    to_parse.push((d, None));
                }
            }
            Some(rec) => {
                if rec.size != d.size || rec.mtime != d.mtime {
                    // content may have changed: hash it to be sure
                    let path = root.join(&d.rel_path);
                    let current = std::fs::read(&path).unwrap_or_default();
                    let h = hash_bytes(&current);
                    if h == rec.hash {
                        metadata_only.push((d, rec));
                    } else {
                        to_parse.push((d, Some(rec)));
                    }
                } else {
                    unchanged.push(rec);
                }
            }
        }
    }
    report.supported_files = supported;
    report.unchanged_files = unchanged.len();
    report.metadata_only = metadata_only.len();
    report.skipped = skipped.len();
    if report.skipped > 0 {
        report.skipped_reason = Some(OVERSIZED_SKIPPED.to_string());
    }

    // Deleted
    let deleted: Vec<String> = existing_by_path
        .keys()
        .filter(|p| !discovered_set.contains(p.as_str()))
        .cloned()
        .collect();
    report.deleted_files = deleted.len();

    // Parse in parallel. Hard failures are recorded so the report can tell the
    // user which files could not be parsed; the rest of the index is unaffected.
    let parsed_results: Vec<(String, Result<ParsedOutcome, String>)> = if to_parse.is_empty() {
        Vec::new()
    } else {
        let root_arc = root.to_path_buf();
        to_parse
            .par_iter()
            .map(|(d, _)| {
                let path = root_arc.join(&d.rel_path);
                let data = match std::fs::read(&path) {
                    Ok(d) => d,
                    Err(e) => {
                        return (d.rel_path.clone(), Err(format!("read error: {e}")));
                    }
                };
                if data.is_empty() {
                    return (
                        d.rel_path.clone(),
                        Ok(ParsedOutcome {
                            rel_path: d.rel_path.clone(),
                            language: d.language,
                            parsed: ParsedFile {
                                language: d.language,
                                symbols: Vec::new(),
                                dependencies: Vec::new(),
                                has_errors: false,
                            },
                        }),
                    );
                }
                let source = String::from_utf8_lossy(&data).into_owned();
                match parse_source(d.language, &source, &d.rel_path, &root_arc) {
                    Ok(parsed) => (
                        d.rel_path.clone(),
                        Ok(ParsedOutcome {
                            rel_path: d.rel_path.clone(),
                            language: d.language,
                            parsed,
                        }),
                    ),
                    Err(e) => {
                        tracing::debug!("parse error {}: {e}", d.rel_path);
                        (d.rel_path.clone(), Err(e.to_string()))
                    }
                }
            })
            .collect()
    };

    let outcomes: Vec<ParsedOutcome> = parsed_results
        .iter()
        .filter_map(|(_, r)| r.as_ref().ok().cloned())
        .collect();
    let hard_failures: Vec<String> = parsed_results
        .iter()
        .filter_map(|(path, r)| r.as_ref().err().map(|e| format!("{path}: {e}")))
        .collect();

    report.parsed_files = outcomes.len();

    let mut id_by_path: HashMap<String, i64> = HashMap::new();
    let mut parse_errors_inner: Vec<String> = Vec::new();

    let tx = db.begin()?;
    // deletions
    for path in &deleted {
        Database::delete_file(&tx, path)?;
    }
    // upsert every discovered file (with existing hash where unchanged)
    for d in &discovered {
        let (hash, mtime, size) = if let Some(rec) = existing_by_path.get(&d.rel_path) {
            if rec.size == d.size && rec.mtime == d.mtime {
                (rec.hash.clone(), d.mtime, d.size)
            } else {
                // metadata-only case: keep old hash
                (rec.hash.clone(), d.mtime, d.size)
            }
        } else {
            (
                hash_bytes(&std::fs::read(root.join(&d.rel_path)).unwrap_or_default()),
                d.mtime,
                d.size,
            )
        };
        let id = Database::upsert_file(&tx, &d.rel_path, &hash, mtime, d.language.as_str(), size)?;
        id_by_path.insert(d.rel_path.clone(), id);
    }
    // index new/changed content
    for outcome in &outcomes {
        let Some(&file_id) = id_by_path.get(&outcome.rel_path) else {
            continue;
        };
        report.symbols_indexed += outcome.parsed.symbols.len();
        report.dependencies_indexed += outcome.parsed.dependencies.len();
        Database::replace_symbols(&tx, file_id, &outcome.parsed.symbols)?;
        Database::replace_dependencies(&tx, file_id, &outcome.parsed.dependencies, &id_by_path)?;
        if outcome.parsed.has_errors {
            parse_errors_inner.push(outcome.rel_path.clone());
        }
    }
    drop(id_by_path);
    tx.commit()?;

    parse_errors_inner.sort();
    parse_errors_inner.extend(hard_failures);
    report.parse_errors = parse_errors_inner;
    report.elapsed_ms = start.elapsed().as_millis() as u64;
    Ok(report)
}

pub fn index_single_file(
    root: &Path,
    db: &mut Database,
    rel_path: &str,
    language: LanguageId,
    config: &Config,
) -> CtxResult<()> {
    // Re-index one file (used by watch mode / changed).
    let path = root.join(rel_path);
    let data = std::fs::read(&path).unwrap_or_default();
    let hash = hash_bytes(&data);
    let mtime = std::fs::metadata(&path)
        .ok()
        .and_then(|m| {
            m.modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
        })
        .unwrap_or(0);
    let size = path.metadata().map(|m| m.len() as i64).unwrap_or(0);

    let parsed = if data.is_empty() || size as u64 > config.index.max_file_size {
        ParsedFile {
            language,
            symbols: Vec::new(),
            dependencies: Vec::new(),
            has_errors: false,
        }
    } else {
        let source = String::from_utf8_lossy(&data).into_owned();
        parse_source(language, &source, rel_path, root)?
    };

    let id_map = db.path_id_map()?;
    let tx = db.begin()?;
    let file_id = Database::upsert_file(&tx, rel_path, &hash, mtime, language.as_str(), size)?;
    Database::replace_symbols(&tx, file_id, &parsed.symbols)?;
    Database::replace_dependencies(&tx, file_id, &parsed.dependencies, &id_map)?;
    tx.commit()?;
    Ok(())
}

pub fn remove_file(root: &Path, db: &mut Database, rel_path: &str) -> CtxResult<()> {
    let _ = root;
    let tx = db.begin()?;
    Database::delete_file(&tx, rel_path)?;
    tx.commit()?;
    Ok(())
}
