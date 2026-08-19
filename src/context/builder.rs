//! Context package builder: selects the most relevant files + symbols for a
//! task using deterministic signals (no external LLM).

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::config::Config;
use crate::context::ranking::{
    TokenMatch, best_token_match, expand_keywords, file_reasons, framework_bonus, hub_bonus,
    idf_keyword_weights, path_keyword_bonus_w, score_symbol_w, symbol_reasons, tokenize,
    word_tokens,
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
    /// True when token counts are heuristic estimates (bytes/4), never precise
    /// counts. The numeric estimate is `total_tokens`.
    pub is_estimate: bool,
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
    build_context_with(db, root, task, config, include_bodies, None, None)
}

/// Extended builder with a token budget override and optional working-tree
/// git changes (used by the CLI and MCP tool). `git_changes` is `None` when the
/// git signal was not consulted (no repo / `--no-git`), `Some([])` when it was
/// consulted but nothing changed, and `Some(paths)` when files are modified.
pub fn build_context_with(
    db: &Database,
    root: &Path,
    task: &str,
    config: &Config,
    include_bodies: bool,
    max_tokens: Option<usize>,
    git_changes: Option<&[String]>,
) -> CtxResult<ContextPackage> {
    let display_keywords = tokenize(task);
    let keywords = expand_keywords(&display_keywords);
    let data = db.context_load()?;
    let (files, symbols, dep_counts) = (data.files, data.symbols, data.dep_counts);

    // IDF dampening: keywords that match a large share of the corpus ("api" in
    // a repo full of `*.api.ts` modules) carry little signal, so their exact
    // symbol/name/path hits are weighted down. This stops a single generic term
    // from flooding the package with structurally-identical files while the
    // real targets (rare, precisely-named symbols) keep full weight.
    // Document frequency counts only strong identity signals — the file path
    // and symbol NAMES — not signature text, which is full of common parameter
    // identifiers ("user", "data") that appear in dozens of files and would
    // falsely damp perfectly relevant keywords.
    let paths: Vec<String> = files.values().map(|f| f.path.clone()).collect();
    let path_index: std::collections::HashMap<i64, usize> =
        files.values().enumerate().map(|(i, f)| (f.id, i)).collect();
    let path_tokens: Vec<Vec<String>> = paths.iter().map(|p| word_tokens(p)).collect();
    let symbol_refs: Vec<(Vec<String>, usize)> = symbols
        .iter()
        .filter_map(|s| {
            path_index
                .get(&s.file_id)
                .map(|&i| (word_tokens(&s.name), i))
        })
        .collect();
    let document_frequencies: Vec<usize> = keywords
        .iter()
        .map(|k| {
            let mut hit = vec![false; paths.len()];
            let mut count = 0usize;
            for (i, tokens) in path_tokens.iter().enumerate() {
                if best_token_match(tokens, k) != TokenMatch::None && !hit[i] {
                    hit[i] = true;
                    count += 1;
                }
            }
            for (name_tokens, file_index) in &symbol_refs {
                if !hit[*file_index] && best_token_match(name_tokens, k) != TokenMatch::None {
                    hit[*file_index] = true;
                    count += 1;
                }
            }
            count
        })
        .collect();
    let keyword_weights = idf_keyword_weights(&document_frequencies);

    let git_considered = git_changes.is_some();
    let changed: std::collections::HashSet<&str> = git_changes
        .unwrap_or_default()
        .iter()
        .map(|p| p.as_str())
        .collect();

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
        let score = score_symbol_w(
            &s.name,
            s.signature.as_deref().unwrap_or(""),
            &file.path,
            &keywords,
            &keyword_weights,
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
                reasons: reasons.clone(),
            });
            let entry = file_scores.entry(s.file_id).or_insert_with(|| ScoredFile {
                record: file.clone(),
                score: 0.0,
                reasons: Vec::new(),
            });
            entry.score = entry.score.max(score);
            entry.reasons.extend(reasons);
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
        score += path_keyword_bonus_w(&file.path, &keywords, &keyword_weights);
        score += if is_recent { 2.0 } else { 0.0 };
        score += framework_bonus(&file.path);
        if let Some(dc) = hub_count {
            score += hub_bonus(dc);
        }
        if in_git {
            score += 1.5;
        }
        entry.score = score;
        let mut file_reason = file_reasons(
            &file.path,
            &keywords,
            is_recent,
            is_framework,
            hub_count,
            in_git,
        );
        entry.reasons.append(&mut file_reason);
        entry.reasons.dedup();
    }

    // also consider paths alone (a strongly-path-matched file with no symbol hits)
    for rec in files.values() {
        if file_scores.contains_key(&rec.id) {
            continue;
        }
        let kw = path_keyword_bonus_w(&rec.path, &keywords, &keyword_weights);
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

    // dependency following: files imported by strongly-relevant files often
    // need to be edited too (e.g. adding a subscription tier touches billing.ts
    // AND the stripe/payment clients it imports). Give those one-hop deps a
    // boost that scales with the strongest relevant importer, so the package
    // prefers the deps of the most-relevant files over generic framework noise.
    // Both follow passes expand only from the *directly* relevant roots (symbol
    // and path matches) captured before any follow additions; a dependency that
    // is itself a hub must not drag its whole dependent tree in.
    let roots: Vec<(i64, f64)> = file_scores
        .iter()
        .filter(|(_, e)| e.score >= 3.0)
        .map(|(fid, e)| (*fid, e.score))
        .collect();
    // Files whose only signal is a follow edge (dependency/dependent of a
    // relevant file) — they have no direct keyword/symbol/path hit. These are
    // ranked after every direct hit and capped so they can never crowd out a
    // genuinely-needed direct match or flood the package.
    let mut follow_only: HashSet<i64> = HashSet::new();
    let mut follow_ids: Vec<i64> = Vec::new();
    let mut follow_parent_score: HashMap<i64, f64> = HashMap::new();
    for (fid, score) in &roots {
        if let Ok(deps) = db.dependencies_of(*fid) {
            for d in deps {
                if let Some(tid) = d.target_file_id {
                    if !follow_ids.contains(&tid) {
                        follow_ids.push(tid);
                    }
                    let ps = follow_parent_score.entry(tid).or_insert(0.0);
                    if *score > *ps {
                        *ps = *score;
                    }
                }
            }
        }
    }
    for tid in follow_ids {
        let Some(rec) = files.get(&tid) else {
            continue;
        };
        let hub_count = dep_counts.get(&tid).copied();
        let is_recent = recent_on_disk(rec);
        let parent = follow_parent_score.get(&tid).copied().unwrap_or(3.0);
        let mut score = 2.0 + parent * 0.1;
        score += if is_recent { 2.0 } else { 0.0 };
        if let Some(dc) = hub_count {
            score += hub_bonus(dc);
        }
        let reason = "imported by a relevant file (dependency)".to_string();
        let extra_reasons = file_reasons(
            &rec.path,
            &keywords,
            is_recent,
            framework_bonus(&rec.path) > 0.0,
            hub_count,
            false,
        );
        // A dependency of a strongly-relevant file may already be scored via a
        // weak symbol/prefix hit; give it the follow boost too.
        if let Some(existing) = file_scores.get_mut(&tid) {
            if score > existing.score {
                existing.score = score;
            }
            existing.reasons.push(reason);
            existing.reasons.extend(extra_reasons);
            existing.reasons.dedup();
            continue;
        }
        follow_only.insert(tid);
        file_scores.insert(
            tid,
            ScoredFile {
                record: rec.clone(),
                score,
                reasons: std::iter::once(reason).chain(extra_reasons).collect(),
            },
        );
    }

    // dependents following (reverse edges): files that IMPORT a strongly-
    // relevant file need to be edited too when that file's contract changes
    // (e.g. renaming a field on the User model touches every service that
    // reads it, replacing the API client touches every module that imports it).
    // A shared hub (a db connection imported by thousands of leaf modules)
    // must never flood the package, so dependents of an oversized hub are only
    // followed when they are themselves integration points (imported by
    // several files) — genuine consumers like a React context provider that
    // wraps the client, not one-off leaf importers.
    const MAX_FOLLOW_DEPENDENTS: usize = 40;
    const MIN_HUB_FOLLOW_DEPENDENTS: i64 = 2;
    let mut reverse_follow_ids: Vec<i64> = Vec::new();
    let mut reverse_parent_score: HashMap<i64, f64> = HashMap::new();
    for (fid, score) in &roots {
        if let Ok(dependents) = db.dependents_of(*fid) {
            let follow_all = dependents.len() <= MAX_FOLLOW_DEPENDENTS;
            for (dep_path, _) in dependents {
                let Some(dep_file) = files.values().find(|f| f.path == dep_path) else {
                    continue;
                };
                let tid = dep_file.id;
                if !follow_all {
                    // Oversized hub: only integration points are followed so the
                    // package surfaces the files that fan the change out to the
                    // rest of the codebase instead of every leaf consumer.
                    let dc = dep_counts.get(&tid).copied().unwrap_or(0);
                    if dc < MIN_HUB_FOLLOW_DEPENDENTS {
                        continue;
                    }
                }
                if !reverse_follow_ids.contains(&tid) {
                    reverse_follow_ids.push(tid);
                }
                let ps = reverse_parent_score.entry(tid).or_insert(0.0);
                if *score > *ps {
                    *ps = *score;
                }
            }
        }
    }
    for tid in reverse_follow_ids {
        let Some(rec) = files.get(&tid) else {
            continue;
        };
        let hub_count = dep_counts.get(&tid).copied();
        let is_recent = recent_on_disk(rec);
        let parent = reverse_parent_score.get(&tid).copied().unwrap_or(3.0);
        let mut score = 2.0 + parent * 0.1;
        score += if is_recent { 2.0 } else { 0.0 };
        if let Some(dc) = hub_count {
            score += hub_bonus(dc);
        }
        let reason = "imports a relevant file (dependent)".to_string();
        let extra_reasons = file_reasons(
            &rec.path,
            &keywords,
            is_recent,
            framework_bonus(&rec.path) > 0.0,
            hub_count,
            false,
        );
        // The dependent of a strongly-relevant file may already be scored via
        // a weak symbol/prefix hit. It must still receive the reverse-follow
        // boost (and its reason) so it isn't crowded out of the package.
        if let Some(existing) = file_scores.get_mut(&tid) {
            if score > existing.score {
                existing.score = score;
            }
            existing.reasons.push(reason);
            existing.reasons.extend(extra_reasons);
            existing.reasons.dedup();
            continue;
        }
        follow_only.insert(tid);
        file_scores.insert(
            tid,
            ScoredFile {
                record: rec.clone(),
                score,
                reasons: std::iter::once(reason).chain(extra_reasons).collect(),
            },
        );
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

    // pick candidate files (top by score). Follow-only files (dependencies/
    // dependents of relevant files) are capped so a hub can never flood the
    // package or consume budget that belongs to a genuinely-needed match.
    const MAX_FOLLOW_ONLY: usize = 6;
    let mut selected_ids: Vec<i64> = Vec::new();
    let mut follow_seen = 0usize;
    for s in &scored {
        if follow_only.contains(&s.record.id) {
            if follow_seen < MAX_FOLLOW_ONLY {
                selected_ids.push(s.record.id);
                follow_seen += 1;
            }
        } else {
            selected_ids.push(s.record.id);
        }
    }
    // always include files holding the highest-scoring symbols
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
        // When bodies are included the real payload is the full file, so the
        // token budget must be charged against that text, not the body-less
        // skeleton (which would under-count and blow the budget silently).
        let included = if include_bodies {
            std::fs::read_to_string(root.join(rel)).unwrap_or_else(|_| opts.skeleton.clone())
        } else {
            opts.skeleton
        };
        let included_tokens = estimate_tokens(&included);
        if included_tokens > budget {
            if relevant_files.is_empty() {
                // keep the single best candidate even if it overflows; this is
                // the only intentional overshoot and it is reported below.
            } else {
                omitted_files += 1;
                continue;
            }
        } else {
            budget -= included_tokens;
        }
        total_tokens += included_tokens;

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
            skeleton: included,
            tokens: included_tokens,
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
        keywords: display_keywords,
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
        is_estimate: true,
        git_changes_considered: git_considered,
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
    if files.is_empty() {
        return lines;
    }
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
