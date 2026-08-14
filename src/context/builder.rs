//! Context package builder: selects the most relevant files + symbols for a
//! task using deterministic signals (no external LLM).

use std::collections::HashMap;
use std::path::Path;

use crate::config::Config;
use crate::context::ranking::{
    file_reasons, framework_bonus, hub_bonus, path_keyword_bonus, score_symbol, symbol_reasons,
    tokenize,
};
use crate::context::skeleton::{estimate_tokens, skeleton_for};
use crate::errors::CtxResult;
use crate::graph::database::{Database, FileRecord};
use crate::lang::LanguageId;

#[derive(Debug, Clone, serde::Serialize)]
pub struct RelevantSymbol {
    pub name: String,
    pub kind: String,
    pub path: String,
    pub line: u32,
    pub signature: Option<String>,
    pub score: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RelevantFile {
    pub path: String,
    pub score: f64,
    pub language: Option<String>,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub skeleton: String,
    pub tokens: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ContextPackage {
    pub task: String,
    pub keywords: Vec<String>,
    pub architecture: Vec<String>,
    pub relevant_symbols: Vec<RelevantSymbol>,
    pub relevant_dependencies: Vec<String>,
    pub files: Vec<RelevantFile>,
    pub suggested_context: String,
    pub files_analyzed: usize,
    /// Requested token budget for the suggested context.
    pub budget: usize,
    /// Total estimated tokens actually included.
    pub total_tokens: usize,
    /// Candidate files left out because of the budget or max_files cap.
    pub omitted_files: usize,
    /// True when `total_tokens` could not fit inside `budget`.
    pub budget_exceeded: bool,
    /// Token counts are heuristic estimates (bytes/4), never precise counts.
    pub token_estimate: bool,
    /// Whether working-tree git changes were considered in scoring.
    pub git_changes_considered: bool,
}

#[derive(Debug, Clone)]
struct ScoredFile {
    record: FileRecord,
    score: f64,
    reasons: Vec<String>,
}

pub fn build_context(
    db: &Database,
    root: &Path,
    task: &str,
    config: &Config,
    include_bodies: bool,
) -> CtxResult<ContextPackage> {
    build_context_with(db, root, task, config, include_bodies, None, &[])
}

/// Extended builder with a token budget override and optional working-tree
/// git changes (used by the CLI and MCP tool). Both default to the config when
/// not supplied.
pub fn build_context_with(
    db: &Database,
    root: &Path,
    task: &str,
    config: &Config,
    include_bodies: bool,
    max_tokens: Option<usize>,
    changed_paths: &[String],
) -> CtxResult<ContextPackage> {
    let keywords = tokenize(task);
    let data = db.context_load()?;
    let (files, symbols, dep_counts) = (data.files, data.symbols, data.dep_counts);

    let changed: std::collections::HashSet<&str> =
        changed_paths.iter().map(|p| p.as_str()).collect();

    // On-disk mtime of a candidate file, used for the recency signal. We stat
    // the real file (not the DB snapshot) so an edit made since the last index
    // is honoured; fresh checkouts are not flagged because the index was built
    // after them.
    let index_built_at = std::fs::metadata(root.join(".ctx/index.db"))
        .and_then(|m| m.modified())
        .map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    let recent_on_disk = |record: &FileRecord| -> bool {
        let on_disk = std::fs::metadata(root.join(&record.path))
            .and_then(|m| m.modified())
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
            })
            .unwrap_or(record.mtime);
        on_disk > index_built_at
    };

    let mut file_scores: HashMap<i64, ScoredFile> = HashMap::new();
    let mut relevant_symbols: Vec<RelevantSymbol> = Vec::new();

