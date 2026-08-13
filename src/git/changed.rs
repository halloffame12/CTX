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
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChangedSymbol {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChangedReport {
    pub since: String,
    pub files: Vec<ChangedFile>,
    pub symbols: Vec<ChangedSymbol>,
    pub count: usize,
}

/// Files changed relative to `since` (or the working tree when None).
pub fn changed_files(repo: &GitRepo, since: Option<&str>) -> CtxResult<Vec<ChangedFile>> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    match since {
        Some(rev) => {
            let raw = repo.run(&["diff", "--name-status", rev])?;
            for line in raw.lines() {
                let mut parts = line.splitn(3, '\t');
                let status = parts.next().unwrap_or("M");
                let path = parts.last().unwrap_or("").trim().to_string();
                if path.is_empty() {
                    continue;
                }
                let code = match status.chars().next() {
                    Some('A') => "A",
                    Some('D') => "D",
                    Some('R') | Some('C') => "R",
                    _ => "M",
                };
                out.insert(path, code.to_string());
            }
            // untracked files are invisible to diff
            if let Ok(u) = repo.run(&["ls-files", "--others", "--exclude-standard"]) {
                for line in u.lines() {
                    let p = line.trim().to_string();
                    if !p.is_empty() {
                        out.entry(p).or_insert_with(|| "A".to_string());
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
                let path = line[3..].trim_end().to_string();
                let path = path.split(" -> ").last().unwrap_or(&path).to_string();
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
                out.insert(path, code.to_string());
            }
        }
    }
    Ok(out
        .into_iter()
        .filter(|(path, _)| {
            let p = path.as_str();
            p != ".ctx" && !p.starts_with(".ctx/") && p != ".git" && !p.starts_with(".git/")
        })
        .map(|(path, status)| ChangedFile { path, status })
        .collect())
}

/// Map changed files to symbols in the graph.
pub fn changed_symbols(
    repo: &GitRepo,
    db: &Database,
    since: Option<&str>,
) -> CtxResult<ChangedReport> {
    let files = changed_files(repo, since)?;
    let mut symbols: Vec<ChangedSymbol> = Vec::new();
    for cf in &files {
        if cf.status == "D" {
            continue;
        }
        if let Some(file) = db.file_by_path(&cf.path)? {
            for s in db.symbols_for_file(file.id)? {
                symbols.push(ChangedSymbol {
                    name: s.name,
                    kind: s.kind,
                    file: cf.path.clone(),
                    line: s.start_line,
                });
            }
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
    let files = changed_files(repo, None)?;
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
