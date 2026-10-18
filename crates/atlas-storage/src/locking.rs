use crate::{StorageError, StorageResult};
use std::{
    fs::{File, OpenOptions, TryLockError},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub(crate) fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

/// Lock files are stable: unlinking one would split cooperating processes across
/// two different locks. The OS releases each open-file lock on process death.
pub(crate) fn acquire(path: &Path, suffix: &str, exclusive: bool) -> StorageResult<File> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(sidecar(path, suffix))?;
    // Brief overlapping operations should not surface as product errors.
    // Long-lived readers still return a bounded busy result.
    let started = Instant::now();
    loop {
        let result = if exclusive {
            file.try_lock()
        } else {
            file.try_lock_shared()
        };
        match result {
            Ok(()) => break,
            Err(TryLockError::WouldBlock) if started.elapsed() < Duration::from_millis(250) => {
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(TryLockError::WouldBlock) => return Err(StorageError::Busy),
            Err(TryLockError::Error(error)) => return Err(StorageError::Io(error)),
        }
    }
    Ok(file)
}
