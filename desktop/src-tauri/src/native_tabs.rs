//! AppKit window grouping. All calls run on the native main thread.

#![allow(unsafe_code)]

use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSAlert, NSAlertFirstButtonReturn, NSAppearance, NSAppearanceCustomization, NSApplication,
    NSColor, NSEventModifierFlags, NSWindow, NSWindowOrderingMode, NSWindowTabbingMode,
};
use objc2_foundation::NSString;
use tauri::{Manager, WebviewWindow};

use crate::{current_product, display, product_view, run_on_main};

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    System,
    Light,
    Dark,
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
        if !product_view(&window, &base, &instance)
            && !(label == crate::opening::PRIMARY
                && window
                    .state::<crate::Desktop>()
                    .staged_setup
                    .attached_to(&label, &instance)
                && window.url().is_ok_and(|url| url.origin() == base.origin()))
        {
            return;
        }
        let _ = with_window(&window, |native| {
            let name = match appearance {
                Appearance::System => None,
                Appearance::Light => Some("NSAppearanceNameAqua"),
                Appearance::Dark => Some("NSAppearanceNameDarkAqua"),
            };
            let appearance =
                name.and_then(|name| NSAppearance::appearanceNamed(&NSString::from_str(name)));
            native.setAppearance(appearance.as_deref());
            let (red, green, blue) = if dark { (14, 13, 12) } else { (246, 239, 227) };
            let background = NSColor::colorWithSRGBRed_green_blue_alpha(
                f64::from(red) / 255.0,
                f64::from(green) / 255.0,
                f64::from(blue) / 255.0,
                1.0,
            );
            native.setBackgroundColor(Some(&background));
        });
    })
}

pub fn disable_automatic_grouping() -> Result<(), String> {
    NSWindow::setAllowsAutomaticWindowTabbing(false, main_thread()?);
    Ok(())
}

pub fn configure_zoom_shortcut() {
    let item = MainThreadMarker::new()
        .and_then(|main| NSApplication::sharedApplication(main).mainMenu())
        .and_then(|menu| menu.itemWithTitle(&NSString::from_str("View")))
        .and_then(|item| item.submenu())
        .and_then(|menu| menu.itemWithTitle(&NSString::from_str("Zoom In (+)")));
    if let Some(item) = item {
        item.setKeyEquivalent(&NSString::from_str("+"));
        item.setKeyEquivalentModifierMask(NSEventModifierFlags::Command);
        item.setAllowsKeyEquivalentWhenHidden(true);
        item.setHidden(true);
    }
}

pub fn configure(window: &WebviewWindow, product: bool) -> Result<(), String> {
    with_window(window, |native| {
        native.setTabbingMode(if product {
            NSWindowTabbingMode::Automatic
        } else {
            NSWindowTabbingMode::Disallowed
        });
        if product {
            native.setTabbingIdentifier(&NSString::from_str("atlas-product"));
        }
    })
}

pub fn attach(
    source: &WebviewWindow,
    target: &WebviewWindow,
    foreground: bool,
) -> Result<(), String> {
    with_window(source, |source| {
        with_window(target, |target| {
            target.setAppearance(source.appearance().as_deref());
            target.setBackgroundColor(Some(&source.backgroundColor()));
            source.addTabbedWindow_ordered(target, NSWindowOrderingMode::Above);
            let group = source
                .tabGroup()
                .ok_or("Could not create an Atlas tab group.")?;
            if !group
                .windows()
                .iter()
                .any(|window| std::ptr::eq(&*window, target))
            {
                return Err("Could not attach the new Atlas tab.".into());
            }
            if !foreground {
                group.setSelectedWindow(Some(source));
                source.makeKeyAndOrderFront(None);
            }
            Ok::<(), String>(())
        })?
    })??;
    if foreground {
        select(target)?;
    }
    Ok(())
}

pub fn select(window: &WebviewWindow) -> Result<(), String> {
    let main = main_thread()?;
    with_window(window, |native| {
        let app = NSApplication::sharedApplication(main);
        app.unhide(None);
        if let Some(group) = native.tabGroup() {
            group.setSelectedWindow(Some(native));
        }
        native.deminiaturize(None);
        native.makeKeyAndOrderFront(None);
        #[allow(deprecated)]
        app.activateIgnoringOtherApps(true);
    })
}

#[cfg(debug_assertions)]
pub fn verify_group(source: &WebviewWindow, tab: &WebviewWindow) -> Result<(), String> {
    with_window(source, |source| {
        with_window(tab, |tab| {
            let group = source.tabGroup().ok_or("No native tab group")?;
            if group.windows().len() != 2
                || group
                    .selectedWindow()
                    .as_deref()
                    .is_none_or(|selected| !std::ptr::eq(selected, tab))
            {
                return Err("The expected tab was not selected in the original window".into());
            }
            Ok(())
        })?
    })?
}

pub fn report(message: &str, detail: &str) {
    if let Some(alert) = alert(message, detail) {
        alert.runModal();
    }
}

pub fn offer_retry(message: &str, detail: &str) -> bool {
    alert(message, detail).is_some_and(|alert| {
        alert.addButtonWithTitle(&NSString::from_str("Retry"));
        alert.addButtonWithTitle(&NSString::from_str("Close"));
        alert.runModal() == NSAlertFirstButtonReturn
    })
}

fn alert(message: &str, detail: &str) -> Option<objc2::rc::Retained<NSAlert>> {
    eprintln!("{message}: {detail}");
    let alert = NSAlert::new(main_thread().ok()?);
    alert.setMessageText(&NSString::from_str(message));
    alert.setInformativeText(&NSString::from_str(detail));
    Some(alert)
}

fn main_thread() -> Result<MainThreadMarker, String> {
    MainThreadMarker::new()
        .ok_or_else(|| "Atlas window placement requires the macOS main thread.".into())
}

fn with_window<T>(
    window: &WebviewWindow,
    action: impl FnOnce(&NSWindow) -> T,
) -> Result<T, String> {
    main_thread()?;
    let native = window.ns_window().map_err(display)?.cast::<NSWindow>();
    // Tauri owns the NSWindow. Use the borrowed pointer synchronously on the
    // main thread without keeping it beyond this call.
    let native = unsafe { native.as_ref() }.ok_or("The Atlas window is unavailable.")?;
    Ok(action(native))
}
