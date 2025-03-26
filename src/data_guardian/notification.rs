use std::collections::HashMap;
#[cfg(target_os = "macos")]
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use thiserror::Error;
use tracing::{debug, error, info};

pub const DEFAULT_COOLDOWN: Duration = Duration::from_secs(300);

#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("Failed to show notification: {0}")]
    ShowError(String),
    #[error("Notification in cooldown")]
    Cooldown,
    #[error("Failed to acquire lock")]
    LockError,
}

#[derive(Debug)]
pub struct NotificationManager {
    cooldown: Duration,
    last_notifications: Mutex<HashMap<String, Instant>>,
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new(DEFAULT_COOLDOWN)
    }
}

impl NotificationManager {
    pub fn new(cooldown: Duration) -> Self {
        Self {
            cooldown,
            last_notifications: Mutex::new(HashMap::new()),
        }
    }

    pub fn is_in_cooldown(&self, app: &str) -> Result<bool, NotificationError> {
        let now = Instant::now();
        let last_notifications = self
            .last_notifications
            .lock()
            .map_err(|_| NotificationError::LockError)?;

        Ok(last_notifications
            .get(app)
            .is_some_and(|last_time| now.duration_since(*last_time) < self.cooldown))
    }

    fn update_last_notification(&self, app: &str) -> Result<(), NotificationError> {
        let mut last_notifications = self
            .last_notifications
            .lock()
            .map_err(|_| NotificationError::LockError)?;

        last_notifications.insert(app.to_string(), Instant::now());
        Ok(())
    }

