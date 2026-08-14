//! Command implementations shared between the CLI and the MCP server.

pub mod benchmark;
pub mod changed;
pub mod context;
pub mod deps;
pub mod diff;
pub mod doctor;
pub mod impact;
pub mod init;
pub mod mcp;
pub mod search;
pub mod skeleton;
pub mod stats;
pub mod symbol;
pub mod watch;

use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::errors::{CtxError, CtxResult};
use crate::git::GitRepo;
use crate::graph::database::Database;

#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub config: Config,
    pub db: Database,
    pub git: Option<GitRepo>,
}

impl Project {
    pub fn open(cwd: &Path, root_override: Option<&Path>) -> CtxResult<Project> {
        let root = match root_override {
            Some(r) => r.to_path_buf(),
            None => discover_root(cwd)?,
        };
        // Never create a project tree under a path that does not exist
        // (e.g. a mistyped `-R`). Callers that are allowed to initialize a
        // directory (MCP fallback, `ctx init`) must pass an existing path.
        if !root.is_dir() {
            return Err(CtxError::NotInitialized(root.display().to_string()));
        }
        let config = Config::load(&root)?;
        let db = Database::open(&root)?;
        let git = GitRepo::discover(&root)?;
        Ok(Project {
            root,
            config,
            db,
            git,
        })
    }

    pub fn require_initialized(&self) -> CtxResult<()> {
        if Database::exists(&self.root) {
            Ok(())
        } else {
            Err(CtxError::NotInitialized(self.root.display().to_string()))
        }
    }
}

/// Walk up from `cwd` to find the project root (a directory containing .ctx).
pub fn discover_root(cwd: &Path) -> CtxResult<PathBuf> {
    let mut dir = Some(cwd);
    while let Some(d) = dir {
        if d.join(".ctx").is_dir() || d.join(".ctx").is_file() {
            // .ctx could be a file marker; canonicalize
            return Ok(d.to_path_buf());
        }
        dir = d.parent();
    }
    Err(CtxError::NotInitialized(cwd.display().to_string()))
}

/// Normalise a user-supplied path to a project-relative path. Accepts
/// absolute paths and repo-relative paths and ensures the result stays
/// inside the project. Relative arguments are interpreted relative to the
/// current directory when possible, otherwise as project-relative.
///
/// Security: any argument that lexically escapes the project root (e.g.
/// `../../etc/passwd`) is rejected with [`CtxError::PathOutsideRoot`] and is
/// never turned into a `root.join(...)` path.
pub fn normalize_rel_path(root: &Path, arg: &str) -> CtxResult<String> {
    // Treat backslashes as path separators on every platform so that
    // Windows-style traversal (..\..\etc\passwd) is rejected consistently.
    let norm_arg = arg.replace('\\', "/");
    let p = Path::new(&norm_arg);
    let candidates: Vec<PathBuf> = if p.is_absolute() {
        vec![p.to_path_buf()]
    } else {
        vec![
            std::env::current_dir()
                .unwrap_or_else(|_| root.to_path_buf())
                .join(p),
            root.join(p),
        ]
    };
    for cand in candidates {
        if let Some(rel) = lexically_rel(root, &cand) {
            return Ok(rel);
        }
    }
    Err(CtxError::PathOutsideRoot(arg.to_string()))
}

/// Resolve `full` to a project-relative path only when it lexically lives
/// inside `root` (after normalising `.`/`..` segments). Returns `None` when
/// the path is outside the root or escapes it.
fn lexically_rel(root: &Path, full: &Path) -> Option<String> {
    use std::path::Component;
    let parts = |p: &Path| {
        let mut stack: Vec<std::ffi::OsString> = Vec::new();
        for c in p.components() {
            match c {
                Component::Prefix(_) | Component::RootDir => stack.clear(),
                Component::CurDir => {}
                Component::ParentDir => {
                    stack.pop();
                }
                Component::Normal(s) => stack.push(s.to_owned()),
            }
        }
        stack
    };
    let root_parts = parts(root);
    let full_parts = parts(full);
    if full_parts.len() < root_parts.len() || full_parts[..root_parts.len()] != root_parts[..] {
        return None;
    }
    let rel: Vec<String> = full_parts[root_parts.len()..]
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    Some(rel.join("/"))
}

/// Resolve a path argument to a file record (exact path match, else LIKE).
pub fn resolve_file(project: &Project, arg: &str) -> CtxResult<crate::graph::database::FileRecord> {
    if let Some(f) = project.db.file_by_path(arg)? {
        return Ok(f);
    }
    for f in project.db.files_like(arg, 20)? {
        if f.path.contains(arg) {
            return Ok(f);
        }
    }
    Err(CtxError::Other(format!(
        "no indexed file matching `{arg}`; run `ctx init` after adding it"
    )))
}

pub fn project_summary(project: &Project) -> CtxResult<serde_json::Value> {
    let (files, symbols, deps) = project.db.stats()?;
    let mut languages: Vec<String> = Vec::new();
    for f in project.db.all_files()? {
        if let Some(lang) = f.language
            && !languages.contains(&lang)
        {
            languages.push(lang);
        }
    }
    Ok(serde_json::json!({
        "root": project.root.display().to_string(),
        "git": project.git.as_ref().map(|g| g.root.display().to_string()),
        "files": files,
        "symbols": symbols,
        "dependencies": deps,
        "languages": languages,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_rejects_escape_attempts() {
        let root = Path::new("/repo/proj");
        // absolute escapes
        assert!(normalize_rel_path(root, "/etc/passwd").is_err());
        // relative `..` traversal
        assert!(normalize_rel_path(root, "../../etc/passwd").is_err());
        assert!(normalize_rel_path(root, "src/../../../../etc/passwd").is_err());
        // windows-style separators
        assert!(normalize_rel_path(root, r"..\..\etc\passwd").is_err());
    }

    #[test]
    fn normalize_accepts_internal_paths() {
        let root = Path::new("/repo/proj");
        // project-relative
        assert_eq!(
            normalize_rel_path(root, "src/app.ts").unwrap(),
            "src/app.ts"
        );
        // dot segments collapse
        assert_eq!(
            normalize_rel_path(root, "./src/./app.ts").unwrap(),
            "src/app.ts"
        );
        // parent-dot within root collapses (a/../b -> b) but stays in-root
        assert_eq!(
            normalize_rel_path(root, "src/../util.ts").unwrap(),
            "util.ts"
        );
    }
}
