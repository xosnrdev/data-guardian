//! Settings management for Data Guardian
//!
//! This module handles configuration settings for the Data Guardian service,
//! providing validation, defaults, and loading from various sources.
//!
//! # Example
//! ```
//! use data_guardian::settings::Settings;
//!
//! let settings = Settings::new().unwrap();
//! assert!(settings.data_limit > 0);
//! ```

use std::path::PathBuf;

use color_eyre::Result;
use config::{Config, Environment, File};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Minimum data limit (1MB)
pub const MIN_DATA_LIMIT: u64 = 1024 * 1024;
/// Minimum check interval (1 second)
pub const MIN_CHECK_INTERVAL: u64 = 1;
/// Minimum persistence interval (10 seconds)
pub const MIN_PERSISTENCE_INTERVAL: u64 = 10;

/// Default data limit (1GB)
pub const DEFAULT_DATA_LIMIT: u64 = 1024 * 1024 * 1024;
/// Default check interval (60 seconds)
pub const DEFAULT_CHECK_INTERVAL: u64 = 60;
/// Default persistence interval (5 minutes)
pub const DEFAULT_PERSISTENCE_INTERVAL: u64 = 300;

/// Errors that can occur during settings operations
#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("Invalid data limit: {0} (min: {1})")]
    InvalidDataLimit(u64, u64),
    #[error("Invalid check interval: {0} seconds (min: {1})")]
    InvalidCheckInterval(u64, u64),
    #[error("Invalid persistence interval: {0} seconds (min: {1})")]
    InvalidPersistenceInterval(u64, u64),
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
}

/// Settings for the Data Guardian service
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct Settings {
    /// Data limit in bytes before triggering alerts
    pub data_limit: u64,
    /// How often to check process data usage (in seconds)
    pub check_interval_seconds: u64,
    /// How often to save usage data to disk (in seconds)
    pub persistence_interval_seconds: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            data_limit: DEFAULT_DATA_LIMIT,
            check_interval_seconds: DEFAULT_CHECK_INTERVAL,
            persistence_interval_seconds: DEFAULT_PERSISTENCE_INTERVAL,
        }
    }
}

impl Settings {
    /// Creates a new Settings instance with values from config files and environment
    ///
    /// # Configuration Sources (in order of precedence)
    /// 1. Environment variables with prefix "DATAGUARDIAN_"
    /// 2. User's local config file (~/.config/DataGuardian/config.toml)
    /// 3. Default values if no config files exist
    ///
    /// # Returns
    /// * `Ok(Settings)` - Valid settings instance
    /// * `Err(SettingsError)` - If loading or validation fails
    pub fn new() -> Result<Self, SettingsError> {
        let mut builder = Config::builder();

        // Add environment variables (highest precedence)
        builder = builder.add_source(Environment::with_prefix("DATAGUARDIAN"));

        // Add user config if it exists
        if let Some(config_path) = get_user_config_path() {
            if config_path.exists() {
                builder = builder.add_source(File::from(config_path));
            }
        }

        // Add default values (lowest precedence)
        builder = builder.set_default("data_limit", DEFAULT_DATA_LIMIT)?;
        builder = builder.set_default("check_interval_seconds", DEFAULT_CHECK_INTERVAL)?;
        builder =
            builder.set_default("persistence_interval_seconds", DEFAULT_PERSISTENCE_INTERVAL)?;

        let settings: Settings = builder.build()?.try_deserialize()?;
        settings.validate()?;
        Ok(settings)
    }

    /// Creates a new Settings instance from a specific configuration file
    ///
    /// This is useful for testing or when you want to load settings from
    /// a non-standard location.
    ///
    /// # Arguments
    /// * `config_path` - Path to the configuration file
    pub fn from_file(config_path: impl AsRef<std::path::Path>) -> Result<Self, SettingsError> {
        let settings: Settings = Config::builder()
            .add_source(File::from(config_path.as_ref()))
            .build()?
            .try_deserialize()?;
        settings.validate()?;
        Ok(settings)
    }

