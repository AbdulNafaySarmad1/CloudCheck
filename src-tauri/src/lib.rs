pub mod aws;
#[cfg(feature = "desktop")]
mod commands;
pub mod domain;
pub mod engine;
pub mod error;
pub mod filesystem;
pub mod licensing;
pub mod persistence;
pub mod reports;
pub mod updates;

#[cfg(feature = "desktop")]
use std::{fs, sync::Arc};

#[cfg(feature = "desktop")]
use commands::AppState;
#[cfg(feature = "desktop")]
use licensing::{KeyringSecretStore, LemonSqueezyClient, LicenseManager};
#[cfg(feature = "desktop")]
use persistence::Repository;
#[cfg(feature = "desktop")]
use tauri::Manager;

#[cfg(feature = "desktop")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            fs::create_dir_all(&data_directory)?;
            let data_directory = data_directory.canonicalize()?;
            let repository = Arc::new(
                Repository::open(data_directory.join("cloudcheck.db"))
                    .map_err(Box::<dyn std::error::Error>::from)?,
            );
            let licensing = Arc::new(LicenseManager::new(
                Arc::new(LemonSqueezyClient::new().map_err(Box::<dyn std::error::Error>::from)?),
                Arc::new(KeyringSecretStore),
                Arc::clone(&repository),
            ));
            app.manage(AppState {
                repository,
                licensing,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::inspect_snapshot,
            commands::run_snapshot_scan,
            commands::list_scans,
            commands::get_scan,
            commands::diff_scans,
            commands::export_scan,
            commands::activate_license,
            commands::license_status,
            commands::deactivate_license,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Nocturne CloudCheck");
}
