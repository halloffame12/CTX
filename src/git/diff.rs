//! Semantic `git diff`: added / modified / removed symbols between refs.

use std::collections::BTreeMap;
use std::path::Path;

use crate::errors::CtxResult;
use crate::git::GitRepo;
use crate::git::changed::changed_files;
use crate::lang::LanguageId;
use crate::parser::{Symbol, parse_source};
#[derive(Debug, Clone, serde::Serialize)]
pub struct SymbolDiffEntry {
    pub status: String, // Added | Removed | Modified
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileDiff {
    pub path: String,
    pub status: String, // A M D R
    pub symbols: Vec<SymbolDiffEntry>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SymbolDiff {
    pub base: String,
    pub head: String,
    pub files: Vec<FileDiff>,
    pub added: usize,
    pub modified: usize,
    pub removed: usize,
    pub large_changes: bool,
}

pub fn symbol_diff(
    repo: &GitRepo,
    base: Option<&str>,
    head: Option<&str>,
    test_mode_root: Option<&Path>,
) -> CtxResult<SymbolDiff> {
    // Single-ref diff (base..worktree): resolve the base to its merge-base
    // with HEAD so the diff shows only the current branch's changes, not
    // everything the base ref did since the fork (documented behavior).
    // The label keeps what the user typed; git ops use the resolved commit.
    let (base_label, base) = match (base, head) {
        (Some(b), None) => {
            let label = b.to_string();
            let resolved = merge_base(repo, b, "HEAD")?.unwrap_or_else(|| b.to_string());
            (label, resolved)
        }
        (b, _) => {
            let label = b.unwrap_or("HEAD").to_string();
            (label.clone(), label)
        }
    };
    // Two-ref diff (base..head): both sides come from git history.
    // Single-ref diff (base..worktree): the "new" side is the working tree.
    let (head_label, files, resolve_head) = match head {
        Some(h) => (h.to_string(), diff_between(repo, &base, h)?, true),
        None => (
            "worktree".to_string(),
            changed_files(repo, Some(&base), false)?,
            false,
        ),
    };
    let mut out: Vec<FileDiff> = Vec::new();
    let mut added = 0;
    let mut modified = 0;
    let mut removed = 0;
    let mut large_changes = false;

    let root: &Path = test_mode_root.unwrap_or(&repo.root);

    for cf in &files {
        let rel = &cf.path;
        let Some(lang) = LanguageId::from_extension(
            Path::new(rel)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or(""),
        ) else {
            continue;
        };
        let old_src = if cf.status != "A" {
            // For a rename the source path is where the symbols lived at the
            // base; the new path does not exist there yet.
            let old_path = cf.old_path.as_deref().unwrap_or(rel);
            repo.show(&base, old_path)?
        } else {
            String::new()
        };
        let new_src = if cf.status == "D" {
            String::new()
        } else if resolve_head {
            // both sides are historical: read "new" content from the head ref
            repo.show(&head_label, rel)?
        } else {
            // working tree
            std::fs::read_to_string(root.join(rel))
                .unwrap_or_else(|_| repo.show(&base, rel).unwrap_or_default())
        };

        let old_syms = symbols_from(&old_src, lang, root, rel);
        let new_syms = symbols_from(&new_src, lang, root, rel);

        if let Some(syms) = diff_symbols(rel, &old_syms, &new_syms) {
            let a = syms.iter().filter(|s| s.status == "Added").count();
            let m = syms.iter().filter(|s| s.status == "Modified").count();
            let r = syms.iter().filter(|s| s.status == "Removed").count();
            added += a;
            modified += m;
            removed += r;
            out.push(FileDiff {
                path: rel.clone(),
                status: cf.status.clone(),
                symbols: syms,
            });
        } else {
            out.push(FileDiff {
                path: rel.clone(),
                status: cf.status.clone(),
                symbols: Vec::new(),
            });
        }
    }
    if out.len() > 200 {
        large_changes = true;
    }
    Ok(SymbolDiff {
        base: base_label,
        head: head_label,
        files: out,
        added,
        modified,
        removed,
        large_changes,
    })
}

/// Best common ancestor of two refs; None when histories are unrelated.
fn merge_base(repo: &GitRepo, a: &str, b: &str) -> CtxResult<Option<String>> {
    match repo.run(&["merge-base", a, b]) {
        Ok(mb) => {
            let t = mb.trim().to_string();
            if t.is_empty() { Ok(None) } else { Ok(Some(t)) }
        }
        Err(_) => Ok(None),
    }
}

/// Files changed between two explicit refs (base..head), rename-aware.
fn diff_between(
    repo: &GitRepo,
    base: &str,
    head: &str,
) -> CtxResult<Vec<crate::git::changed::ChangedFile>> {
    use std::collections::BTreeMap;
    let raw = repo.run(&["diff", "--name-status", base, head])?;
    let mut out: BTreeMap<String, (String, Option<String>)> = BTreeMap::new();
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
            fields
                .get(fields.len().saturating_sub(2))
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
        } else {
            None
        };
        out.insert(path, (code.to_string(), old_path));
    }
    Ok(out
        .into_iter()
        .filter(|(path, _)| path != ".ctx" && !path.starts_with(".ctx/"))
        .map(
            |(path, (status, old_path))| crate::git::changed::ChangedFile {
                path,
                status,
                old_path,
            },
        )
        .collect())
}

fn symbols_from(src: &str, lang: LanguageId, root: &Path, rel: &str) -> Vec<Symbol> {
    if src.is_empty() {
        return Vec::new();
    }
    parse_source(lang, src, rel, root)
        .map(|p| p.symbols)
        .unwrap_or_default()
}

/// Compare two symbol lists by name; produce per-name status.
fn diff_symbols(rel: &str, old: &[Symbol], new: &[Symbol]) -> Option<Vec<SymbolDiffEntry>> {
    let name_of = |s: &Symbol| -> String {
        match (&s.parent, &s.name) {
            (Some(p), n) if *p != *n => format!("{p}.{n}"),
            _ => s.name.clone(),
        }
    };
    let mut by_name_old: BTreeMap<String, (&Symbol, usize)> = BTreeMap::new();
    let mut old_sig: BTreeMap<String, String> = BTreeMap::new();
    for s in old {
        let n = name_of(s);
        by_name_old.insert(n.clone(), (s, s.span.start_line as usize));
        old_sig.insert(n, s.signature.clone());
    }
    let mut entries: Vec<SymbolDiffEntry> = Vec::new();
    for s in new {
        let n = name_of(s);
        match old_sig.get(&n) {
            Some(prev) if *prev == s.signature => {}
            Some(_) => entries.push(SymbolDiffEntry {
                status: "Modified".to_string(),
                name: n.clone(),
                kind: s.kind.as_str().to_string(),
                file: rel.to_string(),
                line: s.span.start_line,
                signature: Some(s.signature.clone()),
            }),
            None => entries.push(SymbolDiffEntry {
                status: "Added".to_string(),
                name: n.clone(),
                kind: s.kind.as_str().to_string(),
                file: rel.to_string(),
                line: s.span.start_line,
                signature: Some(s.signature.clone()),
            }),
        }
    }
    let new_names: std::collections::HashSet<String> = new.iter().map(name_of).collect();
    for s in old {
        let n = name_of(s);
        if !new_names.contains(&n) {
            entries.push(SymbolDiffEntry {
                status: "Removed".to_string(),
                name: n,
                kind: s.kind.as_str().to_string(),
                file: rel.to_string(),
                line: s.span.start_line,
                signature: None,
            });
        }
    }
    let _ = by_name_old;
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}
