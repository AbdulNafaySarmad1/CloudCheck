use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    aws::{AwsSnapshotSource, JsonSnapshotSource, MAX_SNAPSHOT_BYTES},
    domain::Snapshot,
    error::{AppError, AppResult},
    reports::ReportFormat,
};

pub fn import_snapshot(path: &str) -> AppResult<Snapshot> {
    let path = validated_absolute_path(path)?;
    let initial = fs::symlink_metadata(&path).map_err(|_| AppError::UnsafePath)?;
    if initial.file_type().is_symlink() || !initial.is_file() {
        return Err(AppError::UnsafePath);
    }
    if initial.len() > MAX_SNAPSHOT_BYTES {
        return Err(AppError::FileTooLarge);
    }
    if path.extension().and_then(|value| value.to_str()) != Some("json") {
        return Err(AppError::InvalidInput(
            "snapshot must be a JSON file".into(),
        ));
    }
    let canonical = path.canonicalize().map_err(|_| AppError::UnsafePath)?;
    let file = File::open(canonical).map_err(|_| AppError::UnsafePath)?;
    let opened = file.metadata().map_err(|_| AppError::UnsafePath)?;
    if !opened.is_file() || !same_file_identity(&initial, &opened) {
        return Err(AppError::UnsafePath);
    }
    JsonSnapshotSource::new(file).load()
}

pub fn write_export(
    directory: &str,
    filename: &str,
    format: ReportFormat,
    contents: &[u8],
) -> AppResult<PathBuf> {
    validate_filename(filename, format)?;
    let selected_directory = validated_absolute_path(directory)?;
    let selected_metadata =
        fs::symlink_metadata(&selected_directory).map_err(|_| AppError::UnsafePath)?;
    if selected_metadata.file_type().is_symlink() || !selected_metadata.is_dir() {
        return Err(AppError::UnsafePath);
    }
    let directory = selected_directory
        .canonicalize()
        .map_err(|_| AppError::UnsafePath)?;
    let canonical_metadata = fs::metadata(&directory).map_err(|_| AppError::UnsafePath)?;
    if !canonical_metadata.is_dir() || !same_file_identity(&selected_metadata, &canonical_metadata)
    {
        return Err(AppError::UnsafePath);
    }
    let destination = directory.join(filename);
    if destination.exists() {
        return Err(AppError::AlreadyExists);
    }
    let temporary = directory.join(format!(".cloudcheck-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut file = create_private_file(&temporary)?;
        verify_directory_identity(&directory, &canonical_metadata)?;
        file.write_all(contents).map_err(|_| AppError::Storage)?;
        file.sync_all().map_err(|_| AppError::Storage)?;
        verify_directory_identity(&directory, &canonical_metadata)?;
        fs::hard_link(&temporary, &destination).map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                AppError::AlreadyExists
            } else {
                AppError::Storage
            }
        })?;
        verify_directory_identity(&directory, &canonical_metadata)?;
        let temporary_metadata = fs::metadata(&temporary).map_err(|_| AppError::UnsafePath)?;
        let destination_metadata =
            fs::symlink_metadata(&destination).map_err(|_| AppError::UnsafePath)?;
        if destination_metadata.file_type().is_symlink()
            || !same_file_identity(&temporary_metadata, &destination_metadata)
        {
            return Err(AppError::UnsafePath);
        }
        Ok(destination.clone())
    })();
    if result.is_err() {
        if let (Ok(temporary_metadata), Ok(destination_metadata)) =
            (fs::metadata(&temporary), fs::symlink_metadata(&destination))
        {
            if !destination_metadata.file_type().is_symlink()
                && same_file_identity(&temporary_metadata, &destination_metadata)
            {
                let _ = fs::remove_file(&destination);
            }
        }
    }
    let _ = fs::remove_file(&temporary);
    result
}

fn verify_directory_identity(directory: &Path, expected: &fs::Metadata) -> AppResult<()> {
    let current = fs::symlink_metadata(directory).map_err(|_| AppError::UnsafePath)?;
    if current.file_type().is_symlink()
        || !current.is_dir()
        || !same_file_identity(expected, &current)
    {
        return Err(AppError::UnsafePath);
    }
    Ok(())
}

#[cfg(unix)]
fn same_file_identity(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(windows)]
fn same_file_identity(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    left.volume_serial_number() == right.volume_serial_number()
        && left.file_index() == right.file_index()
}

#[cfg(not(any(unix, windows)))]
fn same_file_identity(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    left.len() == right.len()
        && left.created().ok() == right.created().ok()
        && left.modified().ok() == right.modified().ok()
}

fn validated_absolute_path(value: &str) -> AppResult<PathBuf> {
    if value.is_empty() || value.len() > 4096 || value.contains('\0') {
        return Err(AppError::InvalidInput("invalid path".into()));
    }
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(AppError::UnsafePath);
    }
    Ok(path)
}

fn validate_filename(filename: &str, format: ReportFormat) -> AppResult<()> {
    if filename.is_empty()
        || filename.len() > 128
        || filename.starts_with('.')
        || !filename
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(AppError::InvalidInput("invalid export filename".into()));
    }
    if Path::new(filename)
        .extension()
        .and_then(|value| value.to_str())
        != Some(format.extension())
    {
        return Err(AppError::InvalidInput(
            "filename extension does not match format".into(),
        ));
    }
    Ok(())
}

fn create_private_file(path: &Path) -> AppResult<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).map_err(|_| AppError::Storage)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn export_rejects_traversal_and_overwrite() {
        let directory = tempdir().unwrap();
        let root = directory.path().to_str().unwrap();
        assert!(write_export(root, "../report.json", ReportFormat::Json, b"{}").is_err());
        write_export(root, "report.json", ReportFormat::Json, b"{}").unwrap();
        assert!(matches!(
            write_export(root, "report.json", ReportFormat::Json, b"changed"),
            Err(AppError::AlreadyExists)
        ));
        assert_eq!(
            fs::read(directory.path().join("report.json")).unwrap(),
            b"{}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn import_rejects_symlinks() {
        use std::os::unix::fs::symlink;

        let directory = tempdir().unwrap();
        let target = directory.path().join("snapshot.json");
        fs::write(&target, b"{}").unwrap();
        let link = directory.path().join("link.json");
        symlink(target, &link).unwrap();
        assert!(matches!(
            import_snapshot(link.to_str().unwrap()),
            Err(AppError::UnsafePath)
        ));
    }
}
