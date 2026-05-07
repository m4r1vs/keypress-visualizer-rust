use std::fs;
use std::path::PathBuf;
use std::process;

fn get_pid_file_path() -> PathBuf {
    let mut path = if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir)
    } else {
        std::env::temp_dir()
    };
    path.push("keypress-visualizer.pid");
    path
}

pub fn handle_pid() {
    let pid_path = get_pid_file_path();

    if pid_path.exists() {
        if let Ok(content) = fs::read_to_string(&pid_path) {
            if let Ok(pid) = content.trim().parse::<i32>() {
                // Check if process is running
                if unsafe { libc::kill(pid, 0) } == 0 {
                    // Safety check: Is it actually our app?
                    if is_same_app(pid) {
                        println!("Existing instance found (PID: {}). Closing it.", pid);
                        unsafe { libc::kill(pid, libc::SIGTERM) };
                        
                        // Give it a moment to exit and clean up
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        
                        // Ensure the file is gone even if it didn't clean up
                        let _ = fs::remove_file(&pid_path);
                        
                        // Exit current instance (the toggle action)
                        process::exit(0);
                    } else {
                        eprintln!("Warning: PID file exists but process {} does not seem to be this application. Overwriting.", pid);
                    }
                }
            }
        }
    }

    // Write current PID
    let current_pid = process::id();
    if let Err(e) = fs::write(&pid_path, current_pid.to_string()) {
        eprintln!("Warning: Could not write PID file {}: {}", pid_path.display(), e);
    }
}

fn is_same_app(pid: i32) -> bool {
    let current_comm = fs::read_to_string("/proc/self/comm").ok();
    let target_comm = fs::read_to_string(format!("/proc/{}/comm", pid)).ok();

    match (current_comm, target_comm) {
        (Some(c), Some(t)) => c.trim() == t.trim(),
        _ => false,
    }
}

pub fn cleanup_pid() {
    let pid_path = get_pid_file_path();
    if pid_path.exists() {
        // Only remove it if it contains our PID
        if let Ok(content) = fs::read_to_string(&pid_path) {
            if let Ok(pid) = content.trim().parse::<u32>() {
                if pid == process::id() {
                    let _ = fs::remove_file(pid_path);
                }
            }
        }
    }
}
