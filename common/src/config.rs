use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CONFIG_TEMPLATE: &str = include_str!("../Config.toml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub server: ServerSettings,
    pub paths: PathConfig,
    pub database: DatabaseConfig,
    #[serde(rename = "banners")]
    pub banners: Vec<Banner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub dns: String,
    pub http_port: u16,
    pub game_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    pub data_dir: PathBuf,
    pub excel_data: PathBuf,
    pub static_data: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Banner {
    pub id: i32,
    pub open_time: String,
    pub close_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
}

impl ServerConfig {
    pub fn ensure_exists(path: &PathBuf) -> anyhow::Result<()> {
        if path.exists() {
            tracing::debug!("Config file already exists: {}", path.display());
            return Ok(());
        }

        tracing::info!("Creating config from template: {}", path.display());

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create config directory {}: {}",
                    parent.display(),
                    e
                )
            })?;
        }

        std::fs::write(path, CONFIG_TEMPLATE).map_err(|e| {
            anyhow::anyhow!(
                "Failed to write config template to {}: {}",
                path.display(),
                e
            )
        })?;

        tracing::info!("Created config file at: {}", path.display());
        Ok(())
    }

    pub fn load(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let config_path = path.into();

        let content = std::fs::read_to_string(&config_path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to read config file '{}': {}",
                config_path.display(),
                e
            )
        })?;

        toml::from_str(&content).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse config file '{}': {}",
                config_path.display(),
                e
            )
        })
    }

    pub fn load_or_create(path: &PathBuf) -> anyhow::Result<Self> {
        Self::ensure_exists(path)?;
        Self::load(path)
    }

    pub fn resolve_paths(&mut self, config_dir: &Path) -> anyhow::Result<()> {
        if self.database.path.is_relative() {
            self.database.path = config_dir.join(&self.database.path);
        }
        if self.paths.data_dir.is_relative() {
            self.paths.data_dir = config_dir.join(&self.paths.data_dir);
        }
        if self.paths.excel_data.is_relative() {
            self.paths.excel_data = config_dir.join(&self.paths.excel_data);
        }
        if self.paths.static_data.is_relative() {
            self.paths.static_data = config_dir.join(&self.paths.static_data);
        }
        Ok(())
    }

    pub fn validate_paths(&self) -> anyhow::Result<()> {
        if !self.paths.data_dir.exists() {
            anyhow::bail!(
                "Data directory not found: {}",
                self.paths.data_dir.display()
            );
        }
        if !self.paths.excel_data.exists() {
            anyhow::bail!(
                "Excel data directory not found: {}",
                self.paths.excel_data.display()
            );
        }
        if !self.paths.static_data.exists() {
            anyhow::bail!(
                "Static data directory not found: {}",
                self.paths.static_data.display()
            );
        }

        // Create database directory if needed
        if let Some(parent) = self.database.path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create database directory {}: {}",
                    parent.display(),
                    e
                )
            })?;
        }

        Ok(())
    }
}
