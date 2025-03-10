use std::collections::HashMap;
use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use color_eyre::{eyre::Context, Result};
use directories::ProjectDirs;
use sysinfo::{Pid, System};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, instrument};

use data_guardian::{
    compression,
    notification::{self, NotificationError},
    Settings,
};

/// Type alias for process-specific data, mapping PIDs to (name, usage)
type ProcessData = HashMap<Pid, (String, u64)>;
/// Type alias for aggregated usage data, mapping app names to total usage
type UsageData = HashMap<String, u64>;

/// Configuration for data persistence
#[derive(Debug)]
struct PersistenceConfig {
    /// Directory for storing data files
    data_dir: std::path::PathBuf,
    /// File name for usage data
    file_name: &'static str,
}

impl PersistenceConfig {
    /// Creates a new persistence configuration
    fn new() -> Option<Self> {
        ProjectDirs::from("com", "DataGuardian", "DataGuardian").map(|dirs| Self {
            data_dir: dirs.data_dir().to_path_buf(),
            file_name: "usage.dat",
        })
    }

    /// Gets the full path to the usage data file
    fn data_path(&self) -> std::path::PathBuf {
        self.data_dir.join(self.file_name)
    }
}

/// Loads persisted usage data from disk
///
/// This function is pure in the sense that it only reads data and doesn't
/// modify any external state beyond logging.
#[instrument]
async fn load_persisted_data() -> Option<UsageData> {
    let config = PersistenceConfig::new()?;
    let data_path = config.data_path();

    if !data_path.exists() {
        debug!(?data_path, "No existing usage data found");
        return None;
    }

    debug!(?data_path, "Loading persisted usage data");
    match tokio::fs::read(&data_path).await {
        Ok(contents) => {
            debug!(size = contents.len(), "Read persisted data file");
            match compression::decompress_usage_data(&contents) {
                Ok(data) => {
                    debug!(entries = data.len(), "Successfully loaded usage data");
                    Some(data)
                }
                Err(e) => {
                    error!(error = %e, "Failed to decompress persisted data");
                    None
                }
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to read persisted data file");
            None
        }
    }
}

/// Saves usage data to disk with compression
///
/// This function handles the entire persistence flow:
/// 1. Ensures the data directory exists
/// 2. Compresses the data
/// 3. Writes the compressed data to disk
#[instrument(skip(data))]
async fn save_persisted_data(data: &UsageData) -> Result<()> {
    let config = PersistenceConfig::new()
        .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get project directories"))?;

    // Ensure data directory exists with proper logging
    if !config.data_dir.exists() {
        debug!(?config.data_dir, "Creating data directory");
        tokio::fs::create_dir_all(&config.data_dir)
            .await
            .context("Failed to create data directory")?;
    }

    let data_path = config.data_path();

    // Compress and save data with proper error context
    let compressed =
        compression::compress_usage_data(data).context("Failed to compress usage data")?;

    debug!(?data_path, size = compressed.len(), "Saving usage data");
    tokio::fs::write(&data_path, compressed)
        .await
        .context("Failed to write usage data file")?;

    debug!(?data_path, "Successfully saved usage data");
    Ok(())
}

/// Gets current process data from the system
///
/// This function is pure in that it only reads system state and returns
/// a new data structure without modifying anything.
#[instrument]
async fn get_current_processes() -> Result<ProcessData> {
    tokio::task::spawn_blocking(|| {
        let mut sys = System::new();
        sys.refresh_all();

        sys.processes()
            .iter()
            .map(|(pid, process)| {
                let name = process.name().to_string_lossy().into_owned();
                let usage = process.disk_usage();
                (
                    *pid,
                    (name, usage.read_bytes.saturating_add(usage.written_bytes)),
                )
            })
            .collect()
    })
    .await
    .map_err(Into::into)
}

/// Drops privileges on Unix systems for security
#[cfg(unix)]
fn drop_privileges() -> Result<()> {
    use nix::unistd::{setgid, setuid, Gid, Uid};
    let uid = Uid::current();
    let gid = Gid::current();
    setgid(gid)?;
    setuid(uid)?;
    Ok(())
}

/// Sets up logging with appropriate configuration
fn setup_logging() -> Result<()> {
    use tracing_subscriber::{fmt, EnvFilter};
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("data_guardian=info".parse()?))
        .with_ansi(std::io::stdout().is_terminal())
        .init();
    Ok(())
}

/// Monitors process data usage and sends notifications when thresholds are exceeded
async fn monitor_processes(
    settings: &Settings,
    app_usage: &mut UsageData,
    prev_processes: &mut ProcessData,
) -> Result<()> {
    let current_processes = get_current_processes().await?;
    let mut current_usage = UsageData::with_capacity(current_processes.len());

    // Calculate delta usage for each process
    for (pid, (app_name, current_total)) in &current_processes {
        if let Some((prev_app, prev_total)) = prev_processes.get(pid) {
            if prev_app == app_name {
                *current_usage.entry(app_name.clone()).or_insert(0) +=
                    current_total.saturating_sub(*prev_total);
            }
        }
    }

    *prev_processes = current_processes;

    // Update total usage and check thresholds
    for (app, delta) in current_usage {
        let total_usage = app_usage.entry(app.clone()).or_insert(0);
        *total_usage += delta;

        if *total_usage > settings.data_limit {
            match notification::alert_user(&app) {
                Ok(()) => info!(%app, %total_usage, "Application exceeded data limit"),
                Err(NotificationError::Cooldown) => {
                    debug!(%app, %total_usage, "Skipping notification due to cooldown");
                }
                Err(e) => {
                    error!(error = %e, app = %app, "Failed to send notification");
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize error handling and logging
    color_eyre::install()?;
    setup_logging()?;

    #[cfg(unix)]
    drop_privileges().context("Failed to drop privileges")?;

    // Load settings and initial state
    let settings = Settings::new().context("Failed to load settings")?;
    let mut app_usage = load_persisted_data().await.unwrap_or_default();
    let mut prev_processes = ProcessData::new();

    // Setup graceful shutdown handling
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        r.store(false, Ordering::SeqCst);
    });

    // Setup monitoring intervals
    let mut monitor_interval = interval(Duration::from_secs(settings.check_interval_seconds));
    let mut save_interval = interval(Duration::from_secs(settings.persistence_interval_seconds));

    info!(?settings, "Starting Data Guardian service");

    // Main service loop
    while running.load(Ordering::SeqCst) {
        tokio::select! {
            _ = monitor_interval.tick() => {
                if let Err(e) = monitor_processes(&settings, &mut app_usage, &mut prev_processes).await {
                    error!(error = %e, "Failed to monitor processes");
                }
            }
            _ = save_interval.tick() => {
                if let Err(e) = save_persisted_data(&app_usage).await {
                    error!(error = %e, "Failed to persist data");
                }
            }
        }
    }

    // Graceful shutdown
    info!("Shutting down gracefully...");
    save_persisted_data(&app_usage).await?;
    Ok(())
}
