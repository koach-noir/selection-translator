pub type MouseUpCallback = Box<dyn Fn(f64, f64) + Send + 'static>;

pub trait MouseHook: Send + 'static {
    fn start(&self, on_mouse_up: MouseUpCallback);
    fn stop(&self);
}

pub trait SelectionCapture: Send + Sync {
    fn capture(&self) -> Result<Option<String>, String>;
}

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{MacMouseHook as PlatformMouseHook, MacSelectionCapture as PlatformSelectionCapture};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::{WinMouseHook as PlatformMouseHook, WinSelectionCapture as PlatformSelectionCapture};
