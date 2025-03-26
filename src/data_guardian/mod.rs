pub mod compression;
pub mod notification;
pub mod settings;

#[cfg(test)]
mod tests {
    use crate::data_guardian::{notification::NotificationError, settings::Settings};

    use super::notification::alert_user;

    #[tokio::test]
    async fn test_settings_load() {
        let settings = Settings::new().unwrap();
        assert!(settings.data_limit > 0);
        assert!(settings.check_interval_seconds > 0);
    }

    #[test]
    fn test_notification_system() {
        let result = alert_user("test_app");

        if std::env::var("CI").is_ok() {
            #[cfg(target_os = "linux")]
            {
                assert!(
                    matches!(result, Err(NotificationError::ShowError(_))),
                    "Notification should fail gracefully in Linux CI environment"
                );
            }

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
            assert!(
                matches!(result, Err(NotificationError::ShowError(_))),
                "Notification should fail on unsupported platforms"
            );
        }
    }
}
