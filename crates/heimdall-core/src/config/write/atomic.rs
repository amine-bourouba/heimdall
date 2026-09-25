//! Atomic file replacement (ADR 0009, rule 4): write a temp file beside the target, flush it, then
//! rename it over the target. The target's permissions (Unix mode) carry over, and the original
//! stays intact on every failure path.
//!
//! On Windows, Claude Code may hold the file open without `FILE_SHARE_DELETE`, which makes the
//! rename fail. That is retried with backoff for about two seconds, then `ReplaceFileW` is tried
//! once, then the write gives up with `AppError::Busy`.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use crate::error::{AppError, Result};

const RETRY_BUDGET: Duration = Duration::from_secs(2);
const MAX_DELAY: Duration = Duration::from_millis(250);

/// Replaces `target` with `bytes`. The temp file is removed if anything fails.
pub fn replace(target: &Path, bytes: &[u8]) -> Result<()> {
    let temp = temp_path(target);
    let result = write_temp(&temp, bytes, target).and_then(|()| rename_with_retries(&temp, target));
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

/// `.<name>.heimdall-<pid>-<n>.tmp` in the target's folder, so the rename never crosses volumes.
fn temp_path(target: &Path) -> PathBuf {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let name = target.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    target.with_file_name(format!(".{name}.heimdall-{}-{n}.tmp", std::process::id()))
}

fn write_temp(temp: &Path, bytes: &[u8], target: &Path) -> Result<()> {
    let io = |e: std::io::Error| AppError::from_io(&e, temp);
    let mut file = OpenOptions::new().write(true).create_new(true).open(temp).map_err(io)?;
    file.write_all(bytes).map_err(io)?;
    file.sync_all().map_err(io)?;
    drop(file);
    if let Ok(meta) = std::fs::metadata(target) {
        std::fs::set_permissions(temp, meta.permissions()).map_err(io)?;
    }
    Ok(())
}

fn rename_with_retries(temp: &Path, target: &Path) -> Result<()> {
    let start = Instant::now();
    let mut delay = Duration::from_millis(10);
    loop {
        match std::fs::rename(temp, target) {
            Ok(()) => return Ok(()),
            Err(e) if !is_sharing_violation(&e) => return Err(AppError::from_io(&e, target)),
            Err(_) if start.elapsed() < RETRY_BUDGET => {
                std::thread::sleep(delay);
                delay = (delay * 2).min(MAX_DELAY);
            }
            Err(_) => {
                return replace_file(temp, target).map_err(|_| {
                    AppError::busy("the file is in use by another program; close it or try again").with_path(target)
                })
            }
        }
    }
}

/// Access denied, sharing violation and lock violation: what Windows returns while another
/// process holds the file.
fn is_sharing_violation(e: &std::io::Error) -> bool {
    cfg!(windows) && matches!(e.raw_os_error(), Some(5 | 32 | 33))
}

#[cfg(windows)]
fn replace_file(temp: &Path, target: &Path) -> std::io::Result<()> {
    use windows::core::{HSTRING, PCWSTR};
    use windows::Win32::Storage::FileSystem::{ReplaceFileW, REPLACE_FILE_FLAGS};
    let (target_w, temp_w) = (HSTRING::from(target.as_os_str()), HSTRING::from(temp.as_os_str()));
    // SAFETY: both arguments are valid NUL-terminated wide strings that outlive the call; the
    // backup name, exclude and reserved pointers are null as the API allows.
    unsafe { ReplaceFileW(&target_w, &temp_w, PCWSTR::null(), REPLACE_FILE_FLAGS(0), None, None) }
        .map_err(|e| std::io::Error::from_raw_os_error(e.code().0))
}

#[cfg(not(windows))]
fn replace_file(_temp: &Path, _target: &Path) -> std::io::Result<()> {
    Err(std::io::Error::other("no fallback outside Windows"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(dir: &Path) -> Vec<String> {
        let mut v: Vec<_> = std::fs::read_dir(dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        v.sort();
        v
    }

    #[test]
    fn replaces_the_file_and_leaves_no_temp_behind() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("settings.json");
        std::fs::write(&p, "old").unwrap();
        replace(&p, b"new").unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "new");
        assert_eq!(names(d.path()), ["settings.json"]);
    }

    #[cfg(unix)]
    #[test]
    fn unix_mode_is_preserved() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join(".claude.json");
        std::fs::write(&p, "{}").unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600)).unwrap();
        replace(&p, b"{\"a\":1}").unwrap();
        assert_eq!(std::fs::metadata(&p).unwrap().permissions().mode() & 0o777, 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_rename_removes_the_temp_file_and_keeps_the_target() {
        let d = tempfile::tempdir().unwrap();
        let target = d.path().join("dir-not-file");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("keep"), "x").unwrap();
        assert!(replace(&target, b"data").is_err());
        assert_eq!(names(d.path()), ["dir-not-file"]);
        assert!(target.join("keep").exists());
    }

    /// Opens `p` the way Claude Code might: shared for reading and writing, not for deletion.
    #[cfg(windows)]
    fn hold_without_share_delete(p: &Path) -> std::fs::File {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_SHARE_READ: u32 = 1;
        const FILE_SHARE_WRITE: u32 = 2;
        OpenOptions::new().read(true).share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE).open(p).unwrap()
    }

    #[cfg(windows)]
    #[test]
    fn a_held_file_gives_busy_after_the_retries_and_stays_intact() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join(".claude.json");
        std::fs::write(&p, "original").unwrap();
        let handle = hold_without_share_delete(&p);
        let started = Instant::now();
        let err = replace(&p, b"new").unwrap_err();
        let waited = started.elapsed();
        drop(handle);
        assert!(matches!(err, AppError::Busy { path: Some(_), .. }), "{err:?}");
        assert!(waited >= RETRY_BUDGET && waited < RETRY_BUDGET * 3, "{waited:?}");
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "original");
        assert_eq!(names(d.path()), [".claude.json"]);
    }

    #[cfg(windows)]
    #[test]
    fn a_handle_released_during_the_retries_lets_the_write_through() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("settings.json");
        std::fs::write(&p, "original").unwrap();
        let handle = hold_without_share_delete(&p);
        let releaser = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(150));
            drop(handle);
        });
        replace(&p, b"new").unwrap();
        releaser.join().unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "new");
    }
}
