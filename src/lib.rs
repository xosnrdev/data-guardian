//! Data Guardian - A system service for monitoring and optimizing app data usage
//!
//! This library provides functionality for monitoring process data usage
//! and alerting when configurable thresholds are exceeded.

pub mod compression;
pub mod notification;
pub mod settings;

pub use notification::alert_user;
pub use settings::Settings;
