//! The write lock of a dossier, and the check for network drives.

use crate::error::{Code, Error, Result};
use std::fs::{File, TryLockError};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// How long a waiting command pauses between two tries of the lock.
const POLL: Duration = Duration::from_millis(50);

/// A held write lock. The lock is released when the guard is dropped.
pub struct Guard {
    /// Held, never read: the lock lasts as long as this file is open.
    _file: File,
    holder: PathBuf,
}

impl Drop for Guard {
    fn drop(&mut self) {
        // Runs before the file closes, so holder.json is removed while the lock is still held,
        // never after another holder has written its own. Errors are ignored.
        let _ = std::fs::remove_file(&self.holder);
    }
}

/// Locks `<state>/write.lock` with `File::try_lock`, retrying every 50 ms for `wait_seconds`,
/// then Code::Busy naming the holder from holder.json. Writes holder.json; Drop removes it.
pub fn acquire(state: &Path, wait_seconds: u64, name: &str, command: &str) -> Result<Guard> {
    std::fs::create_dir_all(state).map_err(|error| io_error(state, error))?;
    let lock_path = state.join("write.lock");
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|error| io_error(&lock_path, error))?;
    let holder = state.join("holder.json");

    let wait = Duration::from_secs(wait_seconds);
    let started = Instant::now();
    loop {
        match file.try_lock() {
            Ok(()) => break,
            Err(TryLockError::WouldBlock) => {
                let waited = started.elapsed();
                if waited >= wait {
                    return Err(busy(&holder));
                }
                std::thread::sleep((wait - waited).min(POLL));
            }
            Err(TryLockError::Error(error)) => return Err(io_error(&lock_path, error)),
        }
    }

    // The guard is built before holder.json is written: if writing fails, dropping the guard
    // removes whatever was written and releases the lock.
    let guard = Guard {
        _file: file,
        holder,
    };
    let text = serde_json::json!({
        "name": name,
        "command": command,
        "pid": std::process::id(),
        "since": crate::time::now(),
    })
    .to_string();
    std::fs::write(&guard.holder, text).map_err(|error| io_error(&guard.holder, error))?;
    Ok(guard)
}

/// What holder.json says about the holder of the lock.
#[derive(serde::Deserialize)]
struct Holder {
    name: String,
    command: String,
    since: String,
}

/// The error for a lock that is still held: names the holder when holder.json can be read.
fn busy(holder: &Path) -> Error {
    let known = std::fs::read(holder)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Holder>(&bytes).ok());
    let message = match known {
        Some(holder) => format!(
            "the dossier is locked by {} ({}) since {}; try again",
            holder.name, holder.command, holder.since
        ),
        None => "the dossier is locked by another command; try again".to_string(),
    };
    Error::new(Code::Busy, message)
}

/// Windows: UNC path or a drive of type DRIVE_REMOTE. Linux: the mount of the path in
/// /proc/mounts has type cifs, smb3, smbfs, nfs or nfs4.
#[cfg(windows)]
pub fn is_network(path: &Path) -> bool {
    use std::path::{Component, Prefix};

    let full = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    match full.components().next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::UNC(..) | Prefix::VerbatimUNC(..) => true,
            Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => drive_is_remote(letter),
            _ => false,
        },
        _ => false,
    }
}

/// Whether the drive `letter` (an ASCII letter) is a network drive.
#[cfg(windows)]
fn drive_is_remote(letter: u8) -> bool {
    // windows_sys has DRIVE_REMOTE in Win32::System::WindowsProgramming, which this crate does
    // not enable; the value is fixed by the Windows API.
    const DRIVE_REMOTE: u32 = 4;
    let root = [u16::from(letter), u16::from(b':'), u16::from(b'\\'), 0];
    // SAFETY: `root` is a NUL-terminated UTF-16 string that lives for the whole call, and
    // GetDriveTypeW only reads it.
    let kind = unsafe { windows_sys::Win32::Storage::FileSystem::GetDriveTypeW(root.as_ptr()) };
    kind == DRIVE_REMOTE
}

/// Linux: the mount of the path (its canonical form, or the path itself when that fails) in
/// /proc/mounts decides. Without /proc/mounts the answer is no.
#[cfg(target_os = "linux")]
pub fn is_network(path: &Path) -> bool {
    let Ok(mounts) = std::fs::read("/proc/mounts") else {
        return false;
    };
    let full = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    network_mount(&mounts, &full)
}

/// Other systems have no check: nothing is refused.
#[cfg(not(any(windows, target_os = "linux")))]
pub fn is_network(path: &Path) -> bool {
    false
}

