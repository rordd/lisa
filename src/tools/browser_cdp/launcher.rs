//! Browser launcher implementations for CDP direct backend.
//!
//! - `ChromeLauncher` — launches Chrome/Chromium on Linux with persistent profile
//! - `WamLauncher` — connects to webOS TV WAM browser via luna-send

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Result of a browser launch — includes WS endpoint and optional PID.
pub struct LaunchResult {
    pub ws_url: String,
    pub pid: Option<u32>,
}

/// Trait for browser launchers that provide a CDP WebSocket endpoint.
#[async_trait]
pub trait BrowserLauncher: Send + Sync {
    /// Launch the browser and return the CDP WebSocket endpoint URL.
    async fn launch(&mut self) -> Result<LaunchResult>;
    /// Shut down the browser (if managed by us).
    async fn shutdown(&mut self) -> Result<()>;
    /// Check if the browser process is still running.
    fn is_running(&self) -> bool;
}

/// Discover the browser-level WebSocket endpoint by polling /json/version.
/// chromiumoxide `Browser::connect()` requires the browser-level endpoint
/// (not page-level) to function correctly.
pub async fn discover_ws_endpoint(port: u16, timeout_secs: u64) -> Result<String> {
    let version_url = format!("http://127.0.0.1:{port}/json/version");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);

    loop {
        if tokio::time::Instant::now() > deadline {
            anyhow::bail!("Timeout waiting for CDP endpoint on port {port}");
        }

        if let Ok(resp) = reqwest::get(&version_url).await {
            if let Ok(version) = resp.json::<serde_json::Value>().await {
                if let Some(ws_url) = version.get("webSocketDebuggerUrl").and_then(|v| v.as_str()) {
                    return Ok(ws_url.to_string());
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}

// ── Chrome Launcher ──────────────────────────────────────────────

/// Launches Chrome/Chromium for CDP automation on Linux.
pub struct ChromeLauncher {
    port: u16,
    headless: bool,
    chrome_path: Option<String>,
    #[allow(dead_code)] // Intentionally not passed to Chrome to avoid fingerprinting
    window_size: String,
    user_data_dir: String,
    cleanup_stale: bool,
    child: Option<tokio::process::Child>,
}

impl ChromeLauncher {
    pub fn new(
        port: u16,
        headless: bool,
        chrome_path: Option<String>,
        window_size: String,
        user_data_dir: Option<String>,
        cleanup_stale: bool,
    ) -> Self {
        let data_dir = user_data_dir.unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            format!("{home}/.zeroclaw/browser-profile")
        });
        Self {
            port,
            headless,
            chrome_path,
            window_size,
            user_data_dir: data_dir,
            cleanup_stale,
            child: None,
        }
    }

    /// Kill stale Chrome processes on the given CDP port.
    async fn kill_stale_chrome(port: u16) {
        let port_flag = format!("--remote-debugging-port={port}");
        let my_pid = std::process::id() as i32;

        let output = match Command::new("pgrep")
            .args(["-f", &port_flag])
            .output()
            .await
        {
            Ok(o) if o.status.success() => o,
            _ => return,
        };

        let pids = String::from_utf8_lossy(&output.stdout);
        let mut killed = false;
        for pid_str in pids.split_whitespace() {
            if let Ok(pid) = pid_str.parse::<i32>() {
                if pid == my_pid {
                    continue;
                }
                // Verify it's actually Chrome/Chromium before killing
                let exe_path = format!("/proc/{pid}/exe");
                let is_chrome = match std::fs::read_link(&exe_path) {
                    Ok(exe) => {
                        let name = exe.to_string_lossy();
                        name.contains("chrome") || name.contains("chromium")
                    }
                    Err(_) => false, // Can't verify — skip to be safe
                };
                if !is_chrome {
                    continue;
                }
                warn!(pid, port, "Killing stale Chrome process on CDP port");
                unsafe {
                    libc::kill(pid, libc::SIGTERM);
                }
                killed = true;
            }
        }
        if killed {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}

#[async_trait]
impl BrowserLauncher for ChromeLauncher {
    async fn launch(&mut self) -> Result<LaunchResult> {
        let binary = self
            .chrome_path
            .clone()
            .or_else(find_chrome_binary)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No Chrome/Chromium found. Install Chrome or set browser.cdp_direct.chrome_path"
                )
            })?;

        if self.cleanup_stale {
            Self::kill_stale_chrome(self.port).await;
        }

        // Ensure profile directory exists
        if let Err(e) = std::fs::create_dir_all(&self.user_data_dir) {
            warn!(path = %self.user_data_dir, error = %e, "Failed to create browser profile directory");
        }

        info!(
            binary = %binary,
            port = self.port,
            user_data_dir = %self.user_data_dir,
            "Launching Chrome for CDP"
        );

        // Launch Chrome with minimal flags to avoid bot detection.
        // Only essential flags: remote debugging port, user data dir, first-run suppression.
        // Extra flags (--disable-dev-shm-usage, --disable-blink-features, --window-size, etc.)
        // can be fingerprinted by anti-bot systems as automation indicators.
        let mut cmd = Command::new(&binary);
        cmd.arg(format!("--remote-debugging-port={}", self.port))
            .arg(format!("--user-data-dir={}", self.user_data_dir))
            .arg("--no-first-run")
            .arg("--no-default-browser-check");

        if self.headless {
            cmd.arg("--headless=new");
        }

        // Ensure HOME is set
        if std::env::var_os("HOME").is_none() {
            cmd.env("HOME", "/tmp");
        }

        // When not headless, ensure the browser can access the user's display.
        // Detect Wayland/X11 and set the appropriate environment variables
        // so Chrome opens a visible window even when launched from a daemon.
        if !self.headless {
            let uid = unsafe { libc::getuid() };
            let xdg_runtime = format!("/run/user/{uid}");

            if std::env::var_os("XDG_RUNTIME_DIR").is_none() {
                if std::path::Path::new(&xdg_runtime).exists() {
                    cmd.env("XDG_RUNTIME_DIR", &xdg_runtime);
                }
            }

            // Try Wayland first, then X11
            let has_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
            let has_x11 = std::env::var_os("DISPLAY").is_some();

            if !has_wayland && !has_x11 {
                let wayland_sock = format!("{xdg_runtime}/wayland-0");
                if std::path::Path::new(&wayland_sock).exists() {
                    cmd.env("WAYLAND_DISPLAY", "wayland-0");
                    cmd.arg("--ozone-platform=wayland");
                    info!("Setting WAYLAND_DISPLAY=wayland-0 + ozone-platform=wayland for visible Chrome");
                } else if std::path::Path::new("/tmp/.X11-unix/X0").exists() {
                    cmd.env("DISPLAY", ":0");
                    info!("Setting DISPLAY=:0 for visible Chrome");
                } else {
                    warn!("No display server found — Chrome may not be visible");
                }
            } else if has_wayland && !has_x11 {
                // Wayland env is set but Chrome defaults to X11 — force ozone
                cmd.arg("--ozone-platform=wayland");
            }
        }

        cmd.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        let child = cmd
            .spawn()
            .with_context(|| format!("Failed to launch Chrome from: {binary}"))?;

        let pid = child.id();
        self.child = Some(child);

        let ws_url = discover_ws_endpoint(self.port, 15).await?;

        Ok(LaunchResult { ws_url, pid })
    }

    async fn shutdown(&mut self) -> Result<()> {
        if let Some(ref mut child) = self.child {
            debug!("Shutting down Chrome process");
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        self.child = None;
        Ok(())
    }

    fn is_running(&self) -> bool {
        match self.child {
            Some(ref child) => {
                // Check if the process is still alive via /proc
                if let Some(pid) = child.id() {
                    std::path::Path::new(&format!("/proc/{pid}")).exists()
                } else {
                    false
                }
            }
            None => false,
        }
    }
}

impl Drop for ChromeLauncher {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.child {
            let _ = child.start_kill();
        }
    }
}

