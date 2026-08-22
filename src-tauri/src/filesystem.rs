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
    #[cfg(windows)]
    let initial_handle = open_identity_handle(&path)?;
    let canonical = path.canonicalize().map_err(|_| AppError::UnsafePath)?;
    let file = File::open(canonical).map_err(|_| AppError::UnsafePath)?;
    let opened = file.metadata().map_err(|_| AppError::UnsafePath)?;
    #[cfg(windows)]
    let identity_matches = same_file_identity(&initial_handle, &file)?;
    #[cfg(not(windows))]
    let identity_matches = same_file_identity(&initial, &opened)?;
    if !opened.is_file() || !identity_matches {
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
    #[cfg(windows)]
    let selected_identity = open_identity_handle(&selected_directory)?;
    #[cfg(not(windows))]
    let selected_identity = selected_metadata;
    let directory = selected_directory
        .canonicalize()
        .map_err(|_| AppError::UnsafePath)?;
    let canonical_metadata = fs::metadata(&directory).map_err(|_| AppError::UnsafePath)?;
    let canonical_is_directory = canonical_metadata.is_dir();
    #[cfg(windows)]
    let directory_identity = open_identity_handle(&directory)?;
    #[cfg(not(windows))]
    let directory_identity = canonical_metadata;
    if !canonical_is_directory || !same_file_identity(&selected_identity, &directory_identity)? {
        return Err(AppError::UnsafePath);
    }
    let destination = directory.join(filename);
    if destination.exists() {
        return Err(AppError::AlreadyExists);
    }
    let temporary = directory.join(format!(".cloudcheck-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut file = create_private_file(&temporary)?;
        verify_directory_identity(&directory, &directory_identity)?;
        file.write_all(contents).map_err(|_| AppError::Storage)?;
        file.sync_all().map_err(|_| AppError::Storage)?;
        verify_directory_identity(&directory, &directory_identity)?;
        fs::hard_link(&temporary, &destination).map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                AppError::AlreadyExists
            } else {
                AppError::Storage
            }
        })?;
        verify_directory_identity(&directory, &directory_identity)?;
        #[cfg(windows)]
        let temporary_identity = open_identity_handle(&temporary)?;
        #[cfg(not(windows))]
        let temporary_identity = fs::metadata(&temporary).map_err(|_| AppError::UnsafePath)?;
        let destination_metadata =
            fs::symlink_metadata(&destination).map_err(|_| AppError::UnsafePath)?;
        let destination_is_symlink = destination_metadata.file_type().is_symlink();
        #[cfg(windows)]
        let destination_identity = open_identity_handle(&destination)?;
        #[cfg(not(windows))]
        let destination_identity = destination_metadata;
        if destination_is_symlink
            || !same_file_identity(&temporary_identity, &destination_identity)?
        {
            return Err(AppError::UnsafePath);
        }
        Ok(destination.clone())
    })();
    if result.is_err() {
        if let (Ok(_temporary_metadata), Ok(destination_metadata)) =
            (fs::metadata(&temporary), fs::symlink_metadata(&destination))
        {
            #[cfg(windows)]
            let identities_match = open_identity_handle(&temporary)
                .and_then(|temporary_handle| {
                    open_identity_handle(&destination).and_then(|destination_handle| {
                        same_file_identity(&temporary_handle, &destination_handle)
                    })
                })
                .unwrap_or(false);
            #[cfg(not(windows))]
            let identities_match =
                same_file_identity(&_temporary_metadata, &destination_metadata).unwrap_or(false);
            if !destination_metadata.file_type().is_symlink() && identities_match {
                let _ = fs::remove_file(&destination);
            }
        }
    }
    let _ = fs::remove_file(&temporary);
    result
}

#[cfg(windows)]
type FileIdentity = File;

#[cfg(not(windows))]
type FileIdentity = fs::Metadata;

