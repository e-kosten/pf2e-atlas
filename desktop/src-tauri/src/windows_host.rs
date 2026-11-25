//! Native window and shell operations on Windows.

#![allow(unsafe_code)]

use std::ptr::null_mut;

use tauri::{AppHandle, Manager, WebviewWindow};
use url::Url;
use windows_sys::Win32::{
    System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW},
    UI::{
        Shell::ShellExecuteW,
        WindowsAndMessaging::{
            IDRETRY, IDYES, MB_DEFBUTTON2, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONWARNING, MB_OK,
            MB_RETRYCANCEL, MB_YESNO, MESSAGEBOX_STYLE, MessageBoxW, SW_SHOWNORMAL,
        },
    },
};

use crate::{
    appearance::{Appearance, canvas},
    display, process,
};

pub fn system_dark() -> bool {
    let key = wide(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
    let name = wide("AppsUseLightTheme");
    let mut light = 1u32;
    let mut size = size_of::<u32>() as u32;
    // The UTF-16 strings and DWORD buffer live for the duration of the call.
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_DWORD,
            null_mut(),
            (&mut light as *mut u32).cast(),
            &mut size,
        )
    };
    result == 0 && light == 0
}

pub fn apply_appearance(window: &WebviewWindow, _appearance: Appearance, dark: bool) {
    let _ = window.set_background_color(Some(canvas(dark)));
}

pub fn select(window: &WebviewWindow) -> Result<(), String> {
    if window.is_minimized().map_err(display)? {
        window.unminimize().map_err(display)?;
    }
    window
        .show()
        .and_then(|()| window.set_focus())
        .map_err(display)
}

pub fn open_external(destination: &Url) {
    let destination = wide(destination.as_str());
    // Pass the validated URL directly to the shell association API.
    unsafe {
        ShellExecuteW(
            null_mut(),
            std::ptr::null(),
            destination.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        );
    }
}

pub fn powershell_command() -> Result<String, String> {
    Ok(powershell_invocation(
        &process::bundled_cli()?.to_string_lossy(),
    ))
}

fn powershell_invocation(path: &str) -> String {
    let mut command = String::from("& '");
    for ch in path.chars() {
        // PowerShell recognizes all five characters as single-quote delimiters.
        if matches!(ch, '\'' | '\u{2018}'..='\u{201b}') {
            command.push(ch);
        }
        command.push(ch);
    }
    command.push('\'');
    command
}

pub fn show_cli_path(app: &AppHandle) -> Result<(), String> {
    let command = powershell_command()?;
    let mut details = format!("Run the bundled Atlas command in PowerShell.\n\n{command}");
    if let Some(workspace) = app.state::<crate::Desktop>().workspace_route.index()? {
        details.push_str(&format!(
            "\n\nThe app and this command use {}.",
            workspace.display()
        ));
    }
    message(
        "Terminal and Agent Access",
        &details,
        MB_OK | MB_ICONINFORMATION,
    );
    Ok(())
}

pub fn report(title: &str, detail: &str) {
    message(title, detail, MB_OK | MB_ICONERROR);
}

pub fn confirm_exit() -> bool {
    message(
        "Exit Atlas",
        "Discard unsaved changes in all open Atlas windows?",
        MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2,
    ) == IDYES
}

pub fn confirm_update() -> bool {
    message(
        "Install Atlas update",
        "Discard unsaved changes in all open Atlas windows and restart to install the update?",
        MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2,
    ) == IDYES
}

pub fn offer_retry(title: &str, detail: &str) -> bool {
    message(title, detail, MB_RETRYCANCEL | MB_ICONERROR) == IDRETRY
}

fn message(title: &str, detail: &str, style: MESSAGEBOX_STYLE) -> i32 {
    let title = wide(title);
    let detail = wide(detail);
    // Both buffers are NUL-terminated and stay alive until the dialog closes.
    unsafe { MessageBoxW(null_mut(), detail.as_ptr(), title.as_ptr(), style) }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

/// Restart Manager resolves the exact installed file, including sessions that
/// use a different workspace. This query never asks Windows to stop a process.
pub fn check_cli_sessions(launcher: u32, worker: Option<u32>) -> Result<(), String> {
    use std::time::{Duration, Instant};

    let cli = process::bundled_cli()?;
    let deadline = Instant::now() + Duration::from_secs(2);
    check_cli_sessions_at(&cli, launcher, worker, || {
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(150));
        true
    })
}

fn check_cli_sessions_at(
    cli: &std::path::Path,
    launcher: u32,
    worker: Option<u32>,
    mut retry: impl FnMut() -> bool,
) -> Result<(), String> {
    loop {
        let owners = cli_owners(cli)?;
        if owners
            .iter()
            .all(|&pid| pid == launcher || Some(pid) == worker)
        {
            return Ok(());
        }
        if !retry() {
            eprintln!(
                "Bundled Atlas command in use: launcher={launcher}, worker={worker:?}, reported PIDs={:?}",
                &owners[..owners.len().min(8)]
            );
            return Err("Close the terminal session using the bundled Atlas command, then retry the update.".into());
        }
    }
}

