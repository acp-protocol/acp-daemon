//! @acp:module "Daemon Lifecycle"
//! @acp:summary "Process lifecycle management (start/stop/status)"
//! @acp:domain daemon
//! @acp:layer service
//!
//! Manages daemon process lifecycle including daemonization,
//! PID file management, and graceful shutdown.

use std::fs;
use std::path::Path;
use std::process::Command;

use console::style;

const PID_FILE: &str = ".acp/daemon.pid";
const LOG_FILE: &str = ".acp/daemon.log";

/// Start the daemon in background mode
pub fn start_daemon(project_root: impl AsRef<Path>, port: u16) -> anyhow::Result<()> {
    let project_root = project_root.as_ref();
    let pid_path = project_root.join(PID_FILE);

    // Check if already running
    if let Some(pid) = read_pid(&pid_path) {
        if is_process_running(pid) {
            println!(
                "{} Daemon already running with PID {}",
                style("!").yellow(),
                pid
            );
            return Ok(());
        }
        // Stale PID file, remove it
        let _ = fs::remove_file(&pid_path);
    }

    // Ensure .acp directory exists
    let acp_dir = project_root.join(".acp");
    fs::create_dir_all(&acp_dir)?;

    // Get the path to this binary
    let exe_path = std::env::current_exe()?;

    // Spawn daemon in background
    let log_path = project_root.join(LOG_FILE);

    let child = Command::new(&exe_path)
        .arg("run")
        .arg("--port")
        .arg(port.to_string())
        .arg("-C")
        .arg(project_root)
        .stdout(fs::File::create(&log_path)?)
        .stderr(fs::File::create(&log_path)?)
        .spawn()?;

    let pid = child.id();

    // Write PID file
    fs::write(&pid_path, pid.to_string())?;

    println!(
        "{} Daemon started with PID {} (port {})",
        style("✓").green(),
        pid,
        port
    );
    println!("  Log file: {}", log_path.display());
    println!("  API: http://127.0.0.1:{}/health", port);

    Ok(())
}

/// Stop the daemon
pub fn stop_daemon(project_root: impl AsRef<Path>) -> anyhow::Result<()> {
    let pid_path = project_root.as_ref().join(PID_FILE);

    match read_pid(&pid_path) {
        Some(pid) => {
            if is_process_running(pid) {
                // Send SIGTERM
                #[cfg(unix)]
                {
                    use nix::sys::signal::{self, Signal};
                    use nix::unistd::Pid;

                    match signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                        Ok(_) => {
                            println!("{} Sent SIGTERM to daemon (PID {})", style("✓").green(), pid);
                        }
                        Err(e) => {
                            println!("{} Failed to stop daemon: {}", style("✗").red(), e);
                            return Err(e.into());
                        }
                    }
                }

                #[cfg(not(unix))]
                {
                    println!("{} Stopping daemon not supported on this platform", style("✗").red());
                }
            } else {
                println!("{} Daemon not running (stale PID file)", style("!").yellow());
            }

            // Remove PID file
            let _ = fs::remove_file(&pid_path);
            Ok(())
        }
        None => {
            println!("{} No daemon running", style("!").yellow());
            Ok(())
        }
    }
}

/// Check daemon status
pub fn check_status(project_root: impl AsRef<Path>) -> anyhow::Result<()> {
    let pid_path = project_root.as_ref().join(PID_FILE);

    match read_pid(&pid_path) {
        Some(pid) => {
            if is_process_running(pid) {
                println!("{} Daemon is running (PID {})", style("✓").green(), pid);

                // Try to check health endpoint
                // Note: This is a blocking call, could be improved with async
                if let Ok(resp) = reqwest_sync_health(9222) {
                    println!("  Health: {}", resp);
                }
            } else {
                println!("{} Daemon not running (stale PID file)", style("!").yellow());
                let _ = fs::remove_file(&pid_path);
            }
        }
        None => {
            println!("{} Daemon not running", style("•").dim());
        }
    }

    Ok(())
}

fn read_pid(path: &Path) -> Option<u32> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use nix::sys::signal;
        use nix::unistd::Pid;

        // Send signal 0 to check if process exists
        signal::kill(Pid::from_raw(pid as i32), None).is_ok()
    }

    #[cfg(not(unix))]
    {
        // On non-Unix, assume process is running if we have a PID
        true
    }
}

fn reqwest_sync_health(port: u16) -> Result<String, Box<dyn std::error::Error>> {
    // Simple sync HTTP check - in production, use async
    let url = format!("http://127.0.0.1:{}/health", port);

    // Use std::process::Command to curl
    let output = Command::new("curl")
        .arg("-s")
        .arg("-o")
        .arg("/dev/null")
        .arg("-w")
        .arg("%{http_code}")
        .arg(&url)
        .output()?;

    let status = String::from_utf8_lossy(&output.stdout);
    if status == "200" {
        Ok("OK".to_string())
    } else {
        Ok(format!("HTTP {}", status))
    }
}
