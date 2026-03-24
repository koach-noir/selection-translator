use crate::commands::AppState;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_base_menu(app.handle())?;

    TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip("Selection Translator")
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(|tray: &tauri::tray::TrayIcon, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let state = app.state::<AppState>();
                let mut enabled = state.enabled.lock().unwrap();
                *enabled = !*enabled;
                log::info!("Tray click toggle: {}", if *enabled { "ON" } else { "OFF" });
            }
        })
        .build(app)?;

    Ok(())
}

pub fn rebuild_tray_menu(app: &tauri::AppHandle) {
    let menu = match build_menu_with_history(app) {
        Ok(m) => m,
        Err(e) => {
            log::error!("Failed to build tray menu: {}", e);
            return;
        }
    };

    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_menu(Some(menu));
    }
}

fn build_base_menu(app: &tauri::AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let toggle_item = MenuItem::with_id(app, "toggle", "Toggle ON/OFF", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&toggle_item, &settings_item, &quit_item])?;
    Ok(menu)
}

fn build_menu_with_history(
    app: &tauri::AppHandle,
) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let state = app.state::<AppState>();
    let history = state.history.lock().unwrap();
    let entries = history.recent(20);
    drop(history);

    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = Vec::new();

    // 履歴を新しい順に表示
    for entry in entries.iter().rev() {
        let label = format_history_label(&entry.original, &entry.translated);
        let id = format!("history_{}", entry.id);
        let item = MenuItem::with_id(app, &id, &label, true, None::<&str>)?;
        items.push(Box::new(item));
    }

    if !entries.is_empty() {
        items.push(Box::new(PredefinedMenuItem::separator(app)?));
    }

    let toggle_item = MenuItem::with_id(app, "toggle", "Toggle ON/OFF", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    items.push(Box::new(toggle_item));
    items.push(Box::new(settings_item));
    items.push(Box::new(quit_item));

    let item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
        items.iter().map(|i| i.as_ref()).collect();

    let menu = Menu::with_items(app, &item_refs)?;
    Ok(menu)
}

fn format_history_label(original: &str, translated: &str) -> String {
    let orig_short = truncate_str(original, 15);
    let trans_short = truncate_str(translated, 20);
    format!("{} → {}", orig_short, trans_short)
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    let trimmed: String = s.chars().take(max_chars).collect();
    if s.chars().count() > max_chars {
        format!("{}…", trimmed)
    } else {
        trimmed
    }
}

pub(crate) fn handle_menu_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id.as_ref();

    if let Some(entry_id_str) = id.strip_prefix("history_") {
        if let Ok(entry_id) = entry_id_str.parse::<u64>() {
            copy_history_entry(app, entry_id);
        }
        return;
    }

    match id {
        "toggle" => {
            let state = app.state::<AppState>();
            let mut enabled = state.enabled.lock().unwrap();
            *enabled = !*enabled;
            log::info!("Toggle: {}", if *enabled { "ON" } else { "OFF" });
        }
        "settings" => {
            if let Some(window) = app.get_webview_window("settings") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

fn copy_history_entry(app: &tauri::AppHandle, entry_id: u64) {
    let state = app.state::<AppState>();
    let history = state.history.lock().unwrap();

    if let Some(entry) = history.find_by_id(entry_id) {
        let text = entry.translated.clone();
        drop(history);

        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(text) {
                    log::error!("Clipboard write failed: {}", e);
                }
            }
            Err(e) => {
                log::error!("Clipboard init failed: {}", e);
            }
        }
    }
}
