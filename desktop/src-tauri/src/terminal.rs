use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    os::fd::OwnedFd,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use objc2::MainThreadMarker;
use objc2_app_kit::{NSAlert, NSAlertSecondButtonReturn};
use objc2_foundation::NSString;
use rustix::{
    fs::{self as at, AtFlags, FileType, Mode, OFlags, Stat, StatVfsMountFlags},
    io::Errno,
    process::geteuid,
};
use serde::Serialize;
use tauri::AppHandle;

use crate::{display, run_on_main};

const COMMAND: &str = "atlas";
const CHANGED: &str = "The terminal command or its folder changed. Review it and try again.";
const INSTALL: &str =
    "Move PF2e Atlas into Applications and reopen it before enabling the terminal command.";

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    device: u64,
    inode: u64,
    owner: u32,
    mode: u32,
    size: i64,
    changed: (i64, i64),
    modified: (i64, i64),
}

impl Entry {
    fn from_stat(stat: Stat) -> Self {
        Self {
            device: stat.st_dev.cast_unsigned().into(),
            inode: stat.st_ino,
            owner: stat.st_uid,
            mode: stat.st_mode.into(),
            size: stat.st_size,
            changed: (stat.st_ctime, stat.st_ctime_nsec),
            modified: (stat.st_mtime, stat.st_mtime_nsec),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Missing,
    Linked,
    Replaceable,
    Conflict,
}

#[derive(Debug, Serialize)]
pub struct Status {
    pub state: State,
    pub path: PathBuf,
    pub version: String,
}

pub struct Bundled {
    path: PathBuf,
    identity: Entry,
    version: String,
}

impl Bundled {
    pub fn inspect() -> Result<Self, String> {
        let host = std::env::current_exe()
            .map_err(display)?
            .canonicalize()
            .map_err(display)?;
        Self::inspect_path(&host.with_file_name(COMMAND))
    }

    fn inspect_path(path: &Path) -> Result<Self, String> {
        let stat = at::statat(at::CWD, path, AtFlags::SYMLINK_NOFOLLOW).map_err(display)?;
        if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile
            || stat.st_mode & 0o111 == 0
        {
            return Err("The bundled Atlas command is not a regular executable.".into());
        }
        let resolved = path.canonicalize().map_err(display)?;
        if resolved != path {
            return Err("The bundled Atlas command is outside its expected location.".into());
        }
        if resolved
            .components()
            .any(|part| part.as_os_str() == "AppTranslocation")
            || at::statvfs(&resolved)
                .map_err(display)?
                .f_flag
                .contains(StatVfsMountFlags::RDONLY)
        {
            return Err(INSTALL.into());
        }
        let bundle = resolved
            .ancestors()
            .nth(3)
            .filter(|bundle| bundle.extension().is_some_and(|ext| ext == "app"))
            .filter(|bundle| bundle.join("Contents/MacOS/atlas") == resolved)
            .ok_or(INSTALL)?;
        let plist = plist::Value::from_file(bundle.join("Contents/Info.plist")).map_err(display)?;
        let release = plist
            .as_dictionary()
            .ok_or("The Atlas app metadata is invalid.")?;
        let release = match release.get("AtlasReleaseTag") {
            Some(tag) => Some(
                tag.as_string()
                    .and_then(|tag| tag.strip_prefix('v'))
                    .ok_or("The Atlas release version is invalid.")?,
            ),
            None if cfg!(debug_assertions) => None,
            None => return Err("The Atlas release version is missing.".into()),
        };
        let identity = Entry::from_stat(stat);
        let version = verify_command(&resolved, release)?;
        let bundled = Self {
            path: resolved,
            identity,
            version,
        };
        bundled.recheck()?;
        Ok(bundled)
    }

    pub fn recheck(&self) -> Result<(), String> {
        let stat = at::statat(at::CWD, &self.path, AtFlags::SYMLINK_NOFOLLOW).map_err(display)?;
        if self.path.canonicalize().map_err(display)? != self.path
            || Entry::from_stat(stat) != self.identity
            || at::statvfs(&self.path)
                .map_err(display)?
                .f_flag
                .contains(StatVfsMountFlags::RDONLY)
        {
            return Err("The bundled Atlas command changed. Reopen the app and try again.".into());
        }
        Ok(())
    }
}

fn verify_command(path: &Path, expected: Option<&str>) -> Result<String, String> {
    let mut output = tempfile::tempfile().map_err(display)?;
    let mut child = Command::new(path)
        .args(["build-info", "--json"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(output.try_clone().map_err(display)?)
        .spawn()
        .map_err(display)?;
    let deadline = Instant::now() + Duration::from_secs(3);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            result => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(result.err().map_or_else(
                    || "The bundled Atlas command did not respond.".into(),
                    display,
                ));
            }
        }
    };
    let mut bytes = Vec::new();
    if status.success() {
        output.seek(SeekFrom::Start(0)).map_err(display)?;
        output.take(4096).read_to_end(&mut bytes).map_err(display)?;
    }
    let info: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
    let version = info["data"]["version"]
        .as_str()
        .filter(|version| semver::Version::parse(version).is_ok());
    if info["status"] != "ok"
        || info["data"]["variant"] != "full"
        || expected.is_some_and(|expected| Some(expected) != version)
    {
        return Err("The bundled Atlas command did not report the expected full build.".into());
    }
    version
        .map(str::to_owned)
        .ok_or_else(|| "The bundled Atlas command did not report a version.".into())
}

pub struct Destination {
    home: PathBuf,
    directory: Option<OwnedFd>,
    entry: Option<Entry>,
    pub status: Status,
}

fn open_bin(home: &Path, create: bool) -> Result<Option<OwnedFd>, String> {
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let mut directory = at::open(home, flags, Mode::empty()).map_err(display)?;
    for component in [".local", "bin"] {
        let mut next = at::openat(&directory, component, flags, Mode::empty());
        if matches!(next, Err(Errno::NOENT)) && create {
            match at::mkdirat(&directory, component, Mode::from_bits_truncate(0o755)) {
                Ok(()) | Err(Errno::EXIST) => {}
                Err(error) => return Err(display(error)),
            }
            next = at::openat(&directory, component, flags, Mode::empty());
        }
        directory = match next {
            Ok(next) if at::fstat(&next).map_err(display)?.st_uid == geteuid().as_raw() => next,
            Err(Errno::NOENT) if !create => return Ok(None),
            _ => return Err(
                "The terminal folder must be a folder you own at ~/.local/bin, without symlinks."
                    .into(),
            ),
        };
    }
    Ok(Some(directory))
}

fn entry(directory: &OwnedFd, name: &str) -> Result<Option<Entry>, String> {
    match at::statat(directory, name, AtFlags::SYMLINK_NOFOLLOW) {
        Ok(stat) => Ok(Some(Entry::from_stat(stat))),
        Err(Errno::NOENT) => Ok(None),
        Err(error) => Err(display(error)),
    }
}

fn same_directory(first: &OwnedFd, second: &OwnedFd) -> Result<bool, String> {
    let first = at::fstat(first).map_err(display)?;
    let second = at::fstat(second).map_err(display)?;
    Ok((first.st_dev, first.st_ino) == (second.st_dev, second.st_ino))
}

impl Destination {
    pub fn inspect(home: &Path, bundled: &Bundled) -> Result<Self, String> {
        let home = home.canonicalize().map_err(display)?;
        let directory = open_bin(&home, false)?;
        let entry = directory
            .as_ref()
            .map(|directory| entry(directory, COMMAND))
            .transpose()?
            .flatten();
        let state = match &entry {
            None => State::Missing,
            Some(found) => match FileType::from_raw_mode(found.mode as _) {
                FileType::RegularFile
                    if found.owner == geteuid().as_raw() && found.mode & 0o111 != 0 =>
                {
                    State::Replaceable
                }
                FileType::Symlink
                    if directory.as_ref().is_some_and(|directory| {
                        at::readlinkat(directory, COMMAND, Vec::new()).is_ok_and(|target| {
                            target.as_bytes() == bundled.path.as_os_str().as_encoded_bytes()
                        })
                    }) =>
                {
                    State::Linked
                }
                _ => State::Conflict,
            },
        };
        let status = Status {
            state,
            path: home.join(".local/bin/atlas"),
            version: bundled.version.clone(),
        };
        Ok(Self {
            home,
            directory,
            entry,
            status,
        })
    }