fn verify_directory_identity(directory: &Path, expected: &FileIdentity) -> AppResult<()> {
    let current = fs::symlink_metadata(directory).map_err(|_| AppError::UnsafePath)?;
    let current_is_symlink = current.file_type().is_symlink();
    let current_is_directory = current.is_dir();
    #[cfg(windows)]
    let current_identity = open_identity_handle(directory)?;
    #[cfg(not(windows))]
    let current_identity = current;
    if current_is_symlink
        || !current_is_directory
        || !same_file_identity(expected, &current_identity)?
    {
        return Err(AppError::UnsafePath);
    }
    Ok(())
}

#[cfg(unix)]
fn same_file_identity(left: &fs::Metadata, right: &fs::Metadata) -> AppResult<bool> {
    use std::os::unix::fs::MetadataExt;
    Ok(left.dev() == right.dev() && left.ino() == right.ino())
}

#[cfg(windows)]
fn same_file_identity(left: &File, right: &File) -> AppResult<bool> {
    Ok(windows_file_identity(left)? == windows_file_identity(right)?)
}

#[cfg(windows)]
fn windows_file_identity(file: &File) -> AppResult<(u32, u64)> {
    use std::{mem::MaybeUninit, os::windows::io::AsRawHandle};
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    };

    let mut information = MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
    // The handle is owned by `file` and remains valid for this call; the API initializes
    // `information` only when it reports success.
    let succeeded =
        unsafe { GetFileInformationByHandle(file.as_raw_handle() as _, information.as_mut_ptr()) };
    if succeeded == 0 {
        return Err(AppError::UnsafePath);
    }
    let information = unsafe { information.assume_init() };
    let index = ((information.nFileIndexHigh as u64) << 32) | information.nFileIndexLow as u64;
    Ok((information.dwVolumeSerialNumber, index))
}

#[cfg(not(any(unix, windows)))]
fn same_file_identity(left: &fs::Metadata, right: &fs::Metadata) -> AppResult<bool> {
    Ok(left.len() == right.len()
        && left.created().ok() == right.created().ok()
        && left.modified().ok() == right.modified().ok())
}

#[cfg(windows)]
fn open_identity_handle(path: &Path) -> AppResult<File> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES,
        FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
    };

    OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|_| AppError::UnsafePath)
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

    #[cfg(windows)]
    #[test]
    fn windows_identity_matches_the_same_file() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("snapshot.json");
        fs::write(&path, b"{}").unwrap();

        let handle = open_identity_handle(&path).unwrap();
        assert!(same_file_identity(&handle, &handle).unwrap());
    }

    #[cfg(windows)]
    #[test]
    fn windows_identity_matches_a_hard_link() {
        let directory = tempdir().unwrap();
        let file = directory.path().join("snapshot.json");
        let link = directory.path().join("snapshot-link.json");
        fs::write(&file, b"{}").unwrap();
        fs::hard_link(&file, &link).unwrap();

        assert!(same_file_identity(
            &open_identity_handle(&file).unwrap(),
            &open_identity_handle(&link).unwrap()
        )
        .unwrap());
    }

    #[cfg(windows)]
    #[test]
    fn windows_identity_rejects_a_different_file() {
        let directory = tempdir().unwrap();
        let first = directory.path().join("first.json");
        let second = directory.path().join("second.json");
        fs::write(&first, b"first").unwrap();
        fs::write(&second, b"second").unwrap();

        assert!(!same_file_identity(
            &open_identity_handle(&first).unwrap(),
            &open_identity_handle(&second).unwrap()
        )
        .unwrap());
    }

    #[cfg(windows)]
    #[test]
    fn windows_identity_matches_the_same_directory() {
        let directory = tempdir().unwrap();
        assert!(same_file_identity(
            &open_identity_handle(directory.path()).unwrap(),
            &open_identity_handle(directory.path()).unwrap()
        )
        .unwrap());
    }

    #[cfg(windows)]
    #[test]
    fn windows_identity_rejects_missing_paths() {
        let directory = tempdir().unwrap();
        assert!(matches!(
            open_identity_handle(&directory.path().join("missing.json")),
            Err(AppError::UnsafePath)
        ));
    }
}
