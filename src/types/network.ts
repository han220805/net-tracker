export type Protocol = "TCP" | "UDP";

export type ConnectionState =
  | "ESTABLISHED"
  | "SYN_SENT"
  | "SYN_RCVD"
  | "FIN_WAIT1"
  | "FIN_WAIT2"
  | "TIME_WAIT"
  | "CLOSED"
  | "CLOSE_WAIT"
  | "LAST_ACK"
  | "LISTEN"
  | "CLOSING"
  | "UNKNOWN";

export interface NetworkConnection {
  id: string;
  pid: number;
  process_name: string;
  process_path?: string;
  protocol: Protocol;
  local_address: string;
  local_port: number;
  remote_address: string;
  remote_port: number;
  hostname: string;
  state: ConnectionState;
  download_speed: number; // bytes per second
  upload_speed: number;   // bytes per second
  bytes_received: number;
  bytes_sent: number;
  country?: string;
  first_seen: number;
  last_active: number;
}

export interface NetworkSummary {
  total_active_connections: number;
  total_listening_ports: number;
  total_processes: number;
  total_download_speed: number; // bytes/sec
  total_upload_speed: number;   // bytes/sec
  total_bytes_received: number;
  total_bytes_sent: number;
}

export interface HistoryRecord {
  id: string;
  timestamp: number;
  process_name: string;
  remote_address: string;
  hostname: string;
  remote_port: number;
  protocol: Protocol;
  duration_seconds: number;
  total_bytes: number;
}

export interface BlockedIpRecord {
  ip: string;
  hostname: string;
  process_name: string;
  blocked_at: number;
}

