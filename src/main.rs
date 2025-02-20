use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use color_eyre::{eyre::Context, Result};
use directories::ProjectDirs;
use sysinfo::{Pid, System};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, instrument};

mod compression;
mod notification;
mod settings;

use notification::alert_user;
use settings::Settings;

type ProcessData = HashMap<Pid, (String, u64)>;
type UsageData = HashMap<String, u64>;

#[instrument]
async fn load_persisted_data() -> Option<UsageData> {
    let proj_dirs = ProjectDirs::from("com", "DataGuardian", "DataGuardian")?;
    let data_path = proj_dirs.data_dir().join("usage.dat");

    match tokio::fs::read(&data_path).await {
        Ok(contents) => match compression::decompress_usage_data(&contents) {
            Ok(data) => Some(data),
            Err(e) => {
                error!(error = %e, "Failed to decompress persisted data");
                None
            }
        },
        Err(e) => {
            debug!(error = %e, "Failed to read persisted data");
            None
        }
    }
}

#[instrument(skip(data))]
async fn save_persisted_data(data: &UsageData) -> Result<()> {
    let proj_dirs = ProjectDirs::from("com", "DataGuardian", "DataGuardian")
        .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get project directories"))?;
    let data_path = proj_dirs.data_dir().join("usage.dat");

    let compressed =
        compression::compress_usage_data(data).context("Failed to compress usage data")?;
    tokio::fs::create_dir_all(proj_dirs.data_dir()).await?;
    tokio::fs::write(data_path, compressed)
        .await
        .map_err(Into::into)
}

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

#[cfg(unix)]
fn drop_privileges() -> Result<()> {
    use nix::unistd::{setgid, setuid, Gid, Uid};
    let uid = Uid::current();
    let gid = Gid::current();
    setgid(gid)?;
    setuid(uid)?;
    Ok(())
}

fn setup_logging() -> Result<()> {
    use tracing_subscriber::{fmt, EnvFilter};
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("data_guardian=info".parse()?))
        .with_ansi(atty::is(atty::Stream::Stdout))
        .init();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    setup_logging()?;

    #[cfg(unix)]
    drop_privileges().context("Failed to drop privileges")?;

    let settings = Settings::new().context("Failed to load settings")?;
    let mut app_usage = load_persisted_data().await.unwrap_or_default();
    let mut prev_processes = ProcessData::new();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        r.store(false, Ordering::SeqCst);
    });

    let mut monitor_interval = interval(Duration::from_secs(settings.check_interval_seconds));
    let mut save_interval = interval(Duration::from_secs(settings.persistence_interval_seconds));

    info!(?settings, "Starting Data Guardian service");

    while running.load(Ordering::SeqCst) {
        tokio::select! {
            _ = monitor_interval.tick() => {
                let current_processes = get_current_processes().await?;
                let mut current_usage = UsageData::with_capacity(current_processes.len());

                for (pid, (app_name, current_total)) in &current_processes {
                    if let Some((prev_app, prev_total)) = prev_processes.get(pid) {
                        if prev_app == app_name {
                            *current_usage.entry(app_name.clone()).or_insert(0) +=
                                current_total.saturating_sub(*prev_total);
                        }
                    }
                }

                prev_processes = current_processes;

                for (app, delta) in current_usage {
                    let total_usage = app_usage.entry(app.clone()).or_insert(0);
                    *total_usage += delta;

                    if *total_usage > settings.data_limit {
                        if let Err(e) = alert_user(&app) {
                            error!(error = %e, app = %app, "Failed to send notification");
                        }
                        info!(%app, %total_usage, "Application exceeded data limit");
                    }
                }
            }
            _ = save_interval.tick() => {
                if let Err(e) = save_persisted_data(&app_usage).await {
                    error!(error = %e, "Failed to persist data");
                }
            }
        }
    }

    info!("Shutting down gracefully...");
    save_persisted_data(&app_usage).await?;
    Ok(())
}
