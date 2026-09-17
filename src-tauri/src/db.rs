use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use lazy_static::lazy_static;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub id: String,
    pub timestamp: i64,
    pub process_name: String,
    pub remote_address: String,
    pub hostname: String,
    pub remote_port: u16,
    pub protocol: String,
    pub duration_seconds: u32,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedIpRecord {
    pub ip: String,
    pub hostname: String,
    pub process_name: String,
    pub blocked_at: i64,
}

struct DbManager {
    conn: Mutex<Connection>,
}

lazy_static! {
    static ref DB: DbManager = {
        let db_path = "net_tracker.db";
        let conn = Connection::open(db_path).expect("Failed to open SQLite database");

        // Enable WAL mode for better concurrent write performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .expect("Failed to set WAL mode");

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS connection_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                process_name TEXT NOT NULL,
                remote_address TEXT NOT NULL,
                hostname TEXT NOT NULL,
                remote_port INTEGER NOT NULL,
                protocol TEXT NOT NULL,
                duration_seconds INTEGER NOT NULL,
                total_bytes INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_history_time ON connection_history(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_history_host ON connection_history(hostname);

            CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS blocked_ips (
                ip TEXT PRIMARY KEY,
                hostname TEXT NOT NULL,
                process_name TEXT NOT NULL,
                blocked_at INTEGER NOT NULL
            );"
        ).expect("Failed to initialize SQLite tables");

        DbManager {
            conn: Mutex::new(conn),
        }
    };
}

// ─── Settings ────────────────────────────────────────────────────────────────

pub fn get_setting(key: &str) -> Option<String> {
    if let Ok(conn) = DB.conn.lock() {
        let result: Result<String> = conn.query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        result.ok()
    } else {
        None
    }
}

pub fn set_setting(key: &str, value: &str) -> bool {
    if let Ok(conn) = DB.conn.lock() {
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        ).is_ok()
    } else {
        false
    }
}

// ─── Auto Cleanup ─────────────────────────────────────────────────────────────

/// Delete history records older than `retention_days`.
/// If retention_days <= 0, no records are deleted (Forever).
/// Returns the number of rows deleted.
pub fn cleanup_old_history(retention_days: i64) -> u64 {
    if retention_days <= 0 {
        return 0;
    }

    let cutoff_ms = {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        now - retention_days * 24 * 3600 * 1000
    };

    if let Ok(conn) = DB.conn.lock() {
        conn.execute(
            "DELETE FROM connection_history WHERE timestamp < ?1",
            params![cutoff_ms],
        ).unwrap_or(0) as u64
    } else {
        0
    }
}

// ─── History CRUD ─────────────────────────────────────────────────────────────

pub fn save_connection_record(
    process_name: &str,
    remote_address: &str,
    hostname: &str,
    remote_port: u16,
    protocol: &str,
    duration_seconds: u32,
    total_bytes: u64,
) {
    if remote_address.is_empty() || remote_address == "*" || remote_address == "0.0.0.0" || remote_address == "127.0.0.1" {
        return;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    if let Ok(conn) = DB.conn.lock() {
        // Prevent spamming duplicate entries in short time for the same app + remote
        let check_stmt = conn.prepare(
            "SELECT id FROM connection_history 
             WHERE process_name = ?1 AND remote_address = ?2 
             AND timestamp > (?3 - 30000) 
             LIMIT 1"
        );

        if let Ok(mut stmt) = check_stmt {
            let exists = stmt.exists(params![process_name, remote_address, now]).unwrap_or(false);
            if exists {
                // Update total bytes and timestamp of recent session instead of inserting duplicate
                let _ = conn.execute(
                    "UPDATE connection_history 
                     SET total_bytes = total_bytes + ?1, timestamp = ?2, hostname = CASE WHEN ?3 != '' AND ?3 != remote_address THEN ?3 ELSE hostname END
                     WHERE process_name = ?4 AND remote_address = ?5 AND timestamp > (?2 - 30000)",
                    params![total_bytes, now, hostname, process_name, remote_address],
                );
                return;
            }
        }

        let _ = conn.execute(
            "INSERT INTO connection_history 
             (timestamp, process_name, remote_address, hostname, remote_port, protocol, duration_seconds, total_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                now,
                process_name,
                remote_address,
                hostname,
                remote_port,
                protocol,
                duration_seconds,
                total_bytes
            ],
        );
    }
}

pub fn fetch_history(limit: usize) -> Vec<HistoryRecord> {
    let mut records = Vec::new();
    if let Ok(conn) = DB.conn.lock() {
        let mut stmt = match conn.prepare(
            "SELECT id, timestamp, process_name, remote_address, hostname, remote_port, protocol, duration_seconds, total_bytes 
             FROM connection_history 
             ORDER BY timestamp DESC 
             LIMIT ?1"
        ) {
            Ok(s) => s,
            Err(_) => return records,
        };

        let rows = stmt.query_map(params![limit as i64], |row| {
            let id_num: i64 = row.get(0)?;
            Ok(HistoryRecord {
                id: id_num.to_string(),
                timestamp: row.get(1)?,
                process_name: row.get(2)?,
                remote_address: row.get(3)?,
                hostname: row.get(4)?,
                remote_port: row.get(5)?,
                protocol: row.get(6)?,
                duration_seconds: row.get(7)?,
                total_bytes: row.get(8)?,
            })
        });

        if let Ok(rows) = rows {
            for r in rows.flatten() {
                records.push(r);
            }
        }
    }
    records
}

pub fn delete_all_history() -> Result<()> {
    if let Ok(conn) = DB.conn.lock() {
        conn.execute("DELETE FROM connection_history", [])?;
    }
    Ok(())
}

// ─── Blocked IPs CRUD ─────────────────────────────────────────────────────────

pub fn add_blocked_ip(ip: &str, hostname: &str, process_name: &str) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    if let Ok(conn) = DB.conn.lock() {
        conn.execute(
            "INSERT INTO blocked_ips (ip, hostname, process_name, blocked_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(ip) DO UPDATE SET hostname = excluded.hostname, process_name = excluded.process_name, blocked_at = excluded.blocked_at",
            params![ip, hostname, process_name, now],
        ).is_ok()
    } else {
        false
    }
}

pub fn remove_blocked_ip(ip: &str) -> bool {
    if let Ok(conn) = DB.conn.lock() {
        conn.execute("DELETE FROM blocked_ips WHERE ip = ?1", params![ip]).is_ok()
    } else {
        false
    }
}

pub fn fetch_blocked_ips() -> Vec<BlockedIpRecord> {
    let mut list = Vec::new();
    if let Ok(conn) = DB.conn.lock() {
        let mut stmt = match conn.prepare("SELECT ip, hostname, process_name, blocked_at FROM blocked_ips ORDER BY blocked_at DESC") {
            Ok(s) => s,
            Err(_) => return list,
        };

        let rows = stmt.query_map([], |row| {
            Ok(BlockedIpRecord {
                ip: row.get(0)?,
                hostname: row.get(1)?,
                process_name: row.get(2)?,
                blocked_at: row.get(3)?,
            })
        });

        if let Ok(rows) = rows {
            for r in rows.flatten() {
                list.push(r);
            }
        }
    }
    list
}
