use std::path::PathBuf;

use color_eyre::Result;
use config::{Config, Environment, File};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MIN_DATA_LIMIT: u64 = 1024 * 1024;
pub const MIN_CHECK_INTERVAL: u64 = 1;
pub const MIN_PERSISTENCE_INTERVAL: u64 = 10;

pub const DEFAULT_DATA_LIMIT: u64 = 1024 * 1024 * 1024;
pub const DEFAULT_CHECK_INTERVAL: u64 = 60;
pub const DEFAULT_PERSISTENCE_INTERVAL: u64 = 300;

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

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct Settings {
    pub data_limit: u64,
    pub check_interval_seconds: u64,
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
    pub fn new() -> Result<Self, SettingsError> {
        let mut builder = Config::builder();

        builder = builder.add_source(Environment::with_prefix("DATAGUARDIAN"));

        if let Some(config_path) = get_user_config_path() {
            if config_path.exists() {
                builder = builder.add_source(File::from(config_path));
            }
        }

        builder = builder.set_default("data_limit", DEFAULT_DATA_LIMIT)?;
        builder = builder.set_default("check_interval_seconds", DEFAULT_CHECK_INTERVAL)?;
        builder =
            builder.set_default("persistence_interval_seconds", DEFAULT_PERSISTENCE_INTERVAL)?;

        let settings: Settings = builder.build()?.try_deserialize()?;
        settings.validate()?;
        Ok(settings)
    }

    #[cfg(test)]
    pub fn from_file(config_path: impl AsRef<std::path::Path>) -> Result<Self, SettingsError> {
        let settings: Settings = Config::builder()
            .add_source(File::from(config_path.as_ref()))
            .build()?
            .try_deserialize()?;
        settings.validate()?;
        Ok(settings)
    }

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
        let settings = Settings {
            data_limit: MIN_DATA_LIMIT - 1,
            ..Default::default()
        };
        assert!(settings.validate().is_err());

        let settings = Settings {
            data_limit: MIN_DATA_LIMIT,
            ..Default::default()
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_check_interval() {
        let settings = Settings {
            check_interval_seconds: MIN_CHECK_INTERVAL - 1,
            ..Default::default()
        };
        assert!(settings.validate().is_err());

        let settings = Settings {
            check_interval_seconds: MIN_CHECK_INTERVAL,
            ..Default::default()
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_persistence_interval() {
        let settings = Settings {
            persistence_interval_seconds: MIN_PERSISTENCE_INTERVAL - 1,
            ..Default::default()
        };
        assert!(settings.validate().is_err());

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
