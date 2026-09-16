mod network;
pub mod db;

use network::{get_live_connections, NetworkSnapshot};

#[tauri::command]
async fn get_network_snapshot() -> NetworkSnapshot {
    tokio::task::spawn_blocking(get_live_connections)
        .await
        .unwrap_or_else(|_| get_live_connections())
}

#[tauri::command]
fn get_history_logs(limit: Option<usize>) -> Vec<db::HistoryRecord> {
    db::fetch_history(limit.unwrap_or(200))
}

#[tauri::command]
fn clear_history_logs() -> bool {
    db::delete_all_history().is_ok()
}

#[tauri::command]
fn drag_window(window: tauri::WebviewWindow) {
    let _ = window.start_dragging();
}

// ─── Settings Commands ────────────────────────────────────────────────────────

#[tauri::command]
fn get_setting(key: String) -> Option<String> {
    db::get_setting(&key)
}

#[tauri::command]
fn set_setting(key: String, value: String) -> bool {
    db::set_setting(&key, &value)
}

/// Run auto-cleanup based on `retention_days` setting stored in DB.
/// Returns the number of rows deleted.
#[tauri::command]
fn run_cleanup() -> u64 {
    let retention_days: i64 = db::get_setting("retention_days")
        .and_then(|v| v.parse().ok())
        .unwrap_or(30); // default: 30 days
    db::cleanup_old_history(retention_days)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_network_snapshot,
            get_history_logs,
            clear_history_logs,
            drag_window,
            get_setting,
            set_setting,
            run_cleanup
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
