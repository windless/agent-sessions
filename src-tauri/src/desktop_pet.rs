use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};

/// Create the desktop pet floating window.
/// The window is frameless, transparent, always-on-top, and skips the taskbar.
/// If creation fails (e.g., unsupported platform), it fails silently — the
/// pet is an optional enhancement, not a core feature.
pub fn create_pet_window(app: &AppHandle) {
    let result = WebviewWindowBuilder::new(app, "pet", WebviewUrl::App("pet.html".into()))
        .title("Desktop Pet")
        .inner_size(128.0, 128.0)
        .min_inner_size(64.0, 64.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(true)
        .resizable(false)
        .build();

    match result {
        Ok(window) => {
            log::info!("Desktop pet window created");
            // Position in bottom-right corner of primary monitor
            if let Some(monitor) = window.primary_monitor().ok().flatten() {
                let size = monitor.size();
                let scale = monitor.scale_factor();
                let x = ((size.width as f64 / scale) - 128.0).max(0.0);
                let y = ((size.height as f64 / scale) - 128.0).max(0.0);
                let _ = window.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
            }
        }
        Err(e) => {
            log::warn!("Failed to create desktop pet window (non-fatal): {}", e);
        }
    }
}
