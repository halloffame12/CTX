//! Filesystem scanning with gitignore + config exclusion support.

use ignore::WalkBuilder;

use crate::config::Config;
use crate::errors::CtxResult;
use crate::lang::{LanguageId, language_of_path};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    pub rel_path: String,
    pub language: LanguageId,
    pub size: i64,
    pub mtime: i64,
}

pub fn is_ignored(rel: &str, config: &Config) -> bool {
    is_ignored_with_excludes(rel, &config.index.exclude)
}

pub fn is_ignored_with_excludes(rel: &str, exclude: &[String]) -> bool {
    let segments: Vec<&str> = rel.split('/').collect();
    for seg in &segments {
        if seg.is_empty() {
            continue;
        }
        if exclude.iter().any(|e| e == seg) {
            return true;
        }
    }
    for pattern in exclude {
        if rel == *pattern || rel.ends_with(&format!("/{pattern}")) {
            return true;
        }
        // allow glob-ish trailing `/` patterns
        if let Some(stripped) = pattern.strip_suffix('/')
            && let Some(last) = segments.last()
            && last == &stripped
        {
            return true;
        }
    }
    false
}

pub fn scan(root: &Path, config: &Config) -> CtxResult<Vec<DiscoveredFile>> {
    let root_owned = root.to_path_buf();
    let exclude = config.index.exclude.clone();
    let follow_links = config.index.follow_symlinks;
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(true)
        .parents(false)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(true)
        .require_git(false)
        .follow_links(follow_links)
        .filter_entry(move |entry| {
            let path = entry.path();
            if path == root_owned {
                return true;
            }
            let rel = rel_path(&root_owned, path);
            !is_ignored_with_excludes(&rel, &exclude)
        });
    let walker = builder.build();
    let mut out = Vec::new();
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }
        let Some(language) = language_of_path(entry.path()) else {
            continue;
        };
        let rel = rel_path(root, entry.path());
        if is_ignored(&rel, config) {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.len() == 0 {
            // keep zero-length files out of the graph (nothing to index)
        }
        let mtime = meta.modified().ok().and_then(file_mtime_secs).unwrap_or(0);
        out.push(DiscoveredFile {
            rel_path: rel,
            language,
            size: meta.len() as i64,
            mtime,
        });
    }
    Ok(out)
}

pub fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| normalize(p.to_string_lossy().as_ref()))
        .unwrap_or_else(|_| normalize(path.to_string_lossy().as_ref()))
}

pub fn normalize(p: &str) -> String {
    p.replace('\\', "/")
}

fn file_mtime_secs(t: std::time::SystemTime) -> Option<i64> {
    match t.duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => Some(d.as_secs() as i64),
        Err(e) => Some(-(e.duration().as_secs() as i64)),
    }
}
