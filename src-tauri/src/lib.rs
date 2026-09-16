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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_network_snapshot,
            get_history_logs,
            clear_history_logs,
            drag_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
