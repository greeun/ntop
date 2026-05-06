// Configuration management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct GeneralConfig {
    pub refresh_interval: u64,
    pub default_signal: String,
    pub graceful_timeout: u64,
    pub confirm_before_kill: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            refresh_interval: 1,
            default_signal: "SIGTERM".to_string(),
            graceful_timeout: 10,
            confirm_before_kill: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct DisplayConfig {
    pub show_tree: bool,
    pub color_theme: String,
    pub mask_env_values: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_tree: true,
            color_theme: "auto".to_string(),
            mask_env_values: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct FilterConfig {
    pub include_bun: bool,
    pub include_tsx: bool,
    pub include_ts_node: bool,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            include_bun: false,
            include_tsx: false,
            include_ts_node: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub display: DisplayConfig,
    pub filter: FilterConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            display: DisplayConfig::default(),
            filter: FilterConfig::default(),
        }
    }
}

impl Config {
    /// Returns the path to the config file: `~/.config/nsm/config.toml`
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("nsm").join("config.toml"))
    }

    /// Load config from `~/.config/nsm/config.toml`, returning defaults if the file
    /// does not exist or cannot be parsed.
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };

        match std::fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Returns `refresh_interval` as a `Duration`.
    pub fn refresh_duration(&self) -> Duration {
        Duration::from_secs(self.general.refresh_interval)
    }

    /// Returns `graceful_timeout` as a `Duration`.
    pub fn graceful_duration(&self) -> Duration {
        Duration::from_secs(self.general.graceful_timeout)
    }
}
