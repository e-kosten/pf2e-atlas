//! App-wide WebView zoom and its local preference.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU8, Ordering},
};

use tauri::{AppHandle, Manager, WebviewWindow};

use crate::{Desktop, display};

const LEVELS: [u8; 10] = [50, 67, 80, 90, 100, 110, 125, 150, 175, 200];

pub struct Zoom(AtomicU8);

impl Default for Zoom {
    fn default() -> Self {
        Self(AtomicU8::new(100))
    }
}

impl Zoom {
    pub fn load(&self, app: &AppHandle) {
        if let Ok(path) = path(app) {
            self.0.store(read(&path), Ordering::Relaxed);
        }
    }

    pub fn apply(&self, window: &WebviewWindow) -> tauri::Result<()> {
        window.set_zoom(f64::from(self.0.load(Ordering::Relaxed)) / 100.0)
    }

    pub fn change(&self, app: &AppHandle, action: &str) -> Result<(), String> {
        let level = step(self.0.load(Ordering::Relaxed), action);
        self.0.store(level, Ordering::Relaxed);
        let mut result = Ok(());
        for window in app.webview_windows().values() {
            if let Err(error) = self.apply(window) {
                result = Err(display(error));
            }
        }
        save(&path(app)?, level).map_err(display)?;
        result
    }
}

pub fn apply(window: &WebviewWindow) {
    if let Err(error) = window.state::<Desktop>().zoom.apply(window) {
        eprintln!("Could not apply Atlas zoom: {error}");
    }
}

fn path(app: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    if let Some(home) = std::env::var_os("ATLAS_PILOT_HOME") {
        return Ok(PathBuf::from(home).join("desktop-zoom"));
    }
    Ok(app.path().app_config_dir().map_err(display)?.join("zoom"))
}

fn read(path: &Path) -> u8 {
    fs::read_to_string(path)
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .filter(|level| LEVELS.contains(level))
        .unwrap_or(100)
}

fn step(level: u8, action: &str) -> u8 {
    match action {
        "zoom-in" | "zoom-plus" => LEVELS
            .iter()
            .copied()
            .find(|next| *next > level)
            .unwrap_or(level),
        "zoom-out" => LEVELS
            .iter()
            .rev()
            .copied()
            .find(|next| *next < level)
            .unwrap_or(level),
        _ => 100,
    }
}

fn save(path: &Path, level: u8) -> std::io::Result<()> {
    let directory = path.parent().ok_or(std::io::ErrorKind::InvalidInput)?;
    fs::create_dir_all(directory)?;
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    writeln!(temporary, "{level}")?;
    temporary.as_file().sync_all()?;
    temporary.persist(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_steps_reset_and_stop_at_the_bounds() {
        assert_eq!(step(100, "zoom-in"), 110);
        assert_eq!(step(100, "zoom-out"), 90);
        assert_eq!(step(50, "zoom-out"), 50);
        assert_eq!(step(200, "zoom-in"), 200);
        assert_eq!(step(200, "zoom-reset"), 100);
    }

    #[test]
    fn zoom_reloads_and_invalid_preferences_default_to_actual_size() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("zoom");
        assert_eq!(read(&path), 100);
        save(&path, 125).unwrap();
        assert_eq!(read(&path), 125);
        save(&path, 90).unwrap();
        assert_eq!(read(&path), 90);
        fs::write(&path, "500").unwrap();
        assert_eq!(read(&path), 100);
    }
}
