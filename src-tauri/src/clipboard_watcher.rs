use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::AppHandle;

/// ポーリング間隔
const POLL_INTERVAL: Duration = Duration::from_millis(150);

/// ダブルコピー判定の閾値
const DOUBLE_COPY_THRESHOLD: Duration = Duration::from_millis(600);

/// クリップボード変更カウントを監視し、短時間内に2回変更されたら翻訳を起動する
pub fn start(app: AppHandle, running: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let mut last_count = get_change_count();
        let mut last_change_time: Option<Instant> = None;

        while running.load(Ordering::Relaxed) {
            std::thread::sleep(POLL_INTERVAL);

            let count = get_change_count();
            if count == last_count {
                continue;
            }

            let delta = count - last_count;
            last_count = count;

            // ポーリング間隔内に2回以上変更 → ダブルコピーと判定
            if delta >= 2 {
                trigger_translate(&app);
                last_change_time = None;
                continue;
            }

            // 1回の変更 → 前回変更からの経過時間で判定
            let now = Instant::now();
            if let Some(prev) = last_change_time {
                if now.duration_since(prev) < DOUBLE_COPY_THRESHOLD {
                    trigger_translate(&app);
                    last_change_time = None;
                    continue;
                }
            }
            last_change_time = Some(now);
        }

        log::info!("Clipboard watcher stopped");
    });

    log::info!(
        "Clipboard watcher started (poll={}ms, threshold={}ms)",
        POLL_INTERVAL.as_millis(),
        DOUBLE_COPY_THRESHOLD.as_millis(),
    );
}

fn trigger_translate(app: &AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        crate::handle_translate(&handle).await;
    });
}

// --- macOS: NSPasteboard.changeCount をObjective-Cランタイム経由で取得 ---

#[cfg(target_os = "macos")]
fn get_change_count() -> i64 {
    use std::ffi::{c_char, c_void};

    extern "C" {
        fn objc_getClass(name: *const c_char) -> *mut c_void;
        fn sel_registerName(name: *const c_char) -> *mut c_void;
        fn objc_msgSend();
    }

    type SendPtr = unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void;
    type SendI64 = unsafe extern "C" fn(*mut c_void, *mut c_void) -> i64;

    unsafe {
        let send_ptr: SendPtr = std::mem::transmute(objc_msgSend as *const ());
        let send_i64: SendI64 = std::mem::transmute(objc_msgSend as *const ());

        let cls = objc_getClass(b"NSPasteboard\0".as_ptr() as *const c_char);
        let sel_general = sel_registerName(b"generalPasteboard\0".as_ptr() as *const c_char);
        let pasteboard = send_ptr(cls, sel_general);

        if pasteboard.is_null() {
            return -1;
        }

        let sel_count = sel_registerName(b"changeCount\0".as_ptr() as *const c_char);
        send_i64(pasteboard, sel_count)
    }
}

// --- Windows: GetClipboardSequenceNumber ---

#[cfg(target_os = "windows")]
fn get_change_count() -> i64 {
    use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
    unsafe { GetClipboardSequenceNumber() as i64 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_count_is_non_negative() {
        let count = get_change_count();
        assert!(count >= 0, "changeCount should be non-negative: {}", count);
    }
}
