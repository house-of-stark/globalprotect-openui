#![cfg(target_os = "macos")]

// macOS tunnel backend — uses `osascript` (native macOS admin-prompt dialog)
// to run `openconnect` with root privileges, since `sudo` requires a TTY
// that GUI apps don't have.
//
// Auth (portal prelogin, SAML browser flow) happens in connect.rs via the
// gpapi crate (pure HTTPS, platform-independent). Only the tunnel step
// differs from the Linux path.

use std::sync::Arc;

use gpapi::service::vpn_state::{ConnectInfo, VpnState};
use log::info;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::process::Command;

/// Manages the `openconnect` tunnel process (runs as root via osascript).
pub struct TunnelHandle {
  /// Tokio child for the current osascript invocation, if any.
  running: Arc<Mutex<bool>>,
}

impl TunnelHandle {
  pub fn new() -> Self {
    Self {
      running: Arc::new(Mutex::new(false)),
    }
  }

  /// Launch the tunnel. Prompts the user for admin credentials via a native
  /// macOS dialog (osascript + "with administrator privileges").
  pub async fn connect(
    &self,
    app_handle: AppHandle,
    gateway: &str,
    cookie: &str,
    os: Option<&str>,
    disable_ipv6: bool,
    no_dtls: bool,
    connect_info: ConnectInfo,
  ) -> anyhow::Result<()> {
    let openconnect_bin = "/opt/homebrew/bin/openconnect";
    let os_flag = os.unwrap_or("mac-intel");

    // Build the shell command. Important: the cookie must be quoted safely.
    let mut parts: Vec<String> = vec![
      format!("'{}'", openconnect_bin),
      "--protocol=gp".to_string(),
      format!("--os={}", os_flag),
    ];

    // Pass the cookie via stdin rather than argv (avoids exposure in `ps`).
    parts.push(format!(
      "--cookie-stdin"
    ));

    if disable_ipv6 {
      parts.push("--no-ipv6".to_string());
    }
    if no_dtls {
      parts.push("--no-dtls".to_string());
    }

    parts.push(format!("'{}'", gateway));

    // Pipe cookie + newline into openconnect via stdin.
    // The `--cookie-stdin` flag tells openconnect to read the cookie from stdin.
    let shell_cmd = format!("printf '%s\\n' '{}' | {}", cookie, parts.join(" "));

    info!("Spawning openconnect tunnel to {}", gateway);

    // Kill any existing tunnel before starting a new one
    let _ = self.disconnect().await;

    // Run via osascript for a native privileged-auth prompt.
    let mut child = Command::new("osascript")
      .arg("-e")
      .arg(format!(
        "do shell script \"{}\" with administrator privileges",
        shell_cmd.replace("\"", "\\\"")
      ))
      .stdout(std::process::Stdio::null())
      .stderr(std::process::Stdio::null())
      .spawn()?;

    // Wait for osascript to finish (the admin dialog closes after auth).
    // The openconnect process continues running in the background as root.
    child.wait().await?;

    // Mark as running
    {
      let mut guard = self.running.lock().await;
      *guard = true;
    }

    // Emit connecting state
    let _ = app_handle.emit("vpn-state", VpnState::Connecting(Box::new(connect_info.clone())));

    // Spawn a watcher that polls for process liveness
    let running_flag = Arc::clone(&self.running);
    let app_handle2 = app_handle.clone();
    let _connected_info = connect_info.clone();

    tokio::spawn(async move {
      tokio::time::sleep(std::time::Duration::from_secs(5)).await;
      let _ = app_handle2.emit("vpn-state", VpnState::Connected(Box::new(_connected_info)));

      // Poll for process exit every 10 seconds
      loop {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        let is_running = {
          let flag = running_flag.lock().await;
          *flag
        };

        if !is_running {
          break;
        }

        let alive = Command::new("pgrep")
          .arg("-x")
          .arg("openconnect")
          .output()
          .await
          .map(|o| o.status.success())
          .unwrap_or(false);

        if !alive {
          info!("openconnect process exited");
          let _ = app_handle2.emit("vpn-state", VpnState::Disconnected);
          {
            let mut guard = running_flag.lock().await;
            *guard = false;
          }
          break;
        }
      }
    });

    Ok(())
  }

  /// Disconnect by asking osascript to kill the openconnect process.
  pub async fn disconnect(&self) -> anyhow::Result<()> {
    info!("Disconnecting openconnect tunnel");

    // Check if something is running
    let was_running = {
      let guard = self.running.lock().await;
      *guard
    };

    if !was_running {
      // Try killing anyway (safety net)
      let _ = Command::new("osascript")
        .arg("-e")
        .arg("do shell script \"killall openconnect 2>/dev/null || true\" with administrator privileges")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
      return Ok(());
    }

    {
      let mut guard = self.running.lock().await;
      *guard = false;
    }

    let mut child = Command::new("osascript")
      .arg("-e")
      .arg("do shell script \"killall openconnect 2>/dev/null || true\" with administrator privileges")
      .stdout(std::process::Stdio::null())
      .stderr(std::process::Stdio::null())
      .spawn()?;

    child.wait().await?;
    info!("openconnect terminated");
    Ok(())
  }
}
