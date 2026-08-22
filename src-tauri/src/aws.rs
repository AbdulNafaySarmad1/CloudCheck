use std::io::Read;

use crate::{
    domain::Snapshot,
    error::{AppError, AppResult},
};

pub const MAX_SNAPSHOT_BYTES: u64 = 20 * 1024 * 1024;

/// AWS adapters normalize imported or API-collected data into the same snapshot contract.
pub trait AwsSnapshotSource {
    fn load(self) -> AppResult<Snapshot>;
}

pub struct JsonSnapshotSource<R> {
    reader: R,
}

impl<R> JsonSnapshotSource<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R: Read> AwsSnapshotSource for JsonSnapshotSource<R> {
    fn load(self) -> AppResult<Snapshot> {
        let mut reader = std::io::BufReader::new(self.reader);
        let mut bytes = Vec::new();
        reader
            .by_ref()
            .take(MAX_SNAPSHOT_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| AppError::InvalidSnapshot)?;
        if bytes.len() as u64 > MAX_SNAPSHOT_BYTES {
            return Err(AppError::FileTooLarge);
        }
        let snapshot: Snapshot =
            serde_json::from_slice(&bytes).map_err(|_| AppError::InvalidSnapshot)?;
        snapshot.validate()?;
        Ok(snapshot)
    }
}

/// Boundary for a future SDK-backed collector. Implementations must use read-only AWS calls and
/// return normalized data; the rule engine never receives credentials or SDK clients.
pub trait AwsReadOnlyCollector: Send + Sync {
    fn collect(&self, account_id: &str, regions: &[String]) -> AppResult<Snapshot>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_snapshot() {
        let input = br#"{"schema_version":1,"provider":"aws","account_id":"123456789012","scope":["us-east-1"],"resources":[]}"#;
        let source = JsonSnapshotSource::new(input.as_slice());
        assert_eq!(source.load().unwrap().account_id, "123456789012");
    }

    #[test]
    fn rejects_unknown_fields() {
        let input = br#"{"schema_version":1,"provider":"aws","account_id":"123","scope":[],"resources":[],"credentials":"secret"}"#;
        assert!(JsonSnapshotSource::new(input.as_slice()).load().is_err());
    }
}
