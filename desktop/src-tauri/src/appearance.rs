use tauri::{AppHandle, Manager, WebviewWindow, window::Color};

use crate::{Desktop, current_product, lock, run_on_main, window_host::system_dark};

#[derive(Clone, Copy, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    #[default]
    System,
    Light,
    Dark,
}

pub fn canvas(dark: bool) -> Color {
    if dark {
        Color(14, 13, 12, 255)
    } else {
        Color(246, 239, 227, 255)
    }
}

pub fn product_dark(app: &AppHandle) -> bool {
    match *lock(&app.state::<Desktop>().appearance) {
        Appearance::Dark => true,
        Appearance::Light => false,
        Appearance::System => system_dark(),
    }
}

#[tauri::command]
pub async fn desktop_appearance(
    window: WebviewWindow,
    appearance: Appearance,
    dark: bool,
) -> Result<(), String> {
    let label = window.label().to_owned();
    run_on_main(window.app_handle(), move |app| {
        let Ok((base, instance)) = current_product(app) else {
            return;
        };
        let Some(window) = app.get_webview_window(&label) else {
            return;
        };
        if !window
            .state::<Desktop>()
            .staged_setup
            .attached_to(&label, &instance)
            || !window.url().is_ok_and(|url| url.origin() == base.origin())
        {
            return;
        }
        *lock(&app.state::<Desktop>().appearance) = appearance;
        crate::window_host::apply_appearance(&window, appearance, dark);
    })
}
