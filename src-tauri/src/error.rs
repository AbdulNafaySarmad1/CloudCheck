use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("the requested item was not found")]
    NotFound,
    #[error("the selected path is not allowed")]
    UnsafePath,
    #[error("the file is larger than the allowed limit")]
    FileTooLarge,
    #[error("the destination already exists")]
    AlreadyExists,
    #[error("the imported snapshot is invalid")]
    InvalidSnapshot,
    #[error("local storage is unavailable")]
    Storage,
    #[error("the credential store is unavailable")]
    CredentialStore,
    #[error("license validation is unavailable")]
    LicenseUnavailable,
    #[error("the license is invalid")]
    LicenseInvalid,
    #[error("update verification failed")]
    UpdateVerification,
    #[error("an internal operation failed")]
    Internal,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    code: &'static str,
    message: &'static str,
}

impl From<AppError> for CommandError {
    fn from(error: AppError) -> Self {
        let (code, message) = match error {
            AppError::InvalidInput(_) => ("INVALID_INPUT", "The supplied value is invalid."),
            AppError::NotFound => ("NOT_FOUND", "The requested item was not found."),
            AppError::UnsafePath => ("UNSAFE_PATH", "The selected path is not allowed."),
            AppError::FileTooLarge => ("FILE_TOO_LARGE", "The selected file is too large."),
            AppError::AlreadyExists => ("ALREADY_EXISTS", "The destination already exists."),
            AppError::InvalidSnapshot => ("INVALID_SNAPSHOT", "The snapshot format is invalid."),
            AppError::Storage => ("STORAGE_ERROR", "Local storage is unavailable."),
            AppError::CredentialStore => (
                "CREDENTIAL_STORE_ERROR",
                "Secure credential storage is unavailable.",
            ),
            AppError::LicenseUnavailable => (
                "LICENSE_UNAVAILABLE",
                "License validation is temporarily unavailable.",
            ),
            AppError::LicenseInvalid => ("LICENSE_INVALID", "The license is not valid."),
            AppError::UpdateVerification => (
                "UPDATE_VERIFICATION_FAILED",
                "The update could not be verified.",
            ),
            AppError::Internal => ("INTERNAL_ERROR", "The operation could not be completed."),
        };
        Self { code, message }
    }
}

pub type AppResult<T> = Result<T, AppError>;