/// Search for a Chrome/Chromium binary on the system.
fn find_chrome_binary() -> Option<String> {
    let candidates = [
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/snap/bin/chromium",
        "/usr/local/bin/google-chrome",
        "/usr/local/bin/chromium",
    ];

    for candidate in &candidates {
        if Path::new(candidate).exists() {
            return Some(candidate.to_string());
        }
    }
    None
}

// ── WAM Launcher ─────────────────────────────────────────────────

/// Connects to webOS TV WAM (Web App Manager) browser.
pub struct WamLauncher {
    inspector_port: u16,
    app_id: String,
    launch_timeout_secs: u64,
}

impl WamLauncher {
    pub fn new(inspector_port: u16, app_id: String, launch_timeout_secs: u64) -> Self {
        Self {
            inspector_port,
            app_id,
            launch_timeout_secs,
        }
    }

    /// Activate the WAM app via luna-send.
    async fn activate_app(&self) -> Result<()> {
        let params = serde_json::json!({
            "id": self.app_id,
        });
        let output = Command::new("luna-send")
            .arg("-n")
            .arg("1")
            .arg("-f")
            .arg("luna://com.webos.service.applicationManager/launch")
            .arg(serde_json::to_string(&params)?)
            .output()
            .await
            .context("Failed to run luna-send to launch WAM app")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(app_id = %self.app_id, stderr = %stderr, "luna-send launch returned non-zero");
        }
        Ok(())
    }
}

#[async_trait]
impl BrowserLauncher for WamLauncher {
    async fn launch(&mut self) -> Result<LaunchResult> {
        info!(
            app_id = %self.app_id,
            port = self.inspector_port,
            "Connecting to webOS WAM browser"
        );

        self.activate_app().await?;
        let ws_url = discover_ws_endpoint(self.inspector_port, self.launch_timeout_secs).await?;

        Ok(LaunchResult {
            ws_url,
            pid: None, // WAM is system-managed
        })
    }

    async fn shutdown(&mut self) -> Result<()> {
        debug!("WAM shutdown is no-op (system-managed)");
        Ok(())
    }

    fn is_running(&self) -> bool {
        // WAM is always running as a system service
        true
    }
}

/// Check if the current system is webOS (luna-send exists).
pub fn is_webos() -> bool {
    Path::new("/usr/bin/luna-send").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_webos_returns_false_on_standard_linux() {
        let _ = is_webos();
    }

    #[test]
    fn chrome_launcher_new_defaults() {
        let launcher = ChromeLauncher::new(9222, true, None, "1280x720".into(), None, false);
        assert_eq!(launcher.port, 9222);
        assert!(launcher.headless);
        assert!(!launcher.cleanup_stale);
        assert!(launcher
            .user_data_dir
            .ends_with(".zeroclaw/browser-profile"));
    }

    #[test]
    fn chrome_launcher_custom_data_dir() {
        let launcher = ChromeLauncher::new(
            9222,
            false,
            None,
            "1280x720".into(),
            Some("/tmp/my-profile".into()),
            true,
        );
        assert_eq!(launcher.user_data_dir, "/tmp/my-profile");
        assert!(launcher.cleanup_stale);
    }

    #[test]
    fn wam_launcher_new() {
        let launcher = WamLauncher::new(9998, "com.webos.app.browser".into(), 10);
        assert_eq!(launcher.inspector_port, 9998);
        assert_eq!(launcher.app_id, "com.webos.app.browser");
    }
}
