use super::{MouseHook, MouseUpCallback, SelectionCapture};
use arboard::Clipboard;
use rdev::{listen, EventType};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct WinMouseHook {
    running: Arc<AtomicBool>,
}

impl WinMouseHook {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl MouseHook for WinMouseHook {
    fn start(&self, on_mouse_up: MouseUpCallback) {
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        thread::spawn(move || {
            let callback = move |event: rdev::Event| {
                if !running.load(Ordering::SeqCst) {
                    return;
                }
                if let EventType::ButtonRelease(rdev::Button::Left) = event.event_type {
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

pub struct WinSelectionCapture;

impl WinSelectionCapture {
    pub fn new() -> Self {
        Self
    }
}

impl SelectionCapture for WinSelectionCapture {
    fn capture(&self) -> Result<Option<String>, String> {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
            KEYEVENTF_KEYUP, VIRTUAL_KEY,
        };

        let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;

        // クリップボード事前保存
        let saved = clipboard.get_text().ok();

        // Ctrl+C シミュレーション
        let vk_control = VIRTUAL_KEY(0x11); // VK_CONTROL
        let vk_c = VIRTUAL_KEY(0x43); // 'C'

        let inputs = [
            make_key_input(vk_control, KEYBD_EVENT_FLAGS(0)),
            make_key_input(vk_c, KEYBD_EVENT_FLAGS(0)),
            make_key_input(vk_c, KEYEVENTF_KEYUP),
            make_key_input(vk_control, KEYEVENTF_KEYUP),
        ];

        unsafe {
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
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

#[cfg(target_os = "windows")]
fn make_key_input(
    vk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY,
    flags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS,
) -> windows::Win32::UI::Input::KeyboardAndMouse::INPUT {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