// A Restart Manager session remembers affected processes even after they exit.
// Register the resource in a fresh session for each ownership sample.
fn cli_owners(cli: &std::path::Path) -> Result<Vec<u32>, String> {
    use std::{os::windows::ffi::OsStrExt, ptr::null};
    use windows_sys::Win32::{
        Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS},
        System::RestartManager::{
            CCH_RM_SESSION_KEY, RM_PROCESS_INFO, RmEndSession, RmGetList, RmRegisterResources,
            RmStartSession,
        },
    };

    struct Session(u32);
    impl Drop for Session {
        fn drop(&mut self) {
            unsafe { RmEndSession(self.0) };
        }
    }
    let check = |status| {
        if status == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(format!(
                "Atlas could not check which processes are using the bundled command: {}",
                std::io::Error::from_raw_os_error(status as i32)
            ))
        }
    };
    let mut handle = 0;
    let mut key = [0u16; CCH_RM_SESSION_KEY as usize + 1];
    // The session key, path and process buffers live across their respective calls.
    check(unsafe { RmStartSession(&mut handle, 0, key.as_mut_ptr()) })?;
    let session = Session(handle);
    let cli: Vec<_> = cli.as_os_str().encode_wide().chain([0]).collect();
    check(unsafe { RmRegisterResources(session.0, 1, &cli.as_ptr(), 0, null(), 0, null()) })?;
    let mut processes = vec![RM_PROCESS_INFO::default(); 8];
    loop {
        let mut count = processes.len() as u32;
        let mut needed = 0;
        let mut reasons = 0;
        let result = unsafe {
            RmGetList(
                session.0,
                &mut needed,
                &mut count,
                processes.as_mut_ptr(),
                &mut reasons,
            )
        };
        if result == ERROR_MORE_DATA {
            processes.resize(needed as usize, RM_PROCESS_INFO::default());
            continue;
        }
        check(result)?;
        return Ok(processes[..count as usize]
            .iter()
            .map(|entry| entry.Process.dwProcessId)
            .collect());
    }
}

/// The last owned worker may still be draining after its launcher exits. An
/// independent lease keeps the executable mapped beyond this grace period.
pub fn wait_for_cli_release() -> Result<(), String> {
    use std::time::{Duration, Instant};
    let path = process::bundled_cli()?;
    let deadline = Instant::now() + Duration::from_secs(7);
    loop {
        match std::fs::OpenOptions::new().write(true).open(&path) {
            Ok(_) => return Ok(()),
            Err(error) if matches!(error.raw_os_error(), Some(5 | 32 | 1224)) => {
                if Instant::now() >= deadline {
                    return Err("Atlas could not replace its bundled command. Close any terminal sessions using it, check its permissions, and retry.".into());
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(error) => {
                return Err(format!(
                    "Atlas could not check its bundled command: {error}"
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf, process::Command};

    #[test]
    fn cli_guard_refuses_a_live_holder_and_rechecks_after_it_exits() {
        let dir = tempfile::tempdir().unwrap();
        let executable = dir.path().join("atlas.exe");
        let system = PathBuf::from(std::env::var_os("SystemRoot").unwrap());
        fs::copy(system.join("System32/findstr.exe"), &executable).unwrap();
        let mut child = Command::new(&executable)
            .arg("atlas")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        assert!(check_cli_sessions_at(&executable, 0, None, || false).is_err());
        let mut retried = false;
        check_cli_sessions_at(&executable, 0, None, || {
            assert!(!retried, "reported a holder after its process exited");
            retried = true;
            drop(child.stdin.take());
            child.wait().unwrap();
            true
        })
        .unwrap();
        assert!(retried);
    }

    #[test]
    fn copied_command_runs_from_a_quoted_install_path() {
        let dir = tempfile::tempdir().unwrap();
        let executable = dir.path().join("Atlas O'Brien ‘one’ ‚two‛ & $test.exe");
        let system = PathBuf::from(std::env::var_os("SystemRoot").unwrap());
        fs::copy(system.join("System32/where.exe"), &executable).unwrap();
        fs::write(dir.path().join("atlas-marker.txt"), b"").unwrap();

        let command = powershell_invocation(executable.to_str().unwrap());
        for shell in ["powershell.exe", "pwsh.exe"] {
            let result = Command::new(shell)
                .args(["-NoProfile", "-NonInteractive", "-Command"])
                .arg(format!("$ErrorActionPreference = 'Stop'; {command} /q atlas-marker.txt; exit $LASTEXITCODE"))
                .current_dir(dir.path())
                .output()
                .unwrap();
            assert!(result.status.success(), "{shell}: {:?}", result);
        }
    }
}
