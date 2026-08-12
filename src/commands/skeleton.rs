use std::path::Path;

use crate::commands::normalize_rel_path;
use crate::context::skeleton::skeleton_for;
use crate::errors::CtxResult;
use crate::lang::language_of_path;
use crate::output::{Term, emit_json};

pub fn cmd_skeleton(root: &Path, path: &str, stats: bool, t: &Term) -> CtxResult<()> {
    let rel = normalize_rel_path(root, path)?;
    let full = root.join(&rel);
    let language = language_of_path(Path::new(&rel))
        .ok_or_else(|| crate::errors::CtxError::UnsupportedLanguage(rel.clone()))?;
    let source = std::fs::read_to_string(&full)?;
    let result = skeleton_for(root, &rel, language, &source)?;

    if t.is_json() {
        let mut v = serde_json::json!({
            "path": rel,
            "language": language.as_str(),
            "skeleton": result.skeleton,
        });
        if stats {
            v["stats"] = serde_json::to_value(&result.stats)?;
        }
        emit_json(&v);
        return Ok(());
    }

    if stats {
        let s = &result.stats;
        println!("Original:");
        println!("  ~{} tokens (estimate: bytes/4)", s.original_tokens);
        println!("Skeleton:");
        println!("  ~{} tokens (estimate: bytes/4)", s.skeleton_tokens);
        println!("Reduction: {:.1}%", s.reduction_pct);
        println!("Bytes: {} → {}", s.original_bytes, s.skeleton_bytes);
        println!();
    }
    println!("{}", result.skeleton);
    Ok(())
}
