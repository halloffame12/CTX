//! Incremental codebase indexing.

pub mod hasher;
pub mod incremental;
pub mod scanner;

pub use incremental::{IndexReport, ParsedOutcome, index_single_file, remove_file};