    // score symbols
    for s in &symbols {
        let Some(file) = files.get(&s.file_id) else {
            continue;
        };
        let score = score_symbol(
            &s.name,
            s.signature.as_deref().unwrap_or(""),
            &file.path,
            &keywords,
        );
        if score > 0.0 {
            let reasons = symbol_reasons(
                &s.name,
                s.signature.as_deref().unwrap_or(""),
                &file.path,
                &keywords,
            );
            relevant_symbols.push(RelevantSymbol {
                name: s.name.clone(),
                kind: s.kind.clone(),
                path: file.path.clone(),
                line: s.start_line,
                signature: s.signature.clone(),
                score,
                reasons,
            });
            let entry = file_scores.entry(s.file_id).or_insert_with(|| ScoredFile {
                record: file.clone(),
                score: 0.0,
                reasons: Vec::new(),
            });
            entry.score = entry.score.max(score);
        }
    }

    // file-level signals
    for (fid, entry) in file_scores.iter_mut() {
        let file = &entry.record;
        let mut score = entry.score;
        let hub_count = dep_counts.get(fid).copied();
        let is_framework = framework_bonus(&file.path) > 0.0;
        let is_recent = recent_on_disk(file);
        let in_git = changed.contains(file.path.as_str());
        score += path_keyword_bonus(&file.path, &keywords);
        score += if is_recent { 2.0 } else { 0.0 };
        score += framework_bonus(&file.path);
        if let Some(dc) = hub_count {
            score += hub_bonus(dc);
        }
        if in_git {
            score += 1.5;
        }
        entry.score = score;
        entry.reasons = file_reasons(
            &file.path,
            &keywords,
            is_recent,
            is_framework,
            hub_count,
            in_git,
        );
    }

    // also consider paths alone (a strongly-path-matched file with no symbol hits)
    for rec in files.values() {
        if file_scores.contains_key(&rec.id) {
            continue;
        }
        let kw = path_keyword_bonus(&rec.path, &keywords);
        if kw > 0.0 {
            let hub_count = dep_counts.get(&rec.id).copied();
            let is_recent = recent_on_disk(rec);
            let mut score = kw + framework_bonus(&rec.path) * 0.5;
            score += if is_recent { 2.0 } else { 0.0 };
            let in_git = changed.contains(rec.path.as_str());
            if in_git {
                score += 1.5;
            }
            file_scores.insert(
                rec.id,
                ScoredFile {
                    record: rec.clone(),
                    score,
                    reasons: file_reasons(
                        &rec.path,
                        &keywords,
                        is_recent,
                        framework_bonus(&rec.path) > 0.0,
                        hub_count,
                        in_git,
                    ),
                },
            );
        }
    }

