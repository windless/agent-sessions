use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct PetPosition {
    x: i32,
    y: i32,
}

fn position_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("agent-sessions")
        .join("pet_position.json")
}

fn load_position() -> Option<PetPosition> {
    let content = fs::read_to_string(position_path()).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_position(x: i32, y: i32) {
    let path = position_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string(&PetPosition { x, y }) {
        let _ = fs::write(&path, json);
    }
}

/// Create the desktop pet floating window.
/// The window is frameless, transparent, always-on-top, and skips the taskbar.
/// Position is restored from the previous session, or defaults to bottom-right.
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

            // Restore saved position, or default to bottom-right of primary monitor
            if let Some(saved) = load_position() {
                let _ = window.set_position(tauri::PhysicalPosition::new(saved.x, saved.y));
                log::info!("Pet position restored: ({}, {})", saved.x, saved.y);
            } else if let Some(monitor) = window.primary_monitor().ok().flatten() {
                let size = monitor.size();
                let scale = monitor.scale_factor();
                let x = ((size.width as f64 / scale) - 128.0).max(0.0);
                let y = ((size.height as f64 / scale) - 128.0).max(0.0);
                let _ = window.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
            }

            // Persist position on every move so it survives restarts
            window.on_window_event(|event| {
                if let tauri::WindowEvent::Moved(position) = event {
                    save_position(position.x, position.y);
                }
            });
        }
        Err(e) => {
            log::warn!("Failed to create desktop pet window (non-fatal): {}", e);
        }
    }
}
