use super::{MouseHook, MouseUpCallback, SelectionCapture};
use arboard::Clipboard;
use rdev::{listen, EventType};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct MacMouseHook {
    running: Arc<AtomicBool>,
}

impl MacMouseHook {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl MouseHook for MacMouseHook {
    fn start(&self, on_mouse_up: MouseUpCallback) {
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        thread::spawn(move || {
            let callback = move |event: rdev::Event| {
                if !running.load(Ordering::SeqCst) {
                    return;
                }
                if let EventType::ButtonRelease(rdev::Button::Left) = event.event_type {
                    // rdevはマウス座標をイベント名に含まない場合がある
                    // 別途取得する必要がある場合あり
                    on_mouse_up(0.0, 0.0);
                }
            };

            if let Err(e) = listen(callback) {
                log::error!("Mouse hook error: {:?}", e);
            }
        });
    }

    fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

pub struct MacSelectionCapture;

impl MacSelectionCapture {
    pub fn new() -> Self {
        Self
    }
}

impl SelectionCapture for MacSelectionCapture {
    fn capture(&self) -> Result<Option<String>, String> {
        let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;

        // クリップボード事前保存
        let saved = clipboard.get_text().ok();

        // Cmd+C シミュレーション (AppleScript経由)
        let output = Command::new("osascript")
            .arg("-e")
            .arg(r#"tell application "System Events" to keystroke "c" using command down"#)
            .output()
            .map_err(|e| format!("Failed to simulate Cmd+C: {}", e))?;

        if !output.status.success() {
            return Err("osascript failed".to_string());
        }

        // 選択確定待ち
        thread::sleep(Duration::from_millis(150));

        // クリップボード取得
        let current = clipboard.get_text().ok();

        // クリップボード復元
        if let Some(ref saved_text) = saved {
            let _ = clipboard.set_text(saved_text.clone());
        }

        // 変更検知
        match (&saved, &current) {
            (Some(s), Some(c)) if s != c => Ok(Some(c.clone())),
            (None, Some(c)) => Ok(Some(c.clone())),
            _ => Ok(None),
        }
    }
}
