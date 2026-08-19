//! Files & symbols changed since a ref (or in the working tree).

use std::collections::BTreeMap;
use std::path::Path;

use crate::config::Config;
use crate::errors::CtxResult;
use crate::git::GitRepo;
use crate::graph::database::Database;
use crate::indexing::incremental::{index_single_file, remove_file};
use crate::lang::LanguageId;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChangedFile {
    pub path: String,
    pub status: String,
    /// Source path for renames/copies (absent for non-rename changes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChangedSymbol {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: u32,
    /// How the symbol changed relative to the base: Added | Modified | Removed.
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChangedReport {
    pub since: String,
    pub files: Vec<ChangedFile>,
    pub symbols: Vec<ChangedSymbol>,
    pub count: usize,
}

/// Files changed relative to `since` (or the working tree when None).
///
/// `include_untracked` controls whether untracked files (visible to `git
/// status` but not `git diff`) are reported. Diff semantics must exclude
/// them so `ctx diff` matches `git diff`.
pub fn changed_files(
    repo: &GitRepo,
    since: Option<&str>,
    include_untracked: bool,
) -> CtxResult<Vec<ChangedFile>> {
    let mut out: BTreeMap<String, (String, Option<String>)> = BTreeMap::new();
    match since {
        Some(rev) => {
            let raw = repo.run(&["diff", "--name-status", rev])?;
            for line in raw.lines() {
                let fields: Vec<&str> = line.split('\t').collect();
                if fields.is_empty() {
                    continue;
                }
                let status = fields[0];
                let code = match status.chars().next() {
                    Some('A') => "A",
                    Some('D') => "D",
                    Some('R') | Some('C') => "R",
                    _ => "M",
                };
                let is_rename = code == "R";
                let path = fields.last().unwrap_or(&"").trim().to_string();
                if path.is_empty() {
                    continue;
                }
                let old_path = if is_rename {
                    // `R100\told\tnew` — middle field is the source path.
                    fields
                        .get(fields.len().saturating_sub(2))
                        .map(|p| p.trim().to_string())
                        .filter(|p| !p.is_empty())
                } else {
                    None
                };
                out.insert(path, (code.to_string(), old_path));
            }
            // untracked files are invisible to diff but visible to status;
            // include them only for status-like reporting.
            if include_untracked {
                if let Ok(u) = repo.run(&["ls-files", "--others", "--exclude-standard"]) {
                    for line in u.lines() {
                        let p = line.trim().to_string();
                        if !p.is_empty() {
                            out.entry(p).or_insert_with(|| ("A".to_string(), None));
                        }
                    }
                }
            }
        }
        None => {
            let raw = repo.run(&["status", "--porcelain=v1"])?;
            for line in raw.lines() {
                if line.len() < 4 {
                    continue;
                }
                let prefix = &line[..2];
                let raw_path = line[3..].trim_end().to_string();
                // porcelain v1 shows renames as `R  old -> new`; keep both.
                let old_path = raw_path
                    .split(" -> ")
                    .next()
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty() && *p != raw_path);
                let path = raw_path
                    .split(" -> ")
                    .last()
                    .unwrap_or(&raw_path)
                    .trim()
                    .to_string();
                // porcelain v1 is a two-column code: X=index, Y=worktree.
                let (x, y) = (
                    prefix.chars().next().unwrap_or(' '),
                    prefix.chars().nth(1).unwrap_or(' '),
                );
                let code = if x == '?' && y == '?' {
                    "A"
                } else {
                    let tracked =
                        |c: char| c == 'A' || c == 'D' || c == 'R' || c == 'C' || c == 'M';
                    if tracked(x) {
                        match x {
                            'A' => "A",
                            'D' => "D",
                            'R' | 'C' => "R",
                            _ => "M",
                        }
                    } else if tracked(y) {
                        match y {
                            'A' => "A",
                            'D' => "D",
                            'R' | 'C' => "R",
                            _ => "M",
                        }
                    } else {
                        "M"
                    }
                };
                out.insert(path, (code.to_string(), old_path));
            }
        }
    }
    Ok(out
        .into_iter()
        .filter(|(path, _)| {
            let p = path.as_str();
            p != ".ctx" && !p.starts_with(".ctx/") && p != ".git" && !p.starts_with(".git/")
        })
        .map(|(path, (status, old_path))| ChangedFile {
            path,
            status,
            old_path,
        })
        .collect())
}

/// Map changed files to the symbols that actually changed (added, modified or
/// removed) between the base ref and the working tree, reusing the semantic
/// symbol diff. The graph `db` is not needed here — the diff parses both sides
/// directly — but is kept in the signature for call-site compatibility.
pub fn changed_symbols(
    _repo: &GitRepo,
    _db: &Database,
    since: Option<&str>,
) -> CtxResult<ChangedReport> {
    let sd = crate::git::diff::symbol_diff(_repo, since, None, None)?;
    let files: Vec<ChangedFile> = sd
        .files
        .iter()
        .map(|f| ChangedFile {
            path: f.path.clone(),
            status: f.status.clone(),
            old_path: None,
        })
        .collect();
    let mut symbols: Vec<ChangedSymbol> = Vec::new();
    for f in &sd.files {
        for s in &f.symbols {
            symbols.push(ChangedSymbol {
                name: s.name.clone(),
                kind: s.kind.clone(),
                file: f.path.clone(),
                line: s.line,
                status: s.status.clone(),
            });
        }
    }
    symbols.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    Ok(ChangedReport {
        since: since.unwrap_or("working tree").to_string(),
        count: files.len(),
        symbols,
        files,
    })
}

/// Index changed files into the graph so queries reflect current disk state.
pub fn sync_changed(repo: &GitRepo, db: &mut Database, config: &Config) -> CtxResult<()> {
    let files = changed_files(repo, None, true)?;
    for cf in &files {
        let rel = crate::indexing::scanner::normalize(&cf.path);
        let Some(lang) = LanguageId::from_extension(
            Path::new(&rel)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or(""),
        ) else {
            continue;
        };
        if cf.status == "D" {
            remove_file(&repo.root, db, &rel)?;
        } else {
            index_single_file(&repo.root, db, &rel, lang, config)?;
        }
    }
    Ok(())
}