    // order deterministically: score desc, path asc
    let mut scored: Vec<ScoredFile> = file_scores.into_values().collect();
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.record.path.cmp(&b.record.path))
    });
    relevant_symbols.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });

    // top symbols for display
    relevant_symbols.truncate(30);

    // pick candidate files (top by score) but always include files holding
    // the highest-scoring symbols
    let mut selected_ids: Vec<i64> = scored.iter().map(|s| s.record.id).collect();
    let top_symbol_files: Vec<i64> = relevant_symbols
        .iter()
        .filter(|rs| rs.score >= 3.0)
        .filter_map(|rs| files.values().find(|f| f.path == rs.path).map(|f| f.id))
        .collect();
    for id in top_symbol_files {
        if !selected_ids.contains(&id) {
            selected_ids.push(id);
        }
    }

    let budget_total = max_tokens.unwrap_or(config.context.max_tokens).max(1);
    let mut relevant_files: Vec<RelevantFile> = Vec::new();
    let mut relevant_deps: Vec<String> = Vec::new();
    let mut total_tokens = 0usize;
    let mut budget = budget_total;
    let mut omitted_files = 0usize;

    for id in selected_ids {
        if relevant_files.len() >= config.context.max_files {
            omitted_files += 1;
            continue;
        }
        let Some(sc) = scored
            .iter()
            .find(|s| s.record.id == id)
            .cloned()
            .or_else(|| {
                files.get(&id).map(|rec| ScoredFile {
                    record: rec.clone(),
                    score: 0.0,
                    reasons: Vec::new(),
                })
            })
        else {
            continue;
        };
        let rel = &sc.record.path;
        let Some(lang) = LanguageId::from_str(sc.record.language.as_deref().unwrap_or("")) else {
            continue;
        };
        let Ok(opts) = read_file_skeleton(root, rel, lang) else {
            continue;
        };
        let skeleton_tokens = estimate_tokens(&opts.skeleton);
        if skeleton_tokens > budget {
            if relevant_files.is_empty() {
                // keep the single best candidate even if it overflows; this is
                // the only intentional overshoot and it is reported below.
            } else {
                omitted_files += 1;
                continue;
            }
        } else {
            budget -= skeleton_tokens;
        }
        total_tokens += skeleton_tokens;

        // collect distinct internal + external deps for the package
        if let Ok(deps) = internal_and_external_deps(db, &sc.record) {
            for d in deps {
                if !relevant_deps.contains(&d) && relevant_deps.len() < 12 {
                    relevant_deps.push(d);
                }
            }
        }
        relevant_files.push(RelevantFile {
            path: rel.clone(),
            score: sc.score,
            language: sc.record.language.clone(),
            skeleton: if include_bodies {
                std::fs::read_to_string(root.join(rel)).unwrap_or_else(|_| opts.skeleton.clone())
            } else {
                opts.skeleton
            },
            tokens: skeleton_tokens,
            reasons: sc.reasons,
        });
    }

    let architecture = build_architecture(&relevant_files);
    let mut suggested = String::new();
    suggested.push_str(&format!("# Context package for task: {task}\n\n"));
    for f in &relevant_files {
        if f.skeleton.is_empty() {
            continue;
        }
        suggested.push_str(&format!(
            "\n## {p} (score {s:.2})\n{skeleton}\n",
            p = f.path,
            s = f.score,
            skeleton = f.skeleton
        ));
    }

    Ok(ContextPackage {
        task: task.to_string(),
        keywords,
        architecture,
        relevant_symbols,
        relevant_dependencies: relevant_deps,
        files: relevant_files,
        suggested_context: suggested,
        files_analyzed: scored.len(),
        budget: budget_total,
        total_tokens,
        omitted_files,
        budget_exceeded: total_tokens > budget_total,
        token_estimate: true,
        git_changes_considered: !changed_paths.is_empty(),
    })
}

fn internal_and_external_deps(db: &Database, file: &FileRecord) -> CtxResult<Vec<String>> {
    let mut out = Vec::new();
    for dep in db.dependencies_of(file.id)? {
        if let Some(tid) = dep.target_file_id {
            if let Some(f) = db.file_by_id(tid)? {
                out.push(f.path);
            }
        } else {
            let raw = dep.source_raw.trim();
            if !raw.is_empty() && !raw.starts_with("./") && !raw.starts_with("../") {
                out.push(raw.to_string());
            }
        }
    }
    Ok(out)
}

fn read_file_skeleton(
    root: &Path,
    rel: &str,
    lang: LanguageId,
) -> CtxResult<crate::context::skeleton::SkeletonResult> {
    let path = root.join(rel);
    let data = std::fs::read_to_string(&path)
        .map_err(|e| crate::errors::CtxError::Io(format!("{}: {e}", path.display())))?;
    skeleton_for(root, rel, lang, &data)
}

fn build_architecture(files: &[RelevantFile]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut prefixes: Vec<String> = Vec::new();
    for f in files {
        let parts: Vec<&str> = f.path.split('/').collect();
        if parts.len() >= 2 {
            let prefix = parts[..parts.len() - 1].join("/");
            if !prefixes.contains(&prefix) {
                prefixes.push(prefix);
            }
        }
    }
    prefixes.sort();
    for p in prefixes {
        let depth = p.matches('/').count();
        let indent = "  ".repeat(depth);
        let name = p.rsplit('/').next().unwrap_or(&p);
        lines.push(format!("{indent}{name}/"));
    }
    lines.sort();
    lines.push(String::from("..."));
    lines
}
