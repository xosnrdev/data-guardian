use std::path::PathBuf;

use color_eyre::Result;
use config::{Config, Environment, File};
use directories::ProjectDirs;
use serde::Deserialize;
use thiserror::Error;

const MIN_DATA_LIMIT: u64 = 1024 * 1024; // 1MB
const MIN_CHECK_INTERVAL: u64 = 1; // 1 second
const MIN_PERSISTENCE_INTERVAL: u64 = 10; // 10 seconds

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

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub data_limit: u64,
    pub check_interval_seconds: u64,
    pub persistence_interval_seconds: u64,
}

impl Settings {
    pub fn new() -> Result<Self, SettingsError> {
        let mut builder = Config::builder();
        builder = builder.add_source(File::with_name("config/default"));

        if let Some(config_path) = get_user_config_path() {
            if config_path.exists() {
                builder = builder.add_source(File::from(config_path));
            }
        }

        builder = builder.add_source(Environment::with_prefix("DATAGUARDIAN"));
        let settings: Settings = builder.build()?.try_deserialize()?;
        settings.validate()?;
        Ok(settings)
    }

    fn validate(&self) -> Result<(), SettingsError> {
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
        .map(|proj_dirs| proj_dirs.config_dir().join("local.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_load() {
        let settings = Settings::new().unwrap();
        assert!(settings.data_limit >= MIN_DATA_LIMIT);
        assert!(settings.check_interval_seconds >= MIN_CHECK_INTERVAL);
        assert!(settings.persistence_interval_seconds >= MIN_PERSISTENCE_INTERVAL);
    }

    #[test]
    fn test_settings_validation() {
        let settings = Settings::new().unwrap();
        assert!(settings.validate().is_ok());
    }
}