    /// Validates the settings values
    ///
    /// # Returns
    /// * `Ok(())` - If all settings are valid
    /// * `Err(SettingsError)` - If any setting is invalid
    pub fn validate(&self) -> Result<(), SettingsError> {
        if self.data_limit < MIN_DATA_LIMIT {
            return Err(SettingsError::InvalidDataLimit(
                self.data_limit,
                MIN_DATA_LIMIT,
            ));
        }

        if self.check_interval_seconds < MIN_CHECK_INTERVAL {
            return Err(SettingsError::InvalidCheckInterval(
                self.check_interval_seconds,
                MIN_CHECK_INTERVAL,
            ));
        }

        if self.persistence_interval_seconds < MIN_PERSISTENCE_INTERVAL {
            return Err(SettingsError::InvalidPersistenceInterval(
                self.persistence_interval_seconds,
                MIN_PERSISTENCE_INTERVAL,
            ));
        }

        Ok(())
    }
}

/// Gets the path to the user's configuration file
#[inline]
fn get_user_config_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "DataGuardian", "DataGuardian")
        .map(|proj_dirs| proj_dirs.config_dir().join("config.toml"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert_eq!(settings.data_limit, DEFAULT_DATA_LIMIT);
        assert_eq!(settings.check_interval_seconds, DEFAULT_CHECK_INTERVAL);
        assert_eq!(
            settings.persistence_interval_seconds,
            DEFAULT_PERSISTENCE_INTERVAL
        );
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_settings_load() {
        let settings = Settings::new().unwrap();
        assert!(settings.data_limit >= MIN_DATA_LIMIT);
        assert!(settings.check_interval_seconds >= MIN_CHECK_INTERVAL);
        assert!(settings.persistence_interval_seconds >= MIN_PERSISTENCE_INTERVAL);
    }

    #[test]
    fn test_validate_data_limit() {
        // Test invalid data limit
        let settings = Settings {
            data_limit: MIN_DATA_LIMIT - 1,
            ..Default::default()
        };
        assert!(settings.validate().is_err());

        // Test valid data limit
        let settings = Settings {
            data_limit: MIN_DATA_LIMIT,
            ..Default::default()
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_check_interval() {
        // Test invalid check interval
        let settings = Settings {
            check_interval_seconds: MIN_CHECK_INTERVAL - 1,
            ..Default::default()
        };
        assert!(settings.validate().is_err());

        // Test valid check interval
        let settings = Settings {
            check_interval_seconds: MIN_CHECK_INTERVAL,
            ..Default::default()
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_persistence_interval() {
        // Test invalid persistence interval
        let settings = Settings {
            persistence_interval_seconds: MIN_PERSISTENCE_INTERVAL - 1,
            ..Default::default()
        };
        assert!(settings.validate().is_err());

        // Test valid persistence interval
        let settings = Settings {
            persistence_interval_seconds: MIN_PERSISTENCE_INTERVAL,
            ..Default::default()
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_settings_from_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test_config.toml");

        // Create a test config file
        let config_content = format!(
            r#"
            data_limit = {}
            check_interval_seconds = {}
            persistence_interval_seconds = {}
            "#,
            MIN_DATA_LIMIT * 2,
            MIN_CHECK_INTERVAL * 2,
            MIN_PERSISTENCE_INTERVAL * 2
        );
        fs::write(&config_path, config_content).unwrap();

        // Load and verify settings
        let settings = Settings::from_file(&config_path).unwrap();
        assert_eq!(settings.data_limit, MIN_DATA_LIMIT * 2);
        assert_eq!(settings.check_interval_seconds, MIN_CHECK_INTERVAL * 2);
        assert_eq!(
            settings.persistence_interval_seconds,
            MIN_PERSISTENCE_INTERVAL * 2
        );
    }

    #[test]
    fn test_settings_serialization() {
        let settings = Settings::default();
        let serialized = serde_json::to_string(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&serialized).unwrap();
        assert_eq!(settings, deserialized);
    }
}