/// Whether the mount that holds `path`, found in the text of /proc/mounts, is a network file
/// system. The mount point that is the longest whole-folder prefix of `path` decides.
#[cfg(any(target_os = "linux", test))]
fn network_mount(mounts: &[u8], path: &Path) -> bool {
    // The length of the deciding mount point, and whether its type is a network type.
    let mut deciding: Option<(usize, bool)> = None;
    for line in mounts.split(|byte| *byte == b'\n') {
        let mut fields = line
            .split(|byte| *byte == b' ' || *byte == b'\t')
            .filter(|field| !field.is_empty());
        // Device, mount point, type; the options and the two numbers after them are not needed.
        let (Some(_device), Some(point), Some(kind)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let point = os_path(&unescape(point));
        if !path.starts_with(&point) {
            continue;
        }
        let length = point.as_os_str().len();
        // A later line is mounted on top of an earlier one, so a tie goes to the later line.
        if deciding.is_none_or(|(best, _)| length >= best) {
            let network = matches!(kind, b"cifs" | b"smb3" | b"smbfs" | b"nfs" | b"nfs4");
            deciding = Some((length, network));
        }
    }
    deciding.is_some_and(|(_, network)| network)
}

/// The text of a mount point in /proc/mounts with its escapes read: `\040` is a space, `\011`
/// a tab, `\012` a new line and `\134` a backslash. A backslash that does not start such an
/// escape stays as it is.
#[cfg(any(target_os = "linux", test))]
fn unescape(field: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(field.len());
    let mut rest = field;
    while let Some((&first, tail)) = rest.split_first() {
        if first == b'\\' && tail.len() >= 3 && tail[..3].iter().all(|b| (b'0'..=b'7').contains(b))
        {
            let value = tail[..3]
                .iter()
                .fold(0u32, |acc, digit| acc * 8 + u32::from(digit - b'0'));
            if let Ok(byte) = u8::try_from(value) {
                out.push(byte);
                rest = &tail[3..];
                continue;
            }
        }
        out.push(first);
        rest = tail;
    }
    out
}

/// A path from raw bytes of /proc/mounts.
#[cfg(all(any(target_os = "linux", test), unix))]
fn os_path(bytes: &[u8]) -> PathBuf {
    use std::os::unix::ffi::OsStrExt;
    PathBuf::from(std::ffi::OsStr::from_bytes(bytes))
}

/// A path from the bytes of /proc/mounts, where the system has no raw paths: UTF-8 is read,
/// anything else is replaced.
#[cfg(all(any(target_os = "linux", test), not(unix)))]
fn os_path(bytes: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
}

/// An I/O failure on `path` as Code::Io, with the path in the message.
fn io_error(path: &Path, error: std::io::Error) -> Error {
    Error::new(Code::Io, format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The time that DOSSIER_NOW is set to during the tests.
    const NOW: &str = "2026-11-20T14:03:27+01:00";

    /// Holds ENV_LOCK and sets DOSSIER_NOW for as long as it lives, so that holder.json has a
    /// known time.
    struct FixedClock {
        _env: std::sync::MutexGuard<'static, ()>,
    }

    impl FixedClock {
        fn new() -> FixedClock {
            let guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            // SAFETY: ENV_LOCK is held until this value is dropped, so no other test in this
            // process reads or changes the environment meanwhile.
            unsafe { std::env::set_var("DOSSIER_NOW", NOW) };
            FixedClock { _env: guard }
        }
    }

    impl Drop for FixedClock {
        fn drop(&mut self) {
            // SAFETY: ENV_LOCK is still held here (the guard field is dropped after this).
            unsafe { std::env::remove_var("DOSSIER_NOW") };
        }
    }

    #[test]
    fn a_second_writer_with_no_wait_is_busy_and_is_told_who_holds_the_lock() {
        let _clock = FixedClock::new();
        let folder = tempfile::tempdir().unwrap();
        let state = folder.path().join(".state");
        let held = acquire(&state, 0, "anna", "found").unwrap();
        let error = acquire(&state, 0, "ben", "decide").err().unwrap();
        assert_eq!(error.code, Code::Busy);
        assert_eq!(
            error.message,
            format!("the dossier is locked by anna (found) since {NOW}; try again")
        );
        drop(held);
    }

    #[test]
    fn the_lock_is_free_again_after_the_guard_is_dropped() {
        let _clock = FixedClock::new();
        let folder = tempfile::tempdir().unwrap();
        let state = folder.path().join(".state");
        drop(acquire(&state, 0, "anna", "found").unwrap());
        assert!(!state.join("holder.json").exists());
        let again = acquire(&state, 0, "ben", "decide").unwrap();
        drop(again);
    }

    #[test]
    fn holder_json_names_the_holder_while_the_lock_is_held() {
        let _clock = FixedClock::new();
        let folder = tempfile::tempdir().unwrap();
        let state = folder.path().join(".state");
        let held = acquire(&state, 0, "anna", "found").unwrap();
        let text = std::fs::read_to_string(state.join("holder.json")).unwrap();
        let holder: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(holder["name"], "anna");
        assert_eq!(holder["command"], "found");
        assert_eq!(holder["pid"], std::process::id());
        assert_eq!(holder["since"], NOW);
        drop(held);
    }

    #[test]
    fn a_writer_that_waits_gives_up_with_busy_while_the_holder_keeps_the_lock() {
        let _clock = FixedClock::new();
        let folder = tempfile::tempdir().unwrap();
        let state = folder.path().join(".state");
        let held = acquire(&state, 0, "anna", "found").unwrap();
        let started = Instant::now();
        let error = acquire(&state, 1, "ben", "decide").err().unwrap();
        assert_eq!(error.code, Code::Busy);
        assert!(started.elapsed() >= Duration::from_secs(1));
        drop(held);
    }

    #[test]
    fn a_waiting_writer_gets_the_lock_when_the_holder_lets_go() {
        let _clock = FixedClock::new();
        let folder = tempfile::tempdir().unwrap();
        let state_path = folder.path().join(".state");
        let state: &Path = &state_path;
        let (acquired, ready) = std::sync::mpsc::channel();
        std::thread::scope(|scope| {
            scope.spawn(move || {
                let held = acquire(state, 0, "anna", "found").unwrap();
                acquired.send(()).unwrap();
                std::thread::sleep(Duration::from_millis(300));
                drop(held);
            });
            ready.recv().unwrap();
            drop(acquire(state, 10, "ben", "decide").unwrap());
        });
    }

    #[test]
    fn a_temporary_folder_is_not_a_network_drive() {
        let folder = tempfile::tempdir().unwrap();
        assert!(!is_network(folder.path()));
    }

    #[cfg(windows)]
    #[test]
    fn a_unc_share_is_a_network_path() {
        assert!(is_network(Path::new(r"\\server\team\dossier")));
        assert!(is_network(Path::new("//server/team/dossier")));
        assert!(is_network(Path::new(r"\\?\UNC\server\team\dossier")));
    }

    /// A /proc/mounts text: a local disk, a CIFS share, an NFS export inside the CIFS share, a
    /// local disk inside the CIFS share, and a mount point with escaped spaces.
    const MOUNTS: &str = "\
/dev/sda1 / ext4 rw,relatime 0 0
//server/team /mnt/team cifs rw,relatime 0 0
server:/export /mnt/team/sub nfs4 rw 0 0
/dev/sdb1 /mnt/team/local ext4 rw 0 0
server:/data /mnt/dir\\040with\\040space nfs rw 0 0
";

    /// Whether `path` lies on a network mount of MOUNTS.
    fn on_network(path: &str) -> bool {
        network_mount(MOUNTS.as_bytes(), Path::new(path))
    }

    #[test]
    fn a_path_under_a_cifs_mount_is_on_the_network() {
        assert!(on_network("/mnt/team/docs/a.md"));
    }

    #[test]
    fn the_longest_mount_point_decides() {
        assert!(on_network("/mnt/team/sub/x"));
        assert!(!on_network("/mnt/team/local/x"));
    }

    #[test]
    fn a_mount_point_must_match_whole_folder_names() {
        // "/mnt/teamwork" is not under "/mnt/team": the root disk decides.
        assert!(!on_network("/mnt/teamwork/a"));
    }

    #[test]
    fn a_path_on_the_root_disk_is_not_on_the_network() {
        assert!(!on_network("/home/anna/project"));
    }

    #[test]
    fn escaped_spaces_in_a_mount_point_are_read_as_spaces() {
        assert!(on_network("/mnt/dir with space/x"));
    }

    #[test]
    fn unescape_turns_octal_codes_into_bytes() {
        assert_eq!(unescape(br"a\040b\011c\134d"), b"a b\tc\\d");
        // Not three octal digits, or a code above 255: kept as they are.
        assert_eq!(unescape(br"\999 \04"), br"\999 \04");
    }
}
