use thiserror::Error;

#[derive(Debug, Error)]
pub enum CtxError {
    #[error("no `ctx` index found at {0}/.ctx — run `ctx init` first")]
    NotInitialized(String),
    #[error(
        "path `{0}` is outside the project root — pass a project-relative path or set the root with -R"
    )]
    PathOutsideRoot(String),
    #[error("no supported language parser for `{0}`")]
    UnsupportedLanguage(String),
    #[error("git error: {0}")]
    Git(String),
    #[error("SQLite error: {0}")]
    Sqlite(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("parse error in {0}: {1}")]
    Parse(String, String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("{0}")]
    Other(String),
}

impl From<rusqlite::Error> for CtxError {
    fn from(e: rusqlite::Error) -> Self {
        CtxError::Sqlite(e.to_string())
    }
}

impl From<std::io::Error> for CtxError {
    fn from(e: std::io::Error) -> Self {
        CtxError::Io(e.to_string())
    }
}

pub type CtxResult<T> = Result<T, CtxError>;
