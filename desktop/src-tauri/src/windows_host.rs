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
    let path = process::bundled_cli()?
        .display()
        .to_string()
        .replace('\'', "''");
    Ok(format!("& '{path}'"))
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
