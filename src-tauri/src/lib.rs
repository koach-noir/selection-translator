mod clipboard_watcher;
mod commands;
mod config;
mod history;
mod translate;
mod tray;
mod validate;

use commands::AppState;
use history::{TranslationHistory, append_to_session};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Manager};
use translate::TranslationService;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = config::load_config();
    let enabled = config.enabled;
    let watcher_running = Arc::new(AtomicBool::new(true));

    let app_state = AppState {
        config: Mutex::new(config),
        translation_service: tokio::sync::Mutex::new(TranslationService::new()),
        enabled: Mutex::new(enabled),
        history: Mutex::new(TranslationHistory::new()),
        popup_session: Mutex::new(Vec::new()),
    };

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .setup({
            let running = watcher_running.clone();
            move |app| {
                tray::setup_tray(app)?;
                clipboard_watcher::start(app.handle().clone(), running);
                Ok(())
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::update_config,
            commands::get_enabled,
            commands::set_enabled,
            commands::translate_text,
            commands::get_popup_entries,
            commands::dismiss_popup_entry,
            commands::clear_popup_session,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(move |_app, event| {
        match event {
            // ウィンドウが全て閉じてもアプリを終了しない（トレイ常駐）
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            tauri::RunEvent::Exit => {
                watcher_running.store(false, Ordering::Relaxed);
            }
            _ => {}
        }
    });
}

pub(crate) async fn handle_translate(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();

    if !*state.enabled.lock().unwrap() {
        return;
    }

    let text = match read_clipboard() {
        Some(t) => t,
        None => return,
    };

    let config = state.config.lock().unwrap().clone();

    if !validate::is_valid_selection(&text, &config) {
        return;
    }

    let translated = {
        let mut service = state.translation_service.lock().await;
        service.translate(&text).await
    };

    let translated = match translated {
        Ok(t) => t,
        Err(e) => {
            log::error!("Translation failed: {}", e);
            return;
        }
    };

    // 履歴に追加
    let entry = {
        let mut history = state.history.lock().unwrap();
        let (new_history, entry) = history.add(text, translated);
        *history = new_history;
        entry
    };

    // ポップアップセッション更新
    {
        let mut session = state.popup_session.lock().unwrap();
        let popup_exists = app.get_webview_window("popup").is_some();
        if popup_exists {
            *session = append_to_session(&session, entry.clone());
        } else {
            *session = vec![entry.clone()];
        }
    }

    // ポップアップを表示（非表示なら再表示、無ければ新規作成）
    if let Err(e) = ensure_popup_visible(app) {
        log::error!("Popup error: {}", e);
        return;
    }

    // イベントでフロントエンドに通知
    if let Err(e) = app.emit_to("popup", "translation-new", &entry) {
        log::error!("Event emit error: {}", e);
    }

    // トレイメニュー再構築
    tray::rebuild_tray_menu(app);
}

fn read_clipboard() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    let text = clipboard.get_text().ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

fn ensure_popup_visible(app: &tauri::AppHandle) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

    // 既存ウィンドウがあれば表示してフォーカス
    if let Some(window) = app.get_webview_window("popup") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    // macOS WKWebView は visible(false) / 極小サイズだとコンテンツを読み込まない
    // 通常サイズで作成し、JS が即座にサイズ・位置を調整する
    WebviewWindowBuilder::new(app, "popup", WebviewUrl::App("/popup/index.html".into()))
        .title("")
        .inner_size(400.0, 300.0)
        .position(100.0, 100.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(true)
        .build()
        .map_err(|e: tauri::Error| e.to_string())?;

    Ok(())
}
