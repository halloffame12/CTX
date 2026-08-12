//! Skeleton generation orchestration + honest token estimation.

use std::path::Path;

use crate::errors::CtxResult;
use crate::lang::LanguageId;
use crate::parser::skeletonize;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkeletonStats {
    pub original_bytes: usize,
    pub skeleton_bytes: usize,
    /// Estimated token count of the original source. Simple heuristic
    /// (bytes/4) — labelled as an estimate, not a precise count.
    pub original_tokens: usize,
    pub skeleton_tokens: usize,
    pub reduction_pct: f64,
    pub estimate: bool,
}

impl SkeletonStats {
    pub fn compute(original: &str, skeleton: &str) -> SkeletonStats {
        let original_bytes = original.len();
        let skeleton_bytes = skeleton.len();
        let original_tokens = estimate_tokens(original);
        let skeleton_tokens = estimate_tokens(skeleton);
        let reduction_pct = if original_tokens == 0 {
            0.0
        } else {
            let kept = skeleton_tokens as f64 / original_tokens as f64;
            ((1.0 - kept) * 100.0).max(0.0)
        };
        SkeletonStats {
            original_bytes,
            skeleton_bytes,
            original_tokens,
            skeleton_tokens,
            reduction_pct,
            estimate: true,
        }
    }
}

/// Heuristic token estimate. This is bytes/4 (a common rough proxy for
/// English-ish text) — clearly not a precise tokenizer. Used only for
/// budgeting and `--stats`; never exported as exact counts.
pub fn estimate_tokens(text: &str) -> usize {
    (text.len() / 4).max(1)
}

pub struct SkeletonResult {
    pub skeleton: String,
    pub stats: SkeletonStats,
}

pub fn skeleton_for(
    root: &Path,
    _rel_path: &str,
    language: LanguageId,
    source: &str,
) -> CtxResult<SkeletonResult> {
    let skeleton = skeletonize(language, source, root)?;
    let stats = SkeletonStats::compute(source, &skeleton);
    Ok(SkeletonResult { skeleton, stats })
}
