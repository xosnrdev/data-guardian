use data_guardian::{alert_user, Settings};

#[tokio::test]
async fn test_settings_load() {
    let settings = Settings::new().unwrap();
    assert!(settings.data_limit > 0);
    assert!(settings.check_interval_seconds > 0);
}

#[test]
fn test_notification_system() {
    let result = alert_user("test_app");
    assert!(result.is_ok(), "Notification should succeed");
}
