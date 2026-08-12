use std::path::Path;

/// A cursor offset/span merged from tree-sitter's grammar-neutral terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct Span {
    pub start_byte: usize,
    pub end_byte: usize,
    #[serde(rename = "startLine")]
    pub start_line: u32,
    #[serde(rename = "endLine")]
    pub end_line: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageId {
    TypeScript,
    JavaScript,
    Python,
    Rust,
    Go,
}

impl LanguageId {
    pub fn as_str(&self) -> &'static str {
        match self {
            LanguageId::TypeScript => "typescript",
            LanguageId::JavaScript => "javascript",
            LanguageId::Python => "python",
            LanguageId::Rust => "rust",
            LanguageId::Go => "go",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            LanguageId::TypeScript => "TypeScript",
            LanguageId::JavaScript => "JavaScript",
            LanguageId::Python => "Python",
            LanguageId::Rust => "Rust",
            LanguageId::Go => "Go",
        }
    }

    pub fn all() -> &'static [LanguageId] {
        &[
            LanguageId::TypeScript,
            LanguageId::JavaScript,
            LanguageId::Python,
            LanguageId::Rust,
            LanguageId::Go,
        ]
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<LanguageId> {
        match s.to_ascii_lowercase().as_str() {
            "typescript" | "ts" | "tsx" => Some(LanguageId::TypeScript),
            "javascript" | "js" | "jsx" | "mjs" | "cjs" => Some(LanguageId::JavaScript),
            "python" | "py" => Some(LanguageId::Python),
            "rust" | "rs" => Some(LanguageId::Rust),
            "go" | "golang" => Some(LanguageId::Go),
            _ => None,
        }
    }

    pub fn from_extension(ext: &str) -> Option<LanguageId> {
        match ext.to_ascii_lowercase().as_str() {
            "ts" | "tsx" | "mts" | "cts" => Some(LanguageId::TypeScript),
            "js" | "jsx" | "mjs" | "cjs" => Some(LanguageId::JavaScript),
            "py" | "pyi" | "pyw" => Some(LanguageId::Python),
            "rs" => Some(LanguageId::Rust),
            "go" => Some(LanguageId::Go),
            _ => None,
        }
    }

    /// File extensions (without dot) this language is compiled from.
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            LanguageId::TypeScript => &["ts", "tsx", "mts", "cts"],
            LanguageId::JavaScript => &["js", "jsx", "mjs", "cjs"],
            LanguageId::Python => &["py", "pyi", "pyw"],
            LanguageId::Rust => &["rs"],
            LanguageId::Go => &["go"],
        }
    }
}

pub fn language_of_path(path: &Path) -> Option<LanguageId> {
    let ext = path.extension()?.to_str()?;
    LanguageId::from_extension(ext)
}
