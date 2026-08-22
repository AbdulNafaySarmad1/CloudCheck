use std::{path::PathBuf, sync::Mutex};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    domain::{Scan, ScanSummary},
    error::{AppError, AppResult},
};

pub struct Repository {
    connection: Mutex<Connection>,
}

#[derive(Clone, Debug)]
pub struct LicenseRecord {
    pub activation_id: String,
    pub status: String,
    pub last_verified_at: DateTime<Utc>,
}

impl Repository {
    pub fn open(path: PathBuf) -> AppResult<Self> {
        let connection = Connection::open(path).map_err(|_| AppError::Storage)?;
        connection
            .execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA foreign_keys = ON;
                 PRAGMA busy_timeout = 5000;
                 CREATE TABLE IF NOT EXISTS schema_migrations (
                    version INTEGER PRIMARY KEY,
                    applied_at TEXT NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS scans (
                    id TEXT PRIMARY KEY,
                    provider TEXT NOT NULL,
                    account_id TEXT NOT NULL,
                    scanned_at TEXT NOT NULL,
                    resource_count INTEGER NOT NULL CHECK(resource_count >= 0),
                    finding_count INTEGER NOT NULL CHECK(finding_count >= 0),
                    scan_json TEXT NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS idx_scans_scanned_at ON scans(scanned_at DESC);
                 CREATE TABLE IF NOT EXISTS license_state (
                    singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
                    activation_id TEXT NOT NULL,
                    status TEXT NOT NULL,
                    last_verified_at TEXT NOT NULL
                 );
                 INSERT OR IGNORE INTO schema_migrations(version, applied_at)
                 VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));",
            )
            .map_err(|_| AppError::Storage)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn save_scan(&self, scan: &Scan) -> AppResult<()> {
        let json = serde_json::to_string(scan).map_err(|_| AppError::Internal)?;
        let connection = self.connection.lock().map_err(|_| AppError::Storage)?;
        connection
            .execute(
                "INSERT INTO scans
                 (id, provider, account_id, scanned_at, resource_count, finding_count, scan_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    scan.id,
                    "aws",
                    scan.account_id,
                    scan.scanned_at.to_rfc3339(),
                    scan.resource_count as i64,
                    scan.findings.len() as i64,
                    json
                ],
            )
            .map_err(|_| AppError::Storage)?;
        Ok(())
    }

    pub fn get_scan(&self, id: &str) -> AppResult<Scan> {
        validate_scan_id(id)?;
        let connection = self.connection.lock().map_err(|_| AppError::Storage)?;
        let json: Option<String> = connection
            .query_row(
                "SELECT scan_json FROM scans WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| AppError::Storage)?;
        serde_json::from_str(&json.ok_or(AppError::NotFound)?).map_err(|_| AppError::Storage)
    }

    pub fn list_scans(&self, limit: u32) -> AppResult<Vec<ScanSummary>> {
        if !(1..=100).contains(&limit) {
            return Err(AppError::InvalidInput(
                "limit must be between 1 and 100".into(),
            ));
        }
        let connection = self.connection.lock().map_err(|_| AppError::Storage)?;
        let mut statement = connection
            .prepare(
                "SELECT id, account_id, scanned_at, resource_count, finding_count
                 FROM scans ORDER BY scanned_at DESC, id DESC LIMIT ?1",
            )
            .map_err(|_| AppError::Storage)?;
        let rows = statement
            .query_map(params![limit], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })
            .map_err(|_| AppError::Storage)?;
        let mut summaries = Vec::new();
        for row in rows {
            let (id, account_id, scanned_at, resource_count, finding_count) =
                row.map_err(|_| AppError::Storage)?;
            summaries.push(ScanSummary {
                id,
                provider: crate::domain::Provider::Aws,
                account_id,
                scanned_at: DateTime::parse_from_rfc3339(&scanned_at)
                    .map_err(|_| AppError::Storage)?
                    .with_timezone(&Utc),
                resource_count: usize::try_from(resource_count).map_err(|_| AppError::Storage)?,
                finding_count: usize::try_from(finding_count).map_err(|_| AppError::Storage)?,
            });
        }
        Ok(summaries)
    }

    pub fn save_license(&self, record: &LicenseRecord) -> AppResult<()> {
        let connection = self.connection.lock().map_err(|_| AppError::Storage)?;
        connection
            .execute(
                "INSERT INTO license_state(singleton, activation_id, status, last_verified_at)
                 VALUES (1, ?1, ?2, ?3)
                 ON CONFLICT(singleton) DO UPDATE SET
                    activation_id = excluded.activation_id,
                    status = excluded.status,
                    last_verified_at = excluded.last_verified_at",
                params![
                    record.activation_id,
                    record.status,
                    record.last_verified_at.to_rfc3339()
                ],
            )
            .map_err(|_| AppError::Storage)?;
        Ok(())
    }

    pub fn load_license(&self) -> AppResult<Option<LicenseRecord>> {
        let connection = self.connection.lock().map_err(|_| AppError::Storage)?;
        let record = connection
            .query_row(
                "SELECT activation_id, status, last_verified_at FROM license_state WHERE singleton = 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|_| AppError::Storage)?;
        record
            .map(|(activation_id, status, last_verified_at)| {
                Ok(LicenseRecord {
                    activation_id,
                    status,
                    last_verified_at: DateTime::parse_from_rfc3339(&last_verified_at)
                        .map_err(|_| AppError::Storage)?
                        .with_timezone(&Utc),
                })
            })
            .transpose()
    }

    pub fn clear_license(&self) -> AppResult<()> {
        let connection = self.connection.lock().map_err(|_| AppError::Storage)?;
        connection
            .execute("DELETE FROM license_state WHERE singleton = 1", [])
            .map_err(|_| AppError::Storage)?;
        Ok(())
    }
}

fn validate_scan_id(id: &str) -> AppResult<()> {
    if id.len() > 64 || id.is_empty() || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err(AppError::InvalidInput("invalid scan id".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use tempfile::tempdir;

    use super::*;
    use crate::domain::{Provider, Scan};

    fn scan() -> Scan {
        Scan {
            id: "scan-1".into(),
            provider: Provider::Aws,
            account_id: "123".into(),
            scope: vec![],
            scanned_at: Utc::now(),
            product_version: "1".into(),
            ruleset_version: "1".into(),
            resource_count: 0,
            findings: vec![],
            limitations: vec![],
        }
    }

    #[test]
    fn scan_round_trip_and_parameterized_lookup() {
        let directory = tempdir().unwrap();
        let repository = Repository::open(directory.path().join("data.db")).unwrap();
        repository.save_scan(&scan()).unwrap();
        assert_eq!(repository.get_scan("scan-1").unwrap().account_id, "123");
        assert!(repository.get_scan("' OR 1=1 --").is_err());
        assert_eq!(repository.list_scans(10).unwrap().len(), 1);
    }
}
