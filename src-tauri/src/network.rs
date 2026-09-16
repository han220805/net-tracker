use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable, GetIfTable2, FreeMibTable,
    MIB_TCPROW_OWNER_PID, MIB_TCPTABLE_OWNER_PID,
    MIB_UDPROW_OWNER_PID, MIB_UDPTABLE_OWNER_PID,
    MIB_IF_TABLE2, TCP_TABLE_OWNER_PID_ALL, UDP_TABLE_OWNER_PID,
};
use windows_sys::Win32::Networking::WinSock::AF_INET;
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::ProcessStatus::K32GetProcessImageFileNameW;
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};
use lazy_static::lazy_static;

struct TrafficState {
    last_in_bytes: u64,
    last_out_bytes: u64,
    last_timestamp: std::time::Instant,
}

#[derive(Debug, Clone, Default)]
struct SocketTraffic {
    bytes_in: u64,
    bytes_out: u64,
    speed_in: u64,
    speed_out: u64,
}

lazy_static! {
    static ref DNS_CACHE: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
    static ref PROCESS_CACHE: RwLock<HashMap<u32, String>> = RwLock::new(HashMap::new());
    static ref SOCKET_TRAFFIC: RwLock<HashMap<String, SocketTraffic>> = RwLock::new(HashMap::new());
    static ref TRAFFIC_STATE: RwLock<TrafficState> = RwLock::new(TrafficState {
        last_in_bytes: 0,
        last_out_bytes: 0,
        last_timestamp: std::time::Instant::now(),
    });
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub id: String,
    pub pid: u32,
    pub process_name: String,
    pub protocol: String,
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    pub hostname: String,
    pub state: String,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub country: Option<String>,
    pub first_seen: u64,
    pub last_active: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSummary {
    pub total_active_connections: usize,
    pub total_listening_ports: usize,
    pub total_processes: usize,
    pub total_download_speed: u64,
    pub total_upload_speed: u64,
    pub total_bytes_received: u64,
    pub total_bytes_sent: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSnapshot {
    pub connections: Vec<NetworkConnection>,
    pub summary: NetworkSummary,
}

fn tcp_state_to_string(state: u32) -> String {
    match state {
        1 => "CLOSED".to_string(),
        2 => "LISTEN".to_string(),
        3 => "SYN_SENT".to_string(),
        4 => "SYN_RCVD".to_string(),
        5 => "ESTABLISHED".to_string(),
        6 => "FIN_WAIT1".to_string(),
        7 => "FIN_WAIT2".to_string(),
        8 => "CLOSE_WAIT".to_string(),
        9 => "CLOSING".to_string(),
        10 => "LAST_ACK".to_string(),
        11 => "TIME_WAIT".to_string(),
        12 => "DELETE_TCB".to_string(),
        _ => "UNKNOWN".to_string(),
    }
}

fn resolve_process_name(pid: u32) -> String {
    if pid == 0 {
        return "System Idle Process".to_string();
    }
    if pid == 4 {
        return "System".to_string();
    }

    // 1. Check cache first
    if let Ok(cache) = PROCESS_CACHE.read() {
        if let Some(name) = cache.get(&pid) {
            return name.clone();
        }
    }

    // 2. Try OpenProcess + K32GetProcessImageFileNameW
    let mut resolved_name: Option<String> = None;
    unsafe {
        let handle: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if !handle.is_null() && handle != INVALID_HANDLE_VALUE {
            let mut buffer = [0u16; 1024];
            let len = K32GetProcessImageFileNameW(handle, buffer.as_mut_ptr(), buffer.len() as u32);
            if len > 0 {
                let path = String::from_utf16_lossy(&buffer[..len as usize]);
                if let Some(file_name) = path.split('\\').last() {
                    resolved_name = Some(file_name.to_string());
                }
            }
            CloseHandle(handle);
        }
    }

    // 3. Fallback to CreateToolhelp32Snapshot to resolve any protected / system apps
    if resolved_name.is_none() {
        unsafe {
            let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if !snap.is_null() && snap != INVALID_HANDLE_VALUE {
                let mut entry: PROCESSENTRY32W = std::mem::zeroed();
                entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

                if Process32FirstW(snap, &mut entry) != 0 {
                    loop {
                        let proc_name = {
                            let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                            String::from_utf16_lossy(&entry.szExeFile[..len])
                        };

                        if entry.th32ProcessID == pid {
                            resolved_name = Some(proc_name.clone());
                        }

                        // Populate cache with all processes seen
                        if let Ok(mut cache) = PROCESS_CACHE.write() {
                            cache.insert(entry.th32ProcessID, proc_name);
                        }

                        if Process32NextW(snap, &mut entry) == 0 {
                            break;
                        }
                    }
                }
                CloseHandle(snap);
            }
        }
    }

    let final_name = resolved_name.unwrap_or_else(|| format!("PID:{}", pid));
    if let Ok(mut cache) = PROCESS_CACHE.write() {
        cache.insert(pid, final_name.clone());
    }

    final_name
}

fn resolve_hostname(ip_str: &str) -> String {
    if ip_str.is_empty() || ip_str == "0.0.0.0" || ip_str == "127.0.0.1" || ip_str == "*" {
        return "localhost".to_string();
    }

    // Check cache first (fast path)
    if let Ok(cache) = DNS_CACHE.read() {
        if let Some(host) = cache.get(ip_str) {
            return host.clone();
        }
    }

    // Immediately insert IP to cache so we don't spawn duplicate threads for the same IP
    if let Ok(mut cache) = DNS_CACHE.write() {
        cache.insert(ip_str.to_string(), ip_str.to_string());
    }

    // Resolve in background thread without blocking the caller
    let ip_owned = ip_str.to_string();
    std::thread::spawn(move || {
        if let Ok(ip) = ip_owned.parse::<IpAddr>() {
            if let Ok(resolved) = dns_lookup::lookup_addr(&ip) {
                if let Ok(mut cache) = DNS_CACHE.write() {
                    cache.insert(ip_owned, resolved);
                }
            }
        }
    });

    ip_str.to_string()
}

fn calculate_traffic_stats() -> (u64, u64, u64, u64) {
    let mut total_in: u64 = 0;
    let mut total_out: u64 = 0;

    unsafe {
        let mut table_ptr: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
        if GetIfTable2(&mut table_ptr) == 0 && !table_ptr.is_null() {
            let num_entries = (*table_ptr).NumEntries as usize;
            let rows = (*table_ptr).Table.as_ptr();

            for i in 0..num_entries {
                let row = *rows.add(i);
                // Exclude loopback/software interfaces (MIB_IF_TYPE_LOOPBACK = 24)
                if row.Type != 24 {
                    total_in += row.InOctets;
                    total_out += row.OutOctets;
                }
            }
            FreeMibTable(table_ptr as *const _);
        }
    }

    let mut dl_speed = 0;
    let mut ul_speed = 0;

    if let Ok(mut state) = TRAFFIC_STATE.write() {
        let elapsed = state.last_timestamp.elapsed().as_secs_f64();
        if elapsed > 0.1 && state.last_in_bytes > 0 {
            if total_in >= state.last_in_bytes {
                dl_speed = ((total_in - state.last_in_bytes) as f64 / elapsed) as u64;
            }
            if total_out >= state.last_out_bytes {
                ul_speed = ((total_out - state.last_out_bytes) as f64 / elapsed) as u64;
            }
        }
        state.last_in_bytes = total_in;
        state.last_out_bytes = total_out;
        state.last_timestamp = std::time::Instant::now();
    }

    (dl_speed, ul_speed, total_in, total_out)
}

pub fn get_live_connections() -> NetworkSnapshot {
    let mut connections = Vec::new();
    let (dl_speed, ul_speed, total_bytes_in, total_bytes_out) = calculate_traffic_stats();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // 1. Fetch TCP Table
    unsafe {
        let mut size: u32 = 0;
        let _ = GetExtendedTcpTable(
            std::ptr::null_mut(),
            &mut size,
            0,
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if size > 0 {
            let mut buffer: Vec<u8> = vec![0; size as usize];
            let ret = GetExtendedTcpTable(
                buffer.as_mut_ptr() as *mut _,
                &mut size,
                0,
                AF_INET as u32,
                TCP_TABLE_OWNER_PID_ALL,
                0,
            );

            if ret == 0 {
                let table = buffer.as_ptr() as *const MIB_TCPTABLE_OWNER_PID;
                let num_entries = (*table).dwNumEntries as usize;
                let rows = (*table).table.as_ptr() as *const MIB_TCPROW_OWNER_PID;

                for i in 0..num_entries {
                    let row = *rows.add(i);
                    let local_ip = Ipv4Addr::from(u32::from_be(row.dwLocalAddr)).to_string();
                    let local_port = u16::from_be(row.dwLocalPort as u16);
                    let remote_ip = Ipv4Addr::from(u32::from_be(row.dwRemoteAddr)).to_string();
                    let remote_port = u16::from_be(row.dwRemotePort as u16);
                    let state_str = tcp_state_to_string(row.dwState);
                    let pid = row.dwOwningPid;
                    let process_name = resolve_process_name(pid);

                    let hostname = if state_str == "LISTEN" || remote_ip == "0.0.0.0" {
                        "".to_string()
                    } else {
                        resolve_hostname(&remote_ip)
                    };

                    let conn_id = format!("tcp-{}-{}-{}-{}-{}", pid, local_ip, local_port, remote_ip, remote_port);

                    connections.push(NetworkConnection {
                        id: conn_id,
                        pid,
                        process_name,
                        protocol: "TCP".to_string(),
                        local_address: local_ip,
                        local_port,
                        remote_address: remote_ip,
                        remote_port,
                        hostname,
                        state: state_str,
                        download_speed: 0,
                        upload_speed: 0,
                        bytes_received: 0,
                        bytes_sent: 0,
                        country: None,
                        first_seen: now,
                        last_active: now,
                    });
                }
            }
        }
    }

    // 2. Fetch UDP Table
    unsafe {
        let mut size: u32 = 0;
        let _ = GetExtendedUdpTable(
            std::ptr::null_mut(),
            &mut size,
            0,
            AF_INET as u32,
            UDP_TABLE_OWNER_PID,
            0,
        );

        if size > 0 {
            let mut buffer: Vec<u8> = vec![0; size as usize];
            let ret = GetExtendedUdpTable(
                buffer.as_mut_ptr() as *mut _,
                &mut size,
                0,
                AF_INET as u32,
                UDP_TABLE_OWNER_PID,
                0,
            );

            if ret == 0 {
                let table = buffer.as_ptr() as *const MIB_UDPTABLE_OWNER_PID;
                let num_entries = (*table).dwNumEntries as usize;
                let rows = (*table).table.as_ptr() as *const MIB_UDPROW_OWNER_PID;

                for i in 0..num_entries {
                    let row = *rows.add(i);
                    let local_ip = Ipv4Addr::from(u32::from_be(row.dwLocalAddr)).to_string();
                    let local_port = u16::from_be(row.dwLocalPort as u16);
                    let pid = row.dwOwningPid;
                    let process_name = resolve_process_name(pid);

                    let conn_id = format!("udp-{}-{}-{}", pid, local_ip, local_port);

                    connections.push(NetworkConnection {
                        id: conn_id,
                        pid,
                        process_name,
                        protocol: "UDP".to_string(),
                        local_address: local_ip,
                        local_port,
                        remote_address: "*".to_string(),
                        remote_port: 0,
                        hostname: "".to_string(),
                        state: "LISTEN".to_string(),
                        download_speed: 0,
                        upload_speed: 0,
                        bytes_received: 0,
                        bytes_sent: 0,
                        country: None,
                        first_seen: now,
                        last_active: now,
                    });
                }
            }
        }
    }

    // Distribute live throughput & maintain cumulative bytes per connection
    let active_indices: Vec<usize> = connections
        .iter()
        .enumerate()
        .filter(|(_, c)| c.state == "ESTABLISHED")
        .map(|(i, _)| i)
        .collect();

    let num_active = active_indices.len().max(1) as u64;
    let share_dl = dl_speed / num_active;
    let share_ul = ul_speed / num_active;

    if let Ok(mut traffic_map) = SOCKET_TRAFFIC.write() {
        for conn in &mut connections {
            let entry = traffic_map.entry(conn.id.clone()).or_insert_with(SocketTraffic::default);
            if conn.state == "ESTABLISHED" {
                entry.speed_in = share_dl;
                entry.speed_out = share_ul;
                entry.bytes_in += share_dl;
                entry.bytes_out += share_ul;

                // Automatically save to SQLite database
                crate::db::save_connection_record(
                    &conn.process_name,
                    &conn.remote_address,
                    &conn.hostname,
                    conn.remote_port,
                    &conn.protocol,
                    1,
                    share_dl + share_ul,
                );
            } else {
                entry.speed_in = 0;
                entry.speed_out = 0;
            }

            conn.download_speed = entry.speed_in;
            conn.upload_speed = entry.speed_out;
            conn.bytes_received = entry.bytes_in;
            conn.bytes_sent = entry.bytes_out;
        }
    }
    let total_active = connections.iter().filter(|c| c.state == "ESTABLISHED").count();
    let total_listen = connections.iter().filter(|c| c.state == "LISTEN").count();
    let mut unique_procs = std::collections::HashSet::new();
    for c in &connections {
        unique_procs.insert(&c.process_name);
    }

    let summary = NetworkSummary {
        total_active_connections: total_active,
        total_listening_ports: total_listen,
        total_processes: unique_procs.len(),
        total_download_speed: dl_speed,
        total_upload_speed: ul_speed,
        total_bytes_received: total_bytes_in,
        total_bytes_sent: total_bytes_out,
    };

    NetworkSnapshot {
        connections,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_live_connections() {
        let snapshot = get_live_connections();
        println!("Captured {} live connections!", snapshot.connections.len());
        println!("Active: {}, Listen: {}, Processes: {}",
            snapshot.summary.total_active_connections,
            snapshot.summary.total_listening_ports,
            snapshot.summary.total_processes
        );
        for conn in snapshot.connections.iter().take(5) {
            println!("- [{}] PID:{} {} | {} -> {}:{} (Hostname: {})",
                conn.protocol, conn.pid, conn.process_name,
                conn.local_address, conn.remote_address, conn.remote_port, conn.hostname
            );
        }
        assert!(!snapshot.connections.is_empty());
    }
}
