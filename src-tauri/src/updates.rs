use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use crate::error::{AppError, AppResult};

const APP_ID: &str = "com.nocturne.cloudcheck";
const MAX_UPDATE_BYTES: usize = 500 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateEnvelope {
    pub app_id: String,
    pub version: String,
    pub target: String,
    pub download_url: String,
    pub artifact_sha256: String,
    pub signature: String,
}

/// Must be called before downloading. It binds update retrieval to the compiled release origin,
/// application ID, current platform, and an upgrade-only semantic version.
pub fn validate_update_source(envelope: &UpdateEnvelope) -> AppResult<()> {
    if envelope.app_id != APP_ID || envelope.target != current_target() {
        return Err(AppError::UpdateVerification);
    }
    let current =
        Version::parse(env!("CARGO_PKG_VERSION")).map_err(|_| AppError::UpdateVerification)?;
    let candidate = Version::parse(&envelope.version).map_err(|_| AppError::UpdateVerification)?;
    if candidate <= current {
        return Err(AppError::UpdateVerification);
    }
    if envelope.artifact_sha256.len() != 64
        || !envelope
            .artifact_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(AppError::UpdateVerification);
    }
    let trusted_origin = option_env!("NOCTURNE_UPDATE_ORIGIN")
        .filter(|value| !value.is_empty())
        .ok_or(AppError::UpdateVerification)?;
    let trusted = Url::parse(trusted_origin).map_err(|_| AppError::UpdateVerification)?;
    let download = Url::parse(&envelope.download_url).map_err(|_| AppError::UpdateVerification)?;
    if trusted.scheme() != "https"
        || download.scheme() != "https"
        || trusted.origin() != download.origin()
        || !download.username().is_empty()
        || download.password().is_some()
    {
        return Err(AppError::UpdateVerification);
    }
    Ok(())
}

/// Verifies signed metadata and the downloaded artifact. The private signing key never enters the
/// client; the public key and trusted origin are supplied only at release compile time.
pub fn verify_update(artifact: &[u8], envelope: &UpdateEnvelope) -> AppResult<()> {
    validate_update_source(envelope)?;
    if artifact.is_empty() || artifact.len() > MAX_UPDATE_BYTES {
        return Err(AppError::UpdateVerification);
    }
    let digest = format!("{:x}", Sha256::digest(artifact));
    if !constant_time_ascii_eq(&digest, &envelope.artifact_sha256.to_ascii_lowercase()) {
        return Err(AppError::UpdateVerification);
    }
    let message = signed_message(envelope);
    let encoded_key = option_env!("NOCTURNE_UPDATE_PUBLIC_KEY")
        .filter(|value| !value.is_empty())
        .ok_or(AppError::UpdateVerification)?;
    let key_bytes = STANDARD
        .decode(encoded_key)
        .map_err(|_| AppError::UpdateVerification)?;
    let key_bytes: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| AppError::UpdateVerification)?;
    let key = VerifyingKey::from_bytes(&key_bytes).map_err(|_| AppError::UpdateVerification)?;
    let signature = Signature::from_slice(
        &STANDARD
            .decode(&envelope.signature)
            .map_err(|_| AppError::UpdateVerification)?,
    )
    .map_err(|_| AppError::UpdateVerification)?;
    key.verify(message.as_bytes(), &signature)
        .map_err(|_| AppError::UpdateVerification)
}

fn signed_message(envelope: &UpdateEnvelope) -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}",
        envelope.app_id,
        envelope.version,
        envelope.target,
        envelope.download_url,
        envelope.artifact_sha256.to_ascii_lowercase()
    )
}

fn current_target() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn constant_time_ascii_eq(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.bytes()
        .zip(right.bytes())
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(url: &str) -> UpdateEnvelope {
        UpdateEnvelope {
            app_id: APP_ID.into(),
            version: "99.0.0".into(),
            target: current_target(),
            download_url: url.into(),
            artifact_sha256: "0".repeat(64),
            signature: "bad".into(),
        }
    }

    #[test]
    fn rejects_non_https_and_wrong_targets() {
        assert!(verify_update(b"payload", &envelope("http://example.com/update")).is_err());
        let mut wrong = envelope("https://example.com/update");
        wrong.target = "other-platform".into();
        assert!(verify_update(b"payload", &wrong).is_err());
    }

    #[test]
    fn rejects_downgrades_before_download() {
        let mut old = envelope("https://example.com/update");
        old.version = env!("CARGO_PKG_VERSION").into();
        assert!(validate_update_source(&old).is_err());
    }
}
