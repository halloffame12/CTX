//! Language abstraction: every supported grammar implements `LanguageParser`.

pub mod golang;
pub mod python;
pub mod resolve;
pub mod rustlang;
pub mod traits;
pub mod typescript;
pub mod util;

use std::path::Path;

use crate::lang::LanguageId;
use traits::LanguageParser;

pub use traits::{Dependency, DependencyType, ParsedFile, ResolvedDependency, Symbol, SymbolKind};

/// Build a parser for the given language. Parsers are cheap and re-usable.
pub fn parser_for(language: LanguageId, root: &Path) -> Box<dyn LanguageParser> {
    match language {
        LanguageId::TypeScript => Box::new(typescript::JsParser::new(LanguageId::TypeScript)),
        LanguageId::JavaScript => Box::new(typescript::JsParser::new(LanguageId::JavaScript)),
        LanguageId::Python => Box::new(python::PythonParser),
        LanguageId::Rust => Box::new(rustlang::RustParser),
        LanguageId::Go => Box::new(golang::GoParser::detect(root)),
    }
}

/// Convenience wrapper used by search/context/diff paths.
pub fn parse_source(
    language: LanguageId,
    source: &str,
    current_rel: &str,
    root: &Path,
) -> crate::errors::CtxResult<ParsedFile> {
    let parser = parser_for(language, root);
    parser.parse(source, current_rel, root)
}

pub fn skeletonize(
    language: LanguageId,
    source: &str,
    root: &Path,
) -> crate::errors::CtxResult<String> {
    let parser = parser_for(language, root);
    parser.skeleton(source)
}
