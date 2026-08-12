//! Context engine: deterministic relevance selection for AI agents.

pub mod builder;
pub mod ranking;
pub mod resolver;
pub mod skeleton;

pub use builder::{
    ContextPackage, RelevantFile, RelevantSymbol, build_context, build_context_with,
};
pub use skeleton::{SkeletonStats, estimate_tokens, skeleton_for};
