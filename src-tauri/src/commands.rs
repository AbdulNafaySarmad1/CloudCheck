use std::sync::Arc;

use chrono::Utc;
use serde::Serialize;
use tauri::State;

use crate::{
    domain::{InventorySummary, Scan, ScanDiff, ScanSummary},
    engine,
    error::{AppError, CommandError},
    filesystem,
    licensing::{LicenseManager, LicenseState, LicenseStatus},
    persistence::Repository,
    reports::{self, ReportFormat},
};

pub struct AppState {
    pub repository: Arc<Repository>,
    pub licensing: Arc<LicenseManager>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    path: String,
    bytes_written: usize,
}

#[tauri::command]
pub async fn inspect_snapshot(path: String) -> Result<InventorySummary, CommandError> {
    blocking(move || {
        filesystem::import_snapshot(&path).map(|snapshot| InventorySummary::from(&snapshot))
    })
    .await
}

#[tauri::command]
pub async fn run_snapshot_scan(
    path: String,
    state: State<'_, AppState>,
) -> Result<Scan, CommandError> {
    let repository = Arc::clone(&state.repository);
    let licensing = Arc::clone(&state.licensing);
    blocking(move || {
        require_license(&licensing)?;
        let snapshot = filesystem::import_snapshot(&path)?;
        let scan = engine::evaluate(&snapshot, uuid::Uuid::new_v4().to_string(), Utc::now());
        repository.save_scan(&scan)?;
        Ok(scan)
    })
    .await
}

#[tauri::command]
pub async fn list_scans(
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<ScanSummary>, CommandError> {
    let repository = Arc::clone(&state.repository);
    blocking(move || repository.list_scans(limit.unwrap_or(50))).await
}

#[tauri::command]
pub async fn get_scan(scan_id: String, state: State<'_, AppState>) -> Result<Scan, CommandError> {
    let repository = Arc::clone(&state.repository);
    blocking(move || repository.get_scan(&scan_id)).await
}

#[tauri::command]
pub async fn diff_scans(
    base_scan_id: String,
    target_scan_id: String,
    state: State<'_, AppState>,
) -> Result<ScanDiff, CommandError> {
    let repository = Arc::clone(&state.repository);
    blocking(move || {
        let base = repository.get_scan(&base_scan_id)?;
        let target = repository.get_scan(&target_scan_id)?;
        Ok(engine::diff(&base, &target))
    })
    .await
}

#[tauri::command]
pub async fn export_scan(
    scan_id: String,
    directory: String,
    filename: String,
    format: ReportFormat,
    state: State<'_, AppState>,
) -> Result<ExportResult, CommandError> {
    let repository = Arc::clone(&state.repository);
    let licensing = Arc::clone(&state.licensing);
    blocking(move || {
        require_license(&licensing)?;
        let scan = repository.get_scan(&scan_id)?;
        let contents = reports::render(&scan, format)?;
        let path = filesystem::write_export(&directory, &filename, format, &contents)?;
        Ok(ExportResult {
            path: path.to_string_lossy().into_owned(),
            bytes_written: contents.len(),
        })
    })
    .await
}

#[tauri::command]
pub async fn activate_license(
    license_key: String,
    state: State<'_, AppState>,
) -> Result<LicenseStatus, CommandError> {
    let licensing = Arc::clone(&state.licensing);
    blocking(move || licensing.activate(&license_key, Utc::now())).await
}

#[tauri::command]
pub async fn license_status(state: State<'_, AppState>) -> Result<LicenseStatus, CommandError> {
    let licensing = Arc::clone(&state.licensing);
    blocking(move || licensing.status(Utc::now())).await
}

#[tauri::command]
pub async fn deactivate_license(state: State<'_, AppState>) -> Result<(), CommandError> {
    let licensing = Arc::clone(&state.licensing);
    blocking(move || licensing.deactivate()).await
}

fn require_license(licensing: &LicenseManager) -> Result<(), AppError> {
    match licensing.status(Utc::now())?.state {
        LicenseState::Active | LicenseState::Grace => Ok(()),
        LicenseState::Unlicensed | LicenseState::Invalid => Err(AppError::LicenseInvalid),
    }
}

async fn blocking<T, F>(operation: F) -> Result<T, CommandError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|_| CommandError::from(AppError::Internal))?
        .map_err(CommandError::from)
}
