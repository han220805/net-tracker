use std::os::windows::process::CommandExt;
use std::process::Command;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Kill a process by PID using Windows API TerminateProcess,
/// with fallback to `taskkill /F /PID <pid>`.
pub fn kill_process_by_pid(pid: u32) -> Result<(), String> {
    if pid == 0 || pid == 4 {
        return Err("Cannot terminate System Idle Process or System (PID 0/4)".to_string());
    }

    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if !handle.is_null() {
            let success = TerminateProcess(handle, 1);
            CloseHandle(handle);
            if success != 0 {
                return Ok(());
            }
        }
    }

    // Fallback to taskkill
    let output = Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Failed to execute taskkill: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        let out_msg = String::from_utf8_lossy(&output.stdout);
        Err(format!("Process termination failed: {} {}", err_msg.trim(), out_msg.trim()))
    }
}

/// Helper to sanitize IP string to avoid command injection
fn is_valid_ip(ip: &str) -> bool {
    ip.parse::<std::net::IpAddr>().is_ok()
}

/// Block remote IP using Windows Defender Firewall rule via `netsh`.
/// If standard execution fails due to permissions, attempts elevation via PowerShell `Start-Process ... -Verb RunAs`.
pub fn block_remote_ip(ip: &str) -> Result<(), String> {
    let trimmed_ip = ip.trim();
    if !is_valid_ip(trimmed_ip) {
        return Err(format!("Invalid IP address format: {trimmed_ip}"));
    }

    let rule_out = format!("NetTracker_Block_OUT_{}", trimmed_ip);
    let rule_in = format!("NetTracker_Block_IN_{}", trimmed_ip);

    // Try direct netsh command first
    let res_out = Command::new("netsh")
        .args([
            "advfirewall", "firewall", "add", "rule",
            &format!("name={}", rule_out),
            "dir=out", "action=block",
            &format!("remoteip={}", trimmed_ip),
            "enable=yes",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let res_in = Command::new("netsh")
        .args([
            "advfirewall", "firewall", "add", "rule",
            &format!("name={}", rule_in),
            "dir=in", "action=block",
            &format!("remoteip={}", trimmed_ip),
            "enable=yes",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let direct_success = match (&res_out, &res_in) {
        (Ok(o1), Ok(o2)) => o1.status.success() && o2.status.success(),
        _ => false,
    };

    if direct_success {
        return Ok(());
    }

    // If direct netsh failed (likely requires Administrator elevation), use PowerShell RunAs
    let ps_script = format!(
        "Start-Process netsh -ArgumentList 'advfirewall firewall add rule name=\"{}\" dir=out action=block remoteip=\"{}\" enable=yes' -Verb RunAs -WindowStyle Hidden -Wait; \
         Start-Process netsh -ArgumentList 'advfirewall firewall add rule name=\"{}\" dir=in action=block remoteip=\"{}\" enable=yes' -Verb RunAs -WindowStyle Hidden -Wait",
        rule_out, trimmed_ip, rule_in, trimmed_ip
    );

    let elevated_res = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Elevation failed: {e}"))?;

    if elevated_res.status.success() {
        Ok(())
    } else {
        Err("Failed to add firewall rule. Administrator privileges are required.".to_string())
    }
}

/// Unblock / Release remote IP by deleting the Windows Defender Firewall rule.
pub fn unblock_remote_ip(ip: &str) -> Result<(), String> {
    let trimmed_ip = ip.trim();
    if !is_valid_ip(trimmed_ip) {
        return Err(format!("Invalid IP address format: {trimmed_ip}"));
    }

    let rule_out = format!("NetTracker_Block_OUT_{}", trimmed_ip);
    let rule_in = format!("NetTracker_Block_IN_{}", trimmed_ip);

    // Try direct netsh command first
    let res_out = Command::new("netsh")
        .args(["advfirewall", "firewall", "delete", "rule", &format!("name={}", rule_out)])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let res_in = Command::new("netsh")
        .args(["advfirewall", "firewall", "delete", "rule", &format!("name={}", rule_in)])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let direct_success = match (&res_out, &res_in) {
        (Ok(o1), Ok(o2)) => o1.status.success() || o2.status.success(),
        _ => false,
    };

    if direct_success {
        return Ok(());
    }

    // Fallback with elevation
    let ps_script = format!(
        "Start-Process netsh -ArgumentList 'advfirewall firewall delete rule name=\"{}\"' -Verb RunAs -WindowStyle Hidden -Wait; \
         Start-Process netsh -ArgumentList 'advfirewall firewall delete rule name=\"{}\"' -Verb RunAs -WindowStyle Hidden -Wait",
        rule_out, rule_in
    );

    let elevated_res = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Elevation failed: {e}"))?;

    if elevated_res.status.success() {
        Ok(())
    } else {
        Err("Failed to remove firewall rule. Administrator privileges are required.".to_string())
    }
}
