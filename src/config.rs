use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct Config {
    pub index: IndexConfig,
    pub context: ContextConfig,
    pub watch: WatchConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct IndexConfig {
    pub exclude: Vec<String>,
    pub max_file_size: u64,
    pub follow_symlinks: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ContextConfig {
    pub max_tokens: usize,
    pub max_files: usize,
    pub include_bodies: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct WatchConfig {
    pub enabled: bool,
    pub debounce_ms: u64,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            exclude: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
                ".ctx".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".cache".to_string(),
                "vendor".to_string(),
                "coverage".to_string(),
                "__pycache__".to_string(),
                ".next".to_string(),
                ".nuxt".to_string(),
                ".venv".to_string(),
                "venv".to_string(),
                ".venvs".to_string(),
                "env".to_string(),
                ".env".to_string(),
                "Pods".to_string(),
                ".output".to_string(),
            ],
            max_file_size: 1024 * 1024 * 2,
            follow_symlinks: false,
        }
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tokens: 12_000,
            max_files: 25,
            include_bodies: false,
        }
    }
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_ms: 200,
        }
    }
}

impl Config {
    pub fn load(root: &Path) -> crate::errors::CtxResult<Config> {
        let path = root.join(".ctx").join("config.toml");
        if !path.exists() {
            return Ok(Config::default());
        }
        let raw = std::fs::read_to_string(&path)?;
        let cfg: Config = toml::from_str(&raw).map_err(|e| {
            crate::errors::CtxError::Other(format!("invalid {}: {e}", path.display()))
        })?;
        Ok(cfg)
    }
}
