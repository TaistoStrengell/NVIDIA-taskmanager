use std::fs;


/// Retrieves the name of a process given its PID.
pub fn get_process_name(pid: u32) -> String {
    let path = format!("/proc/{}/comm", pid);
    fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string())
}

/// Retrieves the command line of a process given its PID.
pub fn get_process_cmdline(pid: u32) -> String {
    let path = format!("/proc/{}/cmdline", pid);
    
    match fs::read(path) {
        Ok(mut bytes) if !bytes.is_empty() => {
            for b in bytes.iter_mut() {
                if *b == 0 { *b = b' '; }
            }
            
            let cmd = String::from_utf8_lossy(&bytes);
            let trimmed = cmd.trim();
            
            if trimmed.is_empty() { "Unknown".to_string() } else { trimmed.to_string() }
        }
        _ => "Unknown".to_string(),
    }
}
/// Kills a process given its PID. If forceful is true, sends SIGKILL; otherwise, sends SIGTERM.
pub fn kill_process(pid: u32, forceful: bool) -> Result<(), String> {
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;

    let sig = if forceful { Signal::SIGKILL } else { Signal::SIGTERM };
    
    signal::kill(Pid::from_raw(pid as i32), sig)
        .map_err(|e| e.to_string())
}