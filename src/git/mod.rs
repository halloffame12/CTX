//! Git integration via the `git` binary (safe, no repos touched).

pub mod changed;
pub mod diff;

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::errors::{CtxError, CtxResult};

#[derive(Debug, Clone)]
pub struct GitRepo {
    pub root: PathBuf,
}

impl GitRepo {
    /// Find the git working tree root for a directory.
    pub fn discover(dir: &Path) -> CtxResult<Option<GitRepo>> {
        let out = Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .current_dir(dir)
            .output();
        match out {
            Ok(o) if o.status.success() => {
                let text = String::from_utf8_lossy(&o.stdout);
                let root = PathBuf::from(text.trim());
                if root.is_dir() {
                    return Ok(Some(GitRepo { root }));
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    pub fn is_git(&self) -> bool {
        self.root.join(".git").is_dir()
    }

    pub fn run(&self, args: &[&str]) -> CtxResult<String> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.root);
        cmd.args(args);
        let out = cmd
            .output()
            .map_err(|e| CtxError::Git(format!("failed to run git: {e}")))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
            return Err(CtxError::Git(if err.is_empty() {
                format!("git {} failed", args.join(" "))
            } else {
                format!("git {}: {err}", args.join(" "))
            }));
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    /// Read a file's content at `rev` (a ref or commit). Empty string if not
    /// present at that revision.
    pub fn show(&self, rev: &str, path: &str) -> CtxResult<String> {
        let spec = format!("{rev}:{path}");
        let out = Command::new("git")
            .arg("cat-file")
            .arg("-p")
            .arg(&spec)
            .current_dir(&self.root)
            .output()
            .map_err(|e| CtxError::Git(format!("git cat-file failed: {e}")))?;
        if !out.status.success() {
            return Ok(String::new());
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}
