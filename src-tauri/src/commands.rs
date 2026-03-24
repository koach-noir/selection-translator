use crate::config::{self, Config};
use crate::history::{TranslationEntry, TranslationHistory};
use crate::translate::TranslationService;
use crate::validate;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::{Manager, State};

pub struct AppState {
    pub config: Mutex<Config>,
    pub translation_service: tokio::sync::Mutex<TranslationService>,
    pub enabled: Mutex<bool>,
    pub history: Mutex<TranslationHistory>,
    pub popup_session: Mutex<Vec<TranslationEntry>>,
}

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Config {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
pub fn update_config(state: State<AppState>, config: Config) -> Result<(), String> {
    config::save_config(&config)?;
    *state.config.lock().unwrap() = config;
    Ok(())
}

#[tauri::command]
pub fn get_enabled(state: State<AppState>) -> bool {
    *state.enabled.lock().unwrap()
}

#[tauri::command]
pub fn set_enabled(state: State<AppState>, enabled: bool) {
    *state.enabled.lock().unwrap() = enabled;
}

#[tauri::command]
pub async fn translate_text(
    state: State<'_, AppState>,
    text: String,
) -> Result<Option<String>, String> {
    {
        let config = state.config.lock().unwrap().clone();
        if !validate::is_valid_selection(&text, &config) {
            return Ok(None);
        }
    }

    let mut service = state.translation_service.lock().await;
    let translated = service.translate(&text).await?;
    Ok(Some(translated))
}

#[tauri::command]
pub fn get_popup_entries(state: State<AppState>) -> Vec<TranslationEntry> {
    state.popup_session.lock().unwrap().clone()
}

#[tauri::command]
pub fn dismiss_popup_entry(state: State<AppState>, id: u64) -> usize {
    let mut session = state.popup_session.lock().unwrap();
    let new_session: Vec<TranslationEntry> = session.iter().filter(|e| e.id != id).cloned().collect();
    let remaining = new_session.len();
    *session = new_session;
    remaining
}

#[tauri::command]
pub fn clear_popup_session(state: State<AppState>) {
    let mut session = state.popup_session.lock().unwrap();
    *session = Vec::new();
}

#[tauri::command]
pub fn show_context_menu(window: tauri::Window) -> Result<(), String> {
    let app = window.app_handle();
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let separator = PredefinedMenuItem::separator(app)
        .map_err(|e| e.to_string())?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let menu = Menu::with_items(app, &[&settings, &separator, &quit])
        .map_err(|e| e.to_string())?;

    window.popup_menu(&menu).map_err(|e| e.to_string())
}