    pub fn alert_user(&self, app: &str) -> Result<(), NotificationError> {
        if self.is_in_cooldown(app)? {
            debug!(%app, "Skipping notification due to cooldown");
            return Err(NotificationError::Cooldown);
        }

        self.update_last_notification(app)?;

        match self.send_platform_notification(app) {
            Ok(()) => Ok(()),
            Err(e) => {
                debug!(%app, "Notification failed but keeping cooldown");
                Err(e)
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn send_platform_notification(&self, app: &str) -> Result<(), NotificationError> {
        info!("Sending notification for app: {}", app);
        notify_rust::Notification::new()
            .summary("Data Limit Exceeded")
            .body(&format!(
                "Application '{}' has exceeded the data threshold.",
                app
            ))
            .show()
            .map(|_| ())
            .map_err(|e| NotificationError::ShowError(e.to_string()))
    }

    #[cfg(target_os = "macos")]
    fn send_platform_notification(&self, app: &str) -> Result<(), NotificationError> {
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
    fn send_platform_notification(&self, app: &str) -> Result<(), NotificationError> {
        info!("Sending notification for app: {}", app);
        notify_rust::Notification::new()
            .summary("Data Guardian")
            .body(&format!(
                "Application '{}' has exceeded the data threshold.",
                app
            ))
            .show()
            .map(|_| ())
            .map_err(|e| NotificationError::ShowError(e.to_string()))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    fn send_platform_notification(&self, _app: &str) -> Result<(), NotificationError> {
        Err(NotificationError::ShowError(
            "Platform not supported".to_string(),
        ))
    }
}

static NOTIFICATION_MANAGER: OnceLock<NotificationManager> = OnceLock::new();

pub fn alert_user(app: &str) -> Result<(), NotificationError> {
    let manager = NOTIFICATION_MANAGER.get_or_init(NotificationManager::default);
    manager.alert_user(app)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Barrier};
    use std::thread;

    use super::*;

    const TEST_COOLDOWN: Duration = Duration::from_secs(1);
    const THREAD_COUNT: usize = 4;
    const POLL_INTERVAL: Duration = Duration::from_millis(10);
    const MAX_WAIT: Duration = Duration::from_secs(5);

    fn is_ci_environment() -> bool {
        std::env::var("CI").is_ok()
    }

    fn wait_for_cooldown_state(
        manager: &NotificationManager,
        app: &str,
        expected_in_cooldown: bool,
    ) {
        let start = Instant::now();
        let mut attempts = 0;

        while start.elapsed() < MAX_WAIT {
            attempts += 1;
            match manager.is_in_cooldown(app) {
                Ok(in_cooldown) => {
                    debug!(
                        %app,
                        actual = in_cooldown,
                        expected = expected_in_cooldown,
                        attempts,
                        "Checking cooldown state"
                    );
                    if in_cooldown == expected_in_cooldown {
                        return;
                    }
                }
                Err(e) => {
                    debug!(%app, ?e, "Error checking cooldown state");
                }
            }
            thread::sleep(POLL_INTERVAL);
        }

        panic!(
            "Timeout waiting for cooldown state to become {} after {} attempts",
            expected_in_cooldown, attempts
        );
    }

    #[test]
    fn test_notification() {
        let manager = NotificationManager::new(TEST_COOLDOWN);
        let result = manager.alert_user("test_app");

        if is_ci_environment() {
            #[cfg(target_os = "linux")]
            assert!(
                matches!(result, Err(NotificationError::ShowError(_))),
                "Notification should fail gracefully in Linux CI environment"
            );

            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
                assert!(
                    result.is_ok() || matches!(result, Err(NotificationError::ShowError(_))),
                    "Notification should either succeed or fail gracefully in CI environment"
                );
            }
        } else {
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            assert!(
                result.is_ok(),
                "Notification should succeed on supported platforms"
            );

            #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
            assert!(matches!(result, Err(NotificationError::ShowError(_))));
        }
    }

    #[test]
    fn test_notification_cooldown() {
        let manager = NotificationManager::new(TEST_COOLDOWN);
        let app = "test_cooldown_app";

        let result1 = manager.alert_user(app);
        if !is_ci_environment() {
            assert!(result1.is_ok(), "First notification should succeed");
        }

        wait_for_cooldown_state(&manager, app, true);

        let result2 = manager.alert_user(app);
        assert!(
            matches!(result2, Err(NotificationError::Cooldown)),
            "Second notification should be in cooldown"
        );

        thread::sleep(TEST_COOLDOWN * 2);
        wait_for_cooldown_state(&manager, app, false);

        let result3 = manager.alert_user(app);
        if !is_ci_environment() {
            assert!(
                result3.is_ok(),
                "Notification should succeed after cooldown"
            );
        }
    }

    #[test]
    fn test_notification_concurrent() {
        let manager = Arc::new(NotificationManager::new(TEST_COOLDOWN));
        let app = "test_concurrent_app";
        let barrier = Arc::new(Barrier::new(THREAD_COUNT));

        let _ = manager.alert_user(app);

        wait_for_cooldown_state(&manager, app, true);

        let handles: Vec<_> = (0..THREAD_COUNT)
            .map(|_| {
                let app = app.to_string();
                let manager = Arc::clone(&manager);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    let result = manager.alert_user(&app);
                    debug!("Concurrent notification result: {:?}", result);
                    result
                })
            })
            .collect();

        for handle in handles {
            let result = handle.join().unwrap();
            assert!(
                matches!(result, Err(NotificationError::Cooldown)),
                "Concurrent notifications should be in cooldown"
            );
        }
    }

    #[test]
    fn test_notification_special_chars() {
        let manager = NotificationManager::new(TEST_COOLDOWN);
        const TEST_CASES: [&str; 6] = [
            r#"test"app"#,
            r#"test'app"#,
            r#"test\app"#,
            r#"test/app"#,
            r#"test app"#,
            r#"test_app"#,
        ];

        for app in TEST_CASES {
            let result = manager.alert_user(app);

            if !is_ci_environment() {
                assert!(result.is_ok(), "Failed to handle special chars in: {}", app);
            }

            thread::sleep(TEST_COOLDOWN * 2);
            wait_for_cooldown_state(&manager, app, false);
        }
    }
}
