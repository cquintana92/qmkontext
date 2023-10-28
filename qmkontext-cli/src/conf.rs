use config::{Config as CConfig, ConfigError, File};
use std::path::Path;

const DEFAULT_FILE_NAME: &str = "config.toml";
const DEFAULT_INSTALL_CONFIG_PATH: &str = "/etc/qmkontext";

#[cfg(debug_assertions)]
fn default_log_level() -> String {
    "debug".to_string()
}

#[cfg(not(debug_assertions))]
fn default_log_level() -> String {
    "info".to_string()
}

fn default_debug_mode() -> bool {
    false
}

fn default_usage() -> u16 {
    0x61
}

fn default_usage_page() -> u16 {
    0xFF60
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct KeyboardConfig {
    pub vendor_id: u16,
    pub product_id: u16,
    #[serde(default = "default_usage")]
    pub usage: u16,
    #[serde(default = "default_usage_page")]
    pub usage_page: u16,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CurrentProgramConfig {
    pub enable: bool,
    pub command_id: u8,
    pub interval_seconds: u16,
    pub default_value: u8,
    #[serde(default)]
    pub mappings: Vec<CurrentProgramMapping>,
    pub use_lowercase: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CurrentProgramMapping {
    pub key: String,
    pub value: u8,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CustomCommandConfig {
    pub command: String,
    pub command_id: u8,
    pub interval_seconds: u16,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_debug_mode")]
    pub debug_mode: bool,
    pub keyboard: KeyboardConfig,
    pub current_program: CurrentProgramConfig,
    #[serde(default)]
    pub custom_commands: Vec<CustomCommandConfig>,
}

impl Config {
    pub fn new(path: Option<String>) -> Result<Config, ConfigError> {
        let mut c = CConfig::new();

        let config_file_path = Self::get_config_file_path(path)?;
        c.merge(File::with_name(&config_file_path).required(true))?;

        let parsed: Config = c.try_into()?;
        Ok(parsed)
    }

    fn get_config_file_path(cli_path: Option<String>) -> Result<String, ConfigError> {
        let path = match cli_path {
            Some(p) => {
                if file_exists(&p) {
                    Ok(p)
                } else {
                    Err(ConfigError::NotFound(format!("Could not find file {p}")))
                }
            }
            None => {
                // Config file path not specified. Trying local DEFAULT_FILE_NAME
                if file_exists(DEFAULT_FILE_NAME) {
                    Ok(DEFAULT_FILE_NAME.to_string())
                } else {
                    let default_config_file_path =
                        format!("{DEFAULT_INSTALL_CONFIG_PATH}/{DEFAULT_FILE_NAME}");
                    if file_exists(&default_config_file_path) {
                        Ok(default_config_file_path)
                    } else {
                        Err(ConfigError::Message(
                            "Could not find any config file to be used".to_string(),
                        ))
                    }
                }
            }
        }?;

        let as_path = Path::new(&path);
        let canonical_path = as_path.canonicalize().map_err(|e| {
            ConfigError::Message(format!("Cannot get config file path {}: {:?}", path, e))
        })?;
        info!("Using config file {}", canonical_path.display());
        Ok(path)
    }
}

fn file_exists(path: &str) -> bool {
    match std::fs::metadata(path) {
        Ok(m) => m.is_file(),
        Err(_) => false,
    }
}
