use std::process::Command;

use thiserror::Error;
use tracing::{error, info};

#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("Failed to show notification: {0}")]
    ShowError(String),
}

/// Send a notification to the user about data usage.
///
/// # Arguments
/// * `app` - The name of the application that exceeded its data limit
///
/// # Platform Support
/// * Linux: Uses `notify-rust`
/// * macOS: Uses `osascript`
/// * Windows: Uses `notify-rust`
///
/// # Returns
/// * `Ok(())` if the notification was sent successfully
/// * `Err(NotificationError)` if the notification failed
#[cfg(target_os = "linux")]
pub fn alert_user(app: &str) -> Result<(), NotificationError> {
    info!("Sending notification for app: {}", app);
    notify_rust::Notification::new()
        .summary("Data Limit Exceeded")
        .body(&format!(
            "Application '{}' has exceeded the data threshold.",
            app
        ))
        .show()
        .map_err(|e| NotificationError::ShowError(e.to_string()))
}

#[cfg(target_os = "macos")]
pub fn alert_user(app: &str) -> Result<(), NotificationError> {
    info!("Sending notification for app: {}", app);

    let escaped_msg = format!("Application {} has exceeded the data threshold", app)
        .replace('\\', "\\\\")
        .replace('"', "\\\"");

    let script = format!(
        "display notification \"{}\" with title \"Data Guardian\"",
        escaped_msg
    );

    match Command::new("osascript").arg("-e").arg(script).output() {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            let err = String::from_utf8_lossy(&output.stderr);
            error!("Notification error: {}", err);
            Err(NotificationError::ShowError(err.to_string()))
        }
        Err(e) => {
            error!("Failed to execute osascript: {}", e);
            Err(NotificationError::ShowError(e.to_string()))
        }
    }
}

#[cfg(target_os = "windows")]
pub fn alert_user(app: &str) -> Result<(), NotificationError> {
    info!("Sending notification for app: {}", app);
    notify_rust::Notification::new()
        .summary("Data Guardian")
        .body(&format!(
            "Application '{}' has exceeded the data threshold.",
            app
        ))
        .show()
        .map_err(|e| NotificationError::ShowError(e.to_string()))
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn alert_user(_app: &str) -> Result<(), NotificationError> {
    Err(NotificationError::ShowError(
        "Platform not supported".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification() {
        let result = alert_user("test_app");

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        assert!(
            result.is_ok(),
            "Notification should succeed on supported platforms"
        );

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        assert!(matches!(result, Err(NotificationError::ShowError(_))));
    }

    #[test]
    fn test_notification_special_chars() {
        const TEST_CASES: [&str; 6] = [
            r#"test"app"#,
            r#"test'app"#,
            r#"test\app"#,
            r#"test/app"#,
            r#"test app"#,
            r#"test_app"#,
        ];

        for app in TEST_CASES {
            let result = alert_user(app);
            assert!(result.is_ok(), "Failed to handle special chars in: {}", app);
        }
    }
}