    pub fn commit(mut self, bundled: &Bundled, replace: bool) -> Result<Status, String> {
        bundled.recheck()?;
        match self.status.state {
            State::Linked => {
                let latest = Self::inspect(&self.home, bundled)?;
                return if latest.status.state == State::Linked {
                    Ok(latest.status)
                } else {
                    Err(CHANGED.into())
                };
            }
            State::Conflict | State::Replaceable if !replace => {
                return Err(format!(
                    "An atlas command already exists at {}.",
                    self.status.path.display()
                ));
            }
            State::Conflict => return Err(CHANGED.into()),
            State::Missing | State::Replaceable => {}
        }
        let current = open_bin(&self.home, self.directory.is_none())?.ok_or(CHANGED)?;
        if let Some(original) = &self.directory
            && !same_directory(original, &current)?
        {
            return Err(CHANGED.into());
        }
        if entry(&current, COMMAND)? != self.entry {
            return Err(CHANGED.into());
        }
        if self.status.state == State::Missing {
            bundled.recheck()?;
            at::symlinkat(&bundled.path, &current, COMMAND).map_err(display)?;
        } else {
            let mut random = [0_u8; 16];
            getrandom::fill(&mut random).map_err(display)?;
            let name = format!(".atlas-link-{:032x}", u128::from_ne_bytes(random));
            at::symlinkat(&bundled.path, &current, &name).map_err(display)?;
            let result = (|| {
                let staged = at::statat(&current, &name, AtFlags::empty()).map_err(display)?;
                let directory = open_bin(&self.home, false)?.ok_or(CHANGED)?;
                if Entry::from_stat(staged) != bundled.identity
                    || !same_directory(&directory, &current)?
                    || entry(&current, COMMAND)? != self.entry
                {
                    return Err(CHANGED.into());
                }
                // Only the descriptor-relative rename mutates the confirmed entry.
                at::renameat(&current, &name, &current, COMMAND).map_err(display)
            })();
            if result.is_err() {
                let _ = at::unlinkat(&current, &name, AtFlags::empty());
            }
            result?;
        }
        self.status.state = State::Linked;
        Ok(self.status)
    }
}

pub fn confirm(app: &AppHandle, status: &Status) -> Result<bool, String> {
    let (send, receive) = mpsc::sync_channel(1);
    let message = format!(
        "Replace the command at {} with a link to Atlas {} bundled with this app?",
        status.path.display(),
        status.version
    );
    run_on_main(app, move |_| {
        let answer = MainThreadMarker::new().is_some_and(|main| {
            let alert = NSAlert::new(main);
            alert.setMessageText(&NSString::from_str("Switch to bundled CLI?"));
            alert.setInformativeText(&NSString::from_str(&message));
            alert
                .addButtonWithTitle(&NSString::from_str("Cancel"))
                .setKeyEquivalent(&NSString::from_str("\r"));
            alert
                .addButtonWithTitle(&NSString::from_str("Switch to bundled CLI"))
                .setKeyEquivalent(&NSString::from_str(""));
            alert.runModal() == NSAlertSecondButtonReturn
        });
        let _ = send.send(answer);
    })?;
    receive.recv().map_err(display)
}

pub fn link_from_bootstrap(
    home: &Path,
    directory: &Path,
    bundled: &Bundled,
) -> Result<String, String> {
    let home = home.canonicalize().map_err(display)?;
    if directory == home.join(".local/bin") {
        let status = Destination::inspect(&home, bundled)?.commit(bundled, false)?;
        return Ok(status.path.display().to_string());
    }
    let directory = directory.canonicalize().map_err(display)?;
    if !directory.starts_with(home) || !directory.is_dir() {
        return Err("Choose an existing bin folder inside your home folder.".into());
    }
    let destination = directory.join(COMMAND);
    bundled.recheck()?;
    match std::os::unix::fs::symlink(&bundled.path, &destination) {
        Ok(()) => Ok(destination.display().to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if fs::read_link(&destination).is_ok_and(|path| path == bundled.path) {
                Ok(destination.display().to_string())
            } else {
                Err(format!(
                    "An atlas command already exists at {}. Choose another folder to preserve it.",
                    destination.display()
                ))
            }
        }
        Err(error) => Err(display(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};

    fn executable(path: &Path, content: &str) {
        fs::write(path, content).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn fixture() -> (tempfile::TempDir, Bundled) {
        let home = tempfile::tempdir().unwrap();
        let path = home.path().canonicalize().unwrap().join("bundled");
        executable(&path, "bundled CLI");
        let identity =
            Entry::from_stat(at::statat(at::CWD, &path, AtFlags::SYMLINK_NOFOLLOW).unwrap());
        (
            home,
            Bundled {
                path,
                identity,
                version: "0.1.0".into(),
            },
        )
    }

    #[test]
    fn first_enable_is_idempotent() {
        let (home, bundled) = fixture();
        for state in [State::Missing, State::Linked] {
            let destination = Destination::inspect(home.path(), &bundled).unwrap();
            assert_eq!(destination.status.state, state);
            let result = destination.commit(&bundled, false).unwrap();
            assert_eq!(fs::read_link(result.path).unwrap(), bundled.path);
        }
    }

    #[test]
    fn only_confirmed_executable_is_replaced() {
        let (home, bundled) = fixture();
        let bin = home.path().join(".local/bin");
        fs::create_dir_all(&bin).unwrap();
        let path = bin.join(COMMAND);
        executable(&path, "old");
        let inspect = || Destination::inspect(home.path(), &bundled).unwrap();
        assert_eq!(inspect().status.state, State::Replaceable);
        assert!(inspect().commit(&bundled, false).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "old");
        let stale = inspect();
        executable(&path, "changed");
        assert!(stale.commit(&bundled, true).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "changed");
        assert_eq!(
            inspect().commit(&bundled, true).unwrap().state,
            State::Linked
        );
        assert_eq!(fs::read_link(&path).unwrap(), bundled.path);
        assert_eq!(fs::read_dir(bin).unwrap().count(), 1);
    }

    #[test]
    fn protects_symlinks_and_replaced_parents() {
        let (home, bundled) = fixture();
        let bin = home.path().join(".local/bin");
        fs::create_dir_all(&bin).unwrap();
        let path = bin.join(COMMAND);
        symlink("somewhere-else", &path).unwrap();
        let inspect = || Destination::inspect(home.path(), &bundled).unwrap();
        assert_eq!(inspect().status.state, State::Conflict);
        assert!(inspect().commit(&bundled, true).is_err());
        assert_eq!(fs::read_link(&path).unwrap(), Path::new("somewhere-else"));
        fs::remove_file(&path).unwrap();
        executable(&path, "old");
        let stale = inspect();
        let moved = home.path().join("moved");
        fs::rename(&bin, &moved).unwrap();
        symlink(&moved, &bin).unwrap();
        assert!(Destination::inspect(home.path(), &bundled).is_err());
        assert!(stale.commit(&bundled, true).is_err());
        assert_eq!(fs::read_to_string(moved.join(COMMAND)).unwrap(), "old");
    }

    #[test]
    fn verifies_full_cli_version_before_switching() {
        let (home, bundled) = fixture();
        let path = home.path().join(".local/bin/atlas");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        executable(&path, "old");
        executable(
            &bundled.path,
            "#!/bin/sh\nprintf '%s' '{\"status\":\"ok\",\"data\":{\"version\":\"0.1.0\",\"variant\":\"full\"}}'\n",
        );
        assert_eq!(
            verify_command(&bundled.path, Some("0.1.0")).unwrap(),
            "0.1.0"
        );
        assert!(verify_command(&bundled.path, Some("0.1.1")).is_err());
        let destination = Destination::inspect(home.path(), &bundled).unwrap();
        assert!(destination.commit(&bundled, true).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "old");
    }

    #[test]
    fn rejects_a_redirected_sidecar_before_running_it() {
        let (home, bundled) = fixture();
        let redirected = home.path().join("atlas");
        symlink(&bundled.path, &redirected).unwrap();
        assert!(
            Bundled::inspect_path(&redirected)
                .err()
                .unwrap()
                .contains("not a regular executable")
        );
    }
}
