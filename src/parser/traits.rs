use crate::lang::{LanguageId, Span};
use std::path::Path;
use tree_sitter::Tree;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Interface,
    Type,
    Enum,
    Constant,
    Variable,
    Struct,
    Trait,
    Module,
    Field,
    Constructor,
    Impl,
}

impl SymbolKind {
    pub const ALL_NAMES: [&'static str; 14] = [
        "function",
        "method",
        "class",
        "interface",
        "type",
        "enum",
        "constant",
        "variable",
        "struct",
        "trait",
        "module",
        "field",
        "constructor",
        "impl",
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::Function => "function",
            SymbolKind::Method => "method",
            SymbolKind::Class => "class",
            SymbolKind::Interface => "interface",
            SymbolKind::Type => "type",
            SymbolKind::Enum => "enum",
            SymbolKind::Constant => "constant",
            SymbolKind::Variable => "variable",
            SymbolKind::Struct => "struct",
            SymbolKind::Trait => "trait",
            SymbolKind::Module => "module",
            SymbolKind::Field => "field",
            SymbolKind::Constructor => "constructor",
            SymbolKind::Impl => "impl",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<SymbolKind> {
        match s.to_ascii_lowercase().as_str() {
            "function" | "fn" => Some(SymbolKind::Function),
            "method" => Some(SymbolKind::Method),
            "class" => Some(SymbolKind::Class),
            "interface" => Some(SymbolKind::Interface),
            "type" | "alias" => Some(SymbolKind::Type),
            "enum" => Some(SymbolKind::Enum),
            "constant" | "const" => Some(SymbolKind::Constant),
            "variable" => Some(SymbolKind::Variable),
            "struct" => Some(SymbolKind::Struct),
            "trait" => Some(SymbolKind::Trait),
            "module" => Some(SymbolKind::Module),
            "field" => Some(SymbolKind::Field),
            "constructor" => Some(SymbolKind::Constructor),
            "impl" => Some(SymbolKind::Impl),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub signature: String,
    pub span: Span,
    /// Fully qualified parent path (e.g. `UserService` for methods).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(default)]
    pub exported: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    /// `import X from/{}` (ES modules)
    Import,
    /// `export ... from "..."` (re-exports)
    Export,
    /// `require(...)` / `import("...")`
    Require,
    /// `use path::...` (Rust)
    Use,
    /// `import` / `from ... import ...` (Python)
    PyImport,
    /// `from ... import`
    #[serde(rename = "from")]
    PyFrom,
    /// Go `import "..."`
    GoImport,
    /// Any other relation we could not classify
    Other,
}

impl DependencyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DependencyType::Import => "import",
            DependencyType::Export => "export",
            DependencyType::Require => "require",
            DependencyType::Use => "use",
            DependencyType::PyImport => "import",
            DependencyType::PyFrom => "import",
            DependencyType::GoImport => "import",
            DependencyType::Other => "other",
        }
    }
}

/// How an import resolved relative to this project.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "lowercase")]
pub enum ResolvedDependency {
    /// Imports code inside the project. Payload is the project-relative path.
    Internal(String),
    /// A package / external dependency (react, tokio, express, ...).
    External(String),
    /// A reference that could not be resolved to a concrete file (kept for
    /// searchability, target_file_id will be NULL).
    Unresolved(String),
}

impl ResolvedDependency {
    pub fn is_internal(&self) -> bool {
        matches!(self, ResolvedDependency::Internal(_))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dependency {
    /// The symbol a `from`/named import brought in, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imported_symbol: Option<String>,
    pub dependency_type: DependencyType,
    /// Raw import text as written in source (e.g. `./models/User`).
    pub source_raw: String,
    pub resolved: ResolvedDependency,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedFile {
    pub language: LanguageId,
    pub symbols: Vec<Symbol>,
    pub dependencies: Vec<Dependency>,
    #[serde(default)]
    pub has_errors: bool,
}

/// Language-specific extraction + skeletonisation backend.
///
/// Implementations are intentionally independent of the rest of the engine:
/// each language knows how to turn source text into symbols, dependencies and
/// a structural skeleton. New languages are added by implementing this trait
/// and registering the parser in [`crate::parser::parsers`].
pub trait LanguageParser: Send + Sync {
    fn language(&self) -> LanguageId;

    /// Parse source and extract all structural information.
    fn parse(
        &self,
        source: &str,
        current_rel: &str,
        root: &Path,
    ) -> crate::errors::CtxResult<ParsedFile>;

    /// Extract symbols from an already parsed tree.
    fn extract_symbols(&self, tree: &Tree, source: &str) -> crate::errors::CtxResult<Vec<Symbol>>;

    /// Extract imports / dependencies from an already parsed tree.
    fn extract_dependencies(
        &self,
        tree: &Tree,
        source: &str,
        current_rel: &str,
        root: &Path,
    ) -> crate::errors::CtxResult<Vec<Dependency>>;

    /// Produce a body-less skeleton that preserves all structural meaning.
    fn skeleton(&self, source: &str, current_rel: &str) -> crate::errors::CtxResult<String>;
}
