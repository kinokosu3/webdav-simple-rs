use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs;
use std::path::Path;

static GLOBAL_CONFIG: Lazy<Config> =
    Lazy::new(|| Config::load("config.toml").expect("Failed to load config"));

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub log: LogConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub prefix: String,
    pub log_level: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    pub file_enabled: bool,
    pub file_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub backend: String,
    pub filesystem: FileSystemConfig,
    pub quark: QuarkConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileSystemConfig {
    pub root_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuarkConfig {
    pub cookie: String,
    pub root_id: String,
}

impl Config {
    /// 从指定路径加载配置文件
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content =
            fs::read_to_string(path).map_err(|e| ConfigError::ReadError(e.to_string()))?;

        toml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// 获取全局配置的引用
    pub fn get() -> &'static Self {
        &GLOBAL_CONFIG
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(String),

    #[error("Failed to parse config file: {0}")]
    ParseError(String),
}
