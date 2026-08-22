use std::{
    io::Read,
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, AppResult},
    persistence::{LicenseRecord, Repository},
};

const LICENSE_API: &str = "https://api.lemonsqueezy.com/v1/licenses";
#[cfg(feature = "desktop")]
const CREDENTIAL_SERVICE: &str = "com.nocturne.cloudcheck";
#[cfg(feature = "desktop")]
const CREDENTIAL_USER: &str = "license-key";
const OFFLINE_GRACE_DAYS: i64 = 7;
const MAX_LICENSE_RESPONSE_BYTES: u64 = 64 * 1024;

#[derive(Clone, Debug)]
pub struct Activation {
    pub activation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemoteLicenseError {
    Invalid,
    Unavailable,
}

pub trait LicenseClient: Send + Sync {
    fn activate(&self, key: &str, instance_name: &str) -> Result<Activation, RemoteLicenseError>;
    fn validate(&self, key: &str, activation_id: &str) -> Result<bool, RemoteLicenseError>;
    fn deactivate(&self, key: &str, activation_id: &str) -> Result<(), RemoteLicenseError>;
}

pub trait SecretStore: Send + Sync {
    fn set(&self, secret: &str) -> AppResult<()>;
    fn get(&self) -> AppResult<Option<String>>;
    fn delete(&self) -> AppResult<()>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LicenseState {
    Unlicensed,
    Active,
    Grace,
    Invalid,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseStatus {
    pub state: LicenseState,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub grace_expires_at: Option<DateTime<Utc>>,
}

pub struct LicenseManager {
    client: Arc<dyn LicenseClient>,
    secrets: Arc<dyn SecretStore>,
    repository: Arc<Repository>,
    operation: Mutex<()>,
}

impl LicenseManager {
    pub fn new(
        client: Arc<dyn LicenseClient>,
        secrets: Arc<dyn SecretStore>,
        repository: Arc<Repository>,
    ) -> Self {
        Self {
            client,
            secrets,
            repository,
            operation: Mutex::new(()),
        }
    }

    pub fn activate(&self, key: &str, now: DateTime<Utc>) -> AppResult<LicenseStatus> {
        let _guard = self.operation.lock().map_err(|_| AppError::Internal)?;
        validate_license_key(key)?;
        if self.repository.load_license()?.is_some() {
            return Err(AppError::InvalidInput(
                "deactivate the existing license before activating another".into(),
            ));
        }
        let instance_name = format!("Nocturne CloudCheck {}", uuid::Uuid::new_v4());
        let activation = self
            .client
            .activate(key, &instance_name)
            .map_err(map_remote_error)?;
        if let Err(error) = self.secrets.set(key) {
            let _ = self.client.deactivate(key, &activation.activation_id);
            return Err(error);
        }
        let record = LicenseRecord {
            activation_id: activation.activation_id,
            status: "active".into(),
            last_verified_at: now,
        };
        if self.repository.save_license(&record).is_err() {
            let _ = self.secrets.delete();
            let _ = self.client.deactivate(key, &record.activation_id);
            return Err(AppError::Storage);
        }
        Ok(active_status(now))
    }

    pub fn status(&self, now: DateTime<Utc>) -> AppResult<LicenseStatus> {
        let _guard = self.operation.lock().map_err(|_| AppError::Internal)?;
        let Some(record) = self.repository.load_license()? else {
            return Ok(unlicensed_status());
        };
        if record.last_verified_at > now + chrono::Duration::minutes(5) {
            return Ok(LicenseStatus {
                state: LicenseState::Invalid,
                last_verified_at: Some(record.last_verified_at),
                grace_expires_at: None,
            });
        }
        let Some(key) = self.secrets.get()? else {
            return Ok(LicenseStatus {
                state: LicenseState::Invalid,
                last_verified_at: Some(record.last_verified_at),
                grace_expires_at: None,
            });
        };
        match self.client.validate(&key, &record.activation_id) {
            Ok(true) => {
                let refreshed = LicenseRecord {
                    last_verified_at: now,
                    status: "active".into(),
                    ..record
                };
                self.repository.save_license(&refreshed)?;
                Ok(active_status(now))
            }
            Ok(false) | Err(RemoteLicenseError::Invalid) => Ok(LicenseStatus {
                state: LicenseState::Invalid,
                last_verified_at: Some(record.last_verified_at),
                grace_expires_at: None,
            }),
            Err(RemoteLicenseError::Unavailable) => {
                let grace_expires =
                    record.last_verified_at + chrono::Duration::days(OFFLINE_GRACE_DAYS);
                if now <= grace_expires {
                    Ok(LicenseStatus {
                        state: LicenseState::Grace,
                        last_verified_at: Some(record.last_verified_at),
                        grace_expires_at: Some(grace_expires),
                    })
                } else {
                    Err(AppError::LicenseUnavailable)
                }
            }
        }
    }

    pub fn deactivate(&self) -> AppResult<()> {
        let _guard = self.operation.lock().map_err(|_| AppError::Internal)?;
        let Some(record) = self.repository.load_license()? else {
            return Ok(());
        };
        let key = self.secrets.get()?.ok_or(AppError::LicenseInvalid)?;
        self.client
            .deactivate(&key, &record.activation_id)
            .map_err(map_remote_error)?;
        self.secrets.delete()?;
        self.repository.clear_license()
    }
}

pub struct LemonSqueezyClient {
    http: Client,
}

impl LemonSqueezyClient {
    pub fn new() -> AppResult<Self> {
        let http = Client::builder()
            .https_only(true)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(20))
            .user_agent(concat!("Nocturne-CloudCheck/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| AppError::Internal)?;
        Ok(Self { http })
    }

    fn post(
        &self,
        operation: &str,
        fields: &[(&str, &str)],
    ) -> Result<ApiResponse, RemoteLicenseError> {
        let url = format!("{LICENSE_API}/{operation}");
        let response = self
            .http
            .post(url)
            .form(fields)
            .send()
            .map_err(|_| RemoteLicenseError::Unavailable)?;
        if response.status().is_server_error() || response.status().as_u16() == 429 {
            return Err(RemoteLicenseError::Unavailable);
        }
        if !response.status().is_success() {
            return Err(RemoteLicenseError::Invalid);
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_LICENSE_RESPONSE_BYTES)
        {
            return Err(RemoteLicenseError::Unavailable);
        }
        let mut bytes = Vec::new();
        response
            .take(MAX_LICENSE_RESPONSE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| RemoteLicenseError::Unavailable)?;
        if bytes.len() as u64 > MAX_LICENSE_RESPONSE_BYTES {
            return Err(RemoteLicenseError::Unavailable);
        }
        serde_json::from_slice(&bytes).map_err(|_| RemoteLicenseError::Unavailable)
    }
}

impl LicenseClient for LemonSqueezyClient {
    fn activate(&self, key: &str, instance_name: &str) -> Result<Activation, RemoteLicenseError> {
        let response = self.post(
            "activate",
            &[("license_key", key), ("instance_name", instance_name)],
        )?;
        if !response.activated.unwrap_or(false) {
            return Err(RemoteLicenseError::Invalid);
        }
        let activation_id = response
            .instance
            .and_then(|instance| instance.id)
            .filter(|id| !id.is_empty())
            .ok_or(RemoteLicenseError::Unavailable)?;
        Ok(Activation { activation_id })
    }

    fn validate(&self, key: &str, activation_id: &str) -> Result<bool, RemoteLicenseError> {
        let response = self.post(
            "validate",
            &[("license_key", key), ("instance_id", activation_id)],
        )?;
        Ok(response.valid.unwrap_or(false))
    }

    fn deactivate(&self, key: &str, activation_id: &str) -> Result<(), RemoteLicenseError> {
        let response = self.post(
            "deactivate",
            &[("license_key", key), ("instance_id", activation_id)],
        )?;
        if response.deactivated.unwrap_or(false) {
            Ok(())
        } else {
            Err(RemoteLicenseError::Invalid)
        }
    }
}

#[derive(Deserialize)]
struct ApiResponse {
    activated: Option<bool>,
    deactivated: Option<bool>,
    valid: Option<bool>,
    instance: Option<ApiInstance>,
}

#[derive(Deserialize)]
struct ApiInstance {
    id: Option<String>,
}

#[cfg(feature = "desktop")]
pub struct KeyringSecretStore;

#[cfg(feature = "desktop")]
impl KeyringSecretStore {
    fn entry() -> AppResult<keyring::Entry> {
        keyring::Entry::new(CREDENTIAL_SERVICE, CREDENTIAL_USER)
            .map_err(|_| AppError::CredentialStore)
    }
}

#[cfg(feature = "desktop")]
impl SecretStore for KeyringSecretStore {
    fn set(&self, secret: &str) -> AppResult<()> {
        Self::entry()?
            .set_password(secret)
            .map_err(|_| AppError::CredentialStore)
    }

    fn get(&self) -> AppResult<Option<String>> {
        match Self::entry()?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(AppError::CredentialStore),
        }
    }

    fn delete(&self) -> AppResult<()> {
        match Self::entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(AppError::CredentialStore),
        }
    }
}

fn validate_license_key(key: &str) -> AppResult<()> {
    if !(8..=256).contains(&key.len()) || key.chars().any(char::is_control) {
        return Err(AppError::InvalidInput("invalid license key".into()));
    }
    Ok(())
}

fn map_remote_error(error: RemoteLicenseError) -> AppError {
    match error {
        RemoteLicenseError::Invalid => AppError::LicenseInvalid,
        RemoteLicenseError::Unavailable => AppError::LicenseUnavailable,
    }
}

fn active_status(now: DateTime<Utc>) -> LicenseStatus {
    LicenseStatus {
        state: LicenseState::Active,
        last_verified_at: Some(now),
        grace_expires_at: None,
    }
}

fn unlicensed_status() -> LicenseStatus {
    LicenseStatus {
        state: LicenseState::Unlicensed,
        last_verified_at: None,
        grace_expires_at: None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use chrono::TimeZone;
    use tempfile::tempdir;

    use super::*;

    struct FakeClient {
        validation: Result<bool, RemoteLicenseError>,
    }

    impl LicenseClient for FakeClient {
        fn activate(&self, _: &str, _: &str) -> Result<Activation, RemoteLicenseError> {
            Ok(Activation {
                activation_id: "instance".into(),
            })
        }
        fn validate(&self, _: &str, _: &str) -> Result<bool, RemoteLicenseError> {
            self.validation.clone()
        }
        fn deactivate(&self, _: &str, _: &str) -> Result<(), RemoteLicenseError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct MemorySecrets(Mutex<Option<String>>);

    impl SecretStore for MemorySecrets {
        fn set(&self, secret: &str) -> AppResult<()> {
            *self.0.lock().unwrap() = Some(secret.into());
            Ok(())
        }
        fn get(&self) -> AppResult<Option<String>> {
            Ok(self.0.lock().unwrap().clone())
        }
        fn delete(&self) -> AppResult<()> {
            *self.0.lock().unwrap() = None;
            Ok(())
        }
    }

    #[test]
    fn grants_bounded_grace_only_after_activation() {
        let directory = tempdir().unwrap();
        let repository = Arc::new(Repository::open(directory.path().join("db")).unwrap());
        let manager = LicenseManager::new(
            Arc::new(FakeClient {
                validation: Err(RemoteLicenseError::Unavailable),
            }),
            Arc::new(MemorySecrets::default()),
            repository,
        );
        let activated = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();
        manager.activate("valid-key", activated).unwrap();
        let status = manager
            .status(activated + chrono::Duration::days(6))
            .unwrap();
        assert_eq!(status.state, LicenseState::Grace);
        assert!(manager
            .status(activated + chrono::Duration::days(8))
            .is_err());
    }

    #[test]
    fn invalid_remote_response_never_gets_grace() {
        let directory = tempdir().unwrap();
        let manager = LicenseManager::new(
            Arc::new(FakeClient {
                validation: Ok(false),
            }),
            Arc::new(MemorySecrets::default()),
            Arc::new(Repository::open(directory.path().join("db")).unwrap()),
        );
        let now = Utc::now();
        manager.activate("valid-key", now).unwrap();
        assert_eq!(manager.status(now).unwrap().state, LicenseState::Invalid);
    }

    #[test]
    fn rejects_clock_rollback_and_reactivation() {
        let directory = tempdir().unwrap();
        let manager = LicenseManager::new(
            Arc::new(FakeClient {
                validation: Err(RemoteLicenseError::Unavailable),
            }),
            Arc::new(MemorySecrets::default()),
            Arc::new(Repository::open(directory.path().join("db")).unwrap()),
        );
        let now = Utc::now();
        manager.activate("valid-key", now).unwrap();
        assert_eq!(
            manager
                .status(now - chrono::Duration::hours(1))
                .unwrap()
                .state,
            LicenseState::Invalid
        );
        assert!(manager.activate("another-key", now).is_err());
    }
}
