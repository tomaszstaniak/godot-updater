use crate::platform;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub channels: ChannelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub install_dir: PathBuf,
    pub edition: String,
    pub download_templates: bool,
    pub check_on_startup: bool,
    #[serde(default)]
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub stable: bool,
    pub dev: bool,
    pub lts: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            general: GeneralConfig {
                install_dir: platform::default_install_dir(),
                edition: "both".to_string(),
                download_templates: false,
                check_on_startup: true,
                theme: "Default".to_string(),
            },
            channels: ChannelConfig {
                stable: true,
                dev: false,
                lts: true,
            },
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        platform::config_dir().join("config.toml")
    }

    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn install_dir(&self) -> &Path {
        &self.general.install_dir
    }
}
