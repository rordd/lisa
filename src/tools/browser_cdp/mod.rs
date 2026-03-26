//! CDP-direct browser backend using chromiumoxide.
//!
//! Connects to Chrome/Chromium or webOS WAM browsers via Chrome DevTools
//! Protocol using `Browser::connect()`. Browser launching is handled by
//! the `BrowserLauncher` trait (ChromeLauncher for Linux, WamLauncher for webOS).

pub mod launcher;
pub mod snapshot;

use crate::config::BrowserCdpDirectConfig;
use crate::tools::browser::BrowserAction;
use crate::tools::traits::ToolResult;
use anyhow::Result;
use chromiumoxide::browser::Browser;
use chromiumoxide::Page;
use futures_util::StreamExt;
use launcher::{is_webos, BrowserLauncher, ChromeLauncher, WamLauncher};
use serde_json::json;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// Wait for the DOM to stabilize (SPA dynamic rendering).
/// Safely create a JavaScript string literal (with quotes) from a Rust string.
/// Returns a JSON-escaped double-quoted string, safe for direct use in JS expressions.
/// Example: `js_str("hello \"world\"")` → `"hello \"world\""`
fn js_str(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}

/// CDP backend state — manages browser lifecycle and action dispatch.
pub struct CdpBackendState {
    browser: Option<Browser>,
    handler_task: Option<JoinHandle<()>>,
    launcher: Option<Box<dyn BrowserLauncher>>,
    page: Option<Page>,
    config: BrowserCdpDirectConfig,
}

impl CdpBackendState {
    pub fn new(config: BrowserCdpDirectConfig) -> Self {
        Self {
            browser: None,
            handler_task: None,
            launcher: None,
            page: None,
            config,
        }
    }

    /// Ensure a CDP connection is active, launching a browser if needed.
    async fn ensure_connection(&mut self) -> Result<()> {
        // Check if handler task has died (WebSocket closed, crash, etc.)
        if let Some(ref task) = self.handler_task {
            if task.is_finished() {
                warn!("CDP handler task finished unexpectedly — reconnecting");
                self.browser = None;
                self.handler_task = None;
                self.page = None;
            }
        }

        if self.browser.is_some() {
            return Ok(());
        }

        // If launcher exists and is running, reconnect to existing browser
        if let Some(ref launcher) = self.launcher {
            if launcher.is_running() {
                info!("Chrome still running — reconnecting CDP");
                return self.reconnect().await;
            }
        }

        // Fresh launch
        self.launch_and_connect().await
    }

    /// Launch a browser and connect via CDP.
    async fn launch_and_connect(&mut self) -> Result<()> {
        // Create launcher based on platform
        let mut launcher: Box<dyn BrowserLauncher> = if is_webos() {
            Box::new(WamLauncher::new(
                self.config.wam_inspector_port,
                self.config.wam_app_id.clone(),
                self.config.wam_launch_timeout_secs,
            ))
        } else {
            Box::new(ChromeLauncher::new(
                self.config.debug_port,
                self.config.headless,
                self.config.chrome_path.clone(),
                self.config.window_size.clone(),
                self.config.user_data_dir.clone(),
                self.config.cleanup_stale,
            ))
        };

        let result = launcher.launch().await?;
        info!(ws_url = %result.ws_url, pid = ?result.pid, "Browser launched");

        self.connect_to_endpoint(&result.ws_url).await?;
        self.launcher = Some(launcher);
        Ok(())
    }

    /// Reconnect to an existing browser by re-discovering the WS endpoint.
    async fn reconnect(&mut self) -> Result<()> {
        let port = if is_webos() {
            self.config.wam_inspector_port
        } else {
            self.config.debug_port
        };

        let ws_url = launcher::discover_ws_endpoint(port, 10).await?;
        info!(ws_url = %ws_url, "Reconnected to existing browser");
        self.connect_to_endpoint(&ws_url).await
    }

    /// Connect chromiumoxide to a CDP WebSocket endpoint.
    async fn connect_to_endpoint(&mut self, ws_url: &str) -> Result<()> {
        let (browser, mut handler) = Browser::connect(ws_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect chromiumoxide to {ws_url}: {e}"))?;

        // Spawn handler as background task for CDP event processing
        let handler_task = tokio::spawn(async move {
            loop {
                if handler.next().await.is_none() {
                    break;
                }
            }
        });

        // Select the active page once at connect time (before any site loads).
        // This avoids calling browser.pages() during browsing, which triggers
        // CDP Target.getTargets and can be detected as automation.
        let mut page_opt: Option<Page> = None;

        // Try to use Chrome's existing page
        if let Ok(pages) = browser.pages().await {
            page_opt = pages.into_iter().next();
        }

        // If chromiumoxide hasn't discovered pages yet, wait and retry once
        if page_opt.is_none() {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            if let Ok(pages) = browser.pages().await {
                page_opt = pages.into_iter().next();
            }
        }

        // Last resort: create a new page
        if page_opt.is_none() {
            page_opt = Some(
                browser
                    .new_page("about:blank")
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create page: {e}"))?,
            );
        }

        // Navigate to about:blank if the page is on a chrome:// URL.
        // Navigating from chrome://newtab to external sites sends different
        // headers that trigger bot detection on sites like Coupang.
        if let Some(ref page) = page_opt {
            let current_url: String = page
                .evaluate("window.location.href")
                .await
                .map(|v| v.into_value().unwrap_or_default())
                .unwrap_or_default();
            if current_url.starts_with("chrome://") || current_url.is_empty() {
                let _ = page.evaluate("window.location.href = 'about:blank'").await;
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            }

            // Override navigator.webdriver
            let _ = page
                .evaluate(
                    "Object.defineProperty(navigator, 'webdriver', {get: () => undefined})",
                )
                .await;
            // Override window.open to navigate in current tab instead of opening new tab.
            // This enforces single-tab workflow and prevents sites from opening popups.
            let _ = page
                .evaluate(
                    "window.open = function(url) { if (url) window.location.href = url; return window; }",
                )
                .await;
        }

        self.browser = Some(browser);
        self.handler_task = Some(handler_task);
        self.page = page_opt;
        Ok(())
    }

    /// Get the active page — returns the page selected at connect time.
    /// Does NOT call browser.pages() to avoid CDP Target.getTargets calls
    /// that can be detected as automation by anti-bot systems.
    fn active_page(&self) -> Result<Page> {
        self.page
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No active page"))
    }

    /// Execute a browser action.
    pub async fn execute_action(&mut self, action: BrowserAction) -> Result<ToolResult> {
        self.ensure_connection().await?;

        let result = self.execute_action_inner(action).await;
        match result {
            Ok(tr) => Ok(tr),
            Err(e) => {
                // On error, check if connection is still alive
                if let Some(ref task) = self.handler_task {
                    if task.is_finished() {
                        self.browser = None;
                        self.handler_task = None;
                    }
                }
                Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("CDP error: {e:#}")),
                })
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn execute_action_inner(&mut self, action: BrowserAction) -> Result<ToolResult> {
        let timeout_ms = self.config.timeout_ms;
        let page = self.active_page()?;

        match action {
            BrowserAction::Open { url } => {
                // Navigate via JS instead of CDP Page.navigate to avoid
                // bot detection. CDP-initiated navigation is detectable by
                // sites like Coupang/Naver, but JS navigation looks like
                // a user clicking a link.
                page.evaluate(format!("window.location.href = {}", js_str(&url)))
                    .await
                    .map_err(|e| anyhow::anyhow!("Navigation failed: {e}"))?;

                // Re-apply overrides after navigation (page context resets)
                let _ = page
                    .evaluate(
                        "Object.defineProperty(navigator, 'webdriver', {get: () => undefined})",
                    )
                    .await;
                let _ = page
                    .evaluate(
                        "window.open = function(url) { if (url) window.location.href = url; return window; }",
                    )
                    .await;

                // Wait for page load (poll readyState since evaluate doesn't await promises)
                for _ in 0..30 {
                    let state: String = page
                        .evaluate("document.readyState")
                        .await
                        .map(|v| v.into_value().unwrap_or_default())
                        .unwrap_or_default();
                    if state == "complete" || state == "interactive" {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }

                // Wait for DOM to stabilize (SPA dynamic rendering)
                // Use tokio::time::sleep-based polling instead of MutationObserver promise,
                // since chromiumoxide evaluate() doesn't await JS promises.
                {
                    let mut prev_count: i64 = 0;
                    let mut stable = 0u32;
                    // Minimum 1.5s initial wait for API calls to complete
                    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                    for _ in 0..20 {
                        let cur: i64 = page
                            .evaluate("document.querySelectorAll('*').length")
                            .await
                            .map(|v| v.into_value().unwrap_or(0i64))
                            .unwrap_or(0);
                        if cur == prev_count && cur > 50 {
                            stable += 1;
                            if stable >= 3 {
                                break;
                            }
                        } else {
                            stable = 0;
                        }
                        prev_count = cur;
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                    }
                }

                let current_url = page
                    .url()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get URL: {e}"))?
                    .map(|u| u.to_string())
                    .unwrap_or_default();

                // Auto-snapshot after open
                let snapshot = snapshot::take_snapshot(&page, true, true, None).await?;

                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "open",
                        "url": current_url,
                        "snapshot": snapshot,
                    }))?,
                    error: None,
                })
            }

            BrowserAction::Snapshot {
                interactive_only,
                compact,
                depth,
            } => {
                // Wait for page to be fully loaded and dynamic content to stabilize.
                // SPA sites (React/Vue) render content via JS after readyState='complete',
                // so we poll until the DOM stops changing.
                for _ in 0..30 {
                    let state: String = page
                        .evaluate("document.readyState")
                        .await
                        .map(|v| v.into_value().unwrap_or_default())
                        .unwrap_or_default();
                    if state == "complete" || state == "interactive" {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }

                // Wait for DOM to stabilize: compare element count until it stops changing
                // Use tokio::time::sleep-based polling instead of MutationObserver promise,
                // since chromiumoxide evaluate() doesn't await JS promises.
                {
                    let mut prev_count: i64 = 0;
                    let mut stable = 0u32;
                    // Minimum 1.5s initial wait for API calls to complete
                    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                    for _ in 0..20 {
                        let cur: i64 = page
                            .evaluate("document.querySelectorAll('*').length")
                            .await
                            .map(|v| v.into_value().unwrap_or(0i64))
                            .unwrap_or(0);
                        if cur == prev_count && cur > 50 {
                            stable += 1;
                            if stable >= 3 {
                                break;
                            }
                        } else {
                            stable = 0;
                        }
                        prev_count = cur;
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                    }
                }

                let snapshot = snapshot::take_snapshot(
                    &page,
                    interactive_only,
                    compact,
                    depth.map(|d| d as i64),
                )
                .await?;

                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "snapshot",
                        "data": snapshot,
                    }))?,
                    error: None,
                })
            }

            BrowserAction::Click { selector } => {
                let resolved = snapshot::resolve_selector(&selector);

                // Before clicking, force links to open in the current tab.
                // Remove target="_blank" and set target="_self" on the element
                // AND its parent <a> tag (in case the ref points to an inner element).
                // This keeps a single-tab workflow and avoids complex multi-tab handling.
                let _ = page
                    .evaluate(format!(
                        "(function() {{ \
                            var el = document.querySelector({sel}); \
                            if (!el) return; \
                            var targets = [el]; \
                            var parent = el.closest('a'); \
                            if (parent) targets.push(parent); \
                            targets.forEach(function(t) {{ \
                                t.removeAttribute('target'); \
                                t.setAttribute('target', '_self'); \
                            }}); \
                        }})()",
                        sel = js_str(&resolved)
                    ))
                    .await;

                // Scroll element into view before clicking (handles below-the-fold buttons)
                let _ = page
                    .evaluate(format!(
                        "(function() {{ var el = document.querySelector({}); if (el) el.scrollIntoView({{behavior:'smooth',block:'center'}}); }})()",
                        js_str(&resolved)
                    ))
                    .await;
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;

                // Record URL before click to detect navigation
                let url_before: String = page
                    .evaluate("window.location.href")
                    .await
                    .map(|v| v.into_value().unwrap_or_default())
                    .unwrap_or_default();

                page.find_element(resolved.as_str())
                    .await
                    .map_err(|e| anyhow::anyhow!("Element not found: {e}"))?
                    .click()
                    .await
                    .map_err(|e| anyhow::anyhow!("Click failed: {e}"))?;

                // Wait briefly then check if navigation occurred
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

                let url_after: String = page
                    .evaluate("window.location.href")
                    .await
                    .map(|v| v.into_value().unwrap_or_default())
                    .unwrap_or_default();

                if url_after != url_before {
                    // Navigation happened — wait for page load + DOM stabilize
                    debug!(from = %url_before, to = %url_after, "Click triggered navigation");

                    // Re-apply overrides on the new page (context resets after navigation)
                    let _ = page
                        .evaluate(
                            "Object.defineProperty(navigator, 'webdriver', {get: () => undefined})",
                        )
                        .await;
                    let _ = page
                        .evaluate(
                            "window.open = function(url) { if (url) window.location.href = url; return window; }",
                        )
                        .await;

                    for _ in 0..30 {
                        let state: String = page
                            .evaluate("document.readyState")
                            .await
                            .map(|v| v.into_value().unwrap_or_default())
                            .unwrap_or_default();
                        if state == "complete" {
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    }
                }

                // DOM stabilize (handles both navigation and in-page changes)
                {
                    let mut prev_count: i64 = 0;
                    let mut stable = 0u32;
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    for _ in 0..15 {
                        let cur: i64 = page
                            .evaluate("document.querySelectorAll('*').length")
                            .await
                            .map(|v| v.into_value().unwrap_or(0i64))
                            .unwrap_or(0);
                        if cur == prev_count && cur > 50 {
                            stable += 1;
                            if stable >= 3 {
                                break;
                            }
                        } else {
                            stable = 0;
                        }
                        prev_count = cur;
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                    }
                }

                let snapshot = snapshot::take_snapshot(&page, true, true, None).await?;
                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "click",
                        "selector": selector,
                        "snapshot": snapshot,
                    }))?,
                    error: None,
                })
            }

            BrowserAction::Fill { selector, value } => {
                let resolved = snapshot::resolve_selector(&selector);
                let el = page
                    .find_element(resolved.as_str())
                    .await
                    .map_err(|e| anyhow::anyhow!("Element not found: {e}"))?;
                el.click()
                    .await
                    .map_err(|e| anyhow::anyhow!("Focus failed: {e}"))?;
                // Clear existing value
                let _ = page
                    .evaluate(format!(
                        "(function() {{ var el = document.querySelector({}); if (el) el.value = ''; }})()",
                        js_str(&resolved)
                    ))
                    .await;
                el.type_str(&value)
                    .await
                    .map_err(|e| anyhow::anyhow!("Type failed: {e}"))?;

                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "fill",
                        "selector": selector,
                    }))?,
                    error: None,
                })
            }

            BrowserAction::Type { selector, text } => {
                let resolved = snapshot::resolve_selector(&selector);
                // Try to find element; if not found (e.g., refs changed after click),
                // fall back to typing into the currently focused element
                if let Ok(el) = page.find_element(resolved.as_str()).await {
                    el.click()
                        .await
                        .map_err(|e| anyhow::anyhow!("Focus failed: {e}"))?;
                    el.type_str(&text)
                        .await
                        .map_err(|e| anyhow::anyhow!("Type failed: {e}"))?;
                } else {
                    // Fallback: type into whatever element currently has focus
                    debug!(selector = %selector, "Element not found, typing into focused element");
                    let js = format!(
                        "(function() {{ var el = document.activeElement; if (el) {{ el.value = (el.value || '') + {}; el.dispatchEvent(new Event('input', {{bubbles:true}})); el.dispatchEvent(new Event('change', {{bubbles:true}})); }} }})()",
                        js_str(&text)
                    );
                    page.evaluate(js)
                        .await
                        .map_err(|e| anyhow::anyhow!("Type fallback failed: {e}"))?;
                }

                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "type",
                        "selector": selector,
                    }))?,
                    error: None,
                })
            }

            BrowserAction::GetText { selector } => {
                let resolved = snapshot::resolve_selector(&selector);
                let el = page
                    .find_element(resolved.as_str())
                    .await
                    .map_err(|e| anyhow::anyhow!("Element not found: {e}"))?;
                let text = el
                    .inner_text()
                    .await
                    .map_err(|e| anyhow::anyhow!("GetText failed: {e}"))?
                    .unwrap_or_default();

                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "get_text",
                        "text": text,
                    }))?,
                    error: None,
                })
            }

            BrowserAction::GetTitle => {
                let title: String = page
                    .evaluate("document.title")
                    .await
                    .map_err(|e| anyhow::anyhow!("GetTitle failed: {e}"))?
                    .into_value()
                    .unwrap_or_default();

                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "get_title",
                        "title": title,
                    }))?,
                    error: None,
                })
            }

            BrowserAction::GetUrl => {
                let url = page
                    .url()
                    .await
                    .map_err(|e| anyhow::anyhow!("GetUrl failed: {e}"))?
                    .map(|u| u.to_string())
                    .unwrap_or_default();

                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "get_url",
                        "url": url,
                    }))?,
                    error: None,
                })
            }

            BrowserAction::Screenshot { path, full_page } => {
                use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;

                let screenshot_data = if full_page {
                    page.screenshot(
                        chromiumoxide::page::ScreenshotParams::builder()
                            .format(CaptureScreenshotFormat::Png)
                            .full_page(true)
                            .build(),
                    )
                    .await
                    .map_err(|e| anyhow::anyhow!("Screenshot failed: {e}"))?
                } else {
                    page.screenshot(
                        chromiumoxide::page::ScreenshotParams::builder()
                            .format(CaptureScreenshotFormat::Png)
                            .build(),
                    )
                    .await
                    .map_err(|e| anyhow::anyhow!("Screenshot failed: {e}"))?
                };

                if let Some(ref file_path) = path {
                    // Validate screenshot path: canonicalize to resolve .. and
                    // symlinks, then check it's under $HOME or /tmp.
                    let raw = std::path::Path::new(file_path);
                    let abs = if raw.is_absolute() {
                        raw.to_path_buf()
                    } else {
                        std::env::current_dir()?.join(raw)
                    };

                    // Reject paths with .. components before any I/O
                    for comp in abs.components() {
                        if comp == std::path::Component::ParentDir {
                            anyhow::bail!(
                                "Screenshot path rejected: contains '..' traversal: {file_path}"
                            );
                        }
                    }

                    // Ensure parent exists, then canonicalize
                    if let Some(parent) = abs.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let real_path = if abs.exists() {
                        std::fs::canonicalize(&abs)?
                    } else {
                        let parent = abs.parent().unwrap_or(std::path::Path::new("/"));
                        let parent_real = std::fs::canonicalize(parent)?;
                        let filename = abs
                            .file_name()
                            .ok_or_else(|| anyhow::anyhow!("Invalid screenshot path"))?;
                        parent_real.join(filename)
                    };

                    let real_str = real_path.to_string_lossy();
                    let home = std::env::var("HOME").unwrap_or_default();
                    let in_tmp = real_str.starts_with("/tmp/") || real_str == "/tmp";
                    let in_home = !home.is_empty() && real_str.starts_with(&format!("{home}/"));
                    if !in_tmp && !in_home {
                        anyhow::bail!(
                            "Screenshot path rejected: must be under $HOME or /tmp, got: {file_path}"
                        );
                    }
                    // Write to the validated canonical path
                    tokio::fs::write(&real_path, &screenshot_data).await?;
                    debug!(path = %real_str, "Screenshot saved");
                }

                let base64 = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    &screenshot_data,
                );

                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "screenshot",
                        "path": path,
                        "base64_length": base64.len(),
                    }))?,
                    error: None,
                })
            }

            BrowserAction::Wait { selector, ms, text } => {
                if let Some(millis) = ms {
                    let capped = millis.min(30_000);
                    tokio::time::sleep(std::time::Duration::from_millis(capped)).await;
                } else if let Some(ref sel) = selector {
                    let resolved = snapshot::resolve_selector(sel);
                    let deadline =
                        tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
                    loop {
                        if page.find_element(resolved.as_str()).await.is_ok() {
                            break;
                        }
                        if tokio::time::Instant::now() > deadline {
                            anyhow::bail!("Timeout waiting for element: {sel}");
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    }
                } else if let Some(ref txt) = text {
                    let deadline =
                        tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
                    loop {
                        let found: bool = page
                            .evaluate(format!(
                                "document.body && document.body.innerText.includes({})",
                                js_str(txt)
                            ))
                            .await
                            .map(|v| v.into_value().unwrap_or(false))
                            .unwrap_or(false);
                        if found {
                            break;
                        }
                        if tokio::time::Instant::now() > deadline {
                            anyhow::bail!("Timeout waiting for text: {txt}");
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    }
                } else {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }

                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "wait",
                    }))?,
                    error: None,
                })
            }

            BrowserAction::Press { key } => {
                use chromiumoxide::cdp::browser_protocol::input::{
                    DispatchKeyEventParams, DispatchKeyEventType,
                };
                let params = DispatchKeyEventParams::builder()
                    .r#type(DispatchKeyEventType::KeyDown)
                    .key(key.clone())
                    .build()
                    .map_err(|e| anyhow::anyhow!("Invalid key params: {e}"))?;
                page.execute(params)
                    .await
                    .map_err(|e| anyhow::anyhow!("Press failed: {e}"))?;
                let params_up = DispatchKeyEventParams::builder()
                    .r#type(DispatchKeyEventType::KeyUp)
                    .key(key.clone())
                    .build()
                    .map_err(|e| anyhow::anyhow!("Invalid key params: {e}"))?;
                page.execute(params_up)
                    .await
                    .map_err(|e| anyhow::anyhow!("Press up failed: {e}"))?;

                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "press",
                        "key": key,
                    }))?,
                    error: None,
                })
            }

            BrowserAction::Hover { selector } => {
                let resolved = snapshot::resolve_selector(&selector);
                let el = page
                    .find_element(resolved.as_str())
                    .await
                    .map_err(|e| anyhow::anyhow!("Element not found: {e}"))?;
                el.scroll_into_view()
                    .await
                    .map_err(|e| anyhow::anyhow!("Scroll failed: {e}"))?;
                // Hover via JS
                let _ = page
                    .evaluate(format!(
                        "(function() {{ var el = document.querySelector({}); if (el) el.dispatchEvent(new MouseEvent('mouseenter', {{bubbles: true}})); }})()",
                        js_str(&resolved)
                    ))
                    .await;

                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "hover",
                        "selector": selector,
                    }))?,
                    error: None,
                })
            }

            BrowserAction::Scroll { direction, pixels } => {
                let px = pixels.unwrap_or(300);
                let (dx, dy) = match direction.as_str() {
                    "up" => (0, -(px as i64)),
                    "down" => (0, px as i64),
                    "left" => (-(px as i64), 0),
                    "right" => (px as i64, 0),
                    _ => (0, px as i64),
                };
                let _ = page.evaluate(format!("window.scrollBy({dx}, {dy})")).await;

                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "scroll",
                        "direction": direction,
                    }))?,
                    error: None,
                })
            }

            BrowserAction::IsVisible { selector } => {
                let resolved = snapshot::resolve_selector(&selector);
                let visible: bool = page
                    .evaluate(format!(
                        "(function() {{ var el = document.querySelector({}); if (!el) return false; var r = el.getBoundingClientRect(); return r.width > 0 && r.height > 0; }})()",
                        js_str(&resolved)
                    ))
                    .await
                    .map(|v| v.into_value().unwrap_or(false))
                    .unwrap_or(false);

                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "is_visible",
                        "selector": selector,
                        "visible": visible,
                    }))?,
                    error: None,
                })
            }

            BrowserAction::Close => {
                self.shutdown().await;
                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "close",
                    }))?,
                    error: None,
                })
            }

            BrowserAction::Find {
                by,
                value,
                action,
                fill_value,
            } => {
                // Build a JS expression that finds and marks the element.
                // Uses JS iteration + getAttribute comparison (no CSS selector injection risk).
                let safe_value = js_str(&value);
                let find_js = match by.as_str() {
                    "text" => format!(
                        "(function() {{ var val = {}; var els = document.querySelectorAll('*'); for (var i = 0; i < els.length; i++) {{ if (els[i].textContent.trim() === val || els[i].textContent.trim().includes(val)) {{ els[i].setAttribute('data-cdp-found', '1'); return true; }} }} return false; }})()",
                        safe_value
                    ),
                    "label" => format!(
                        "(function() {{ var val = {}; var els = document.querySelectorAll('*'); for (var i = 0; i < els.length; i++) {{ var el = els[i]; if (el.getAttribute('aria-label') === val) {{ el.setAttribute('data-cdp-found', '1'); return true; }} }} var labels = document.querySelectorAll('label'); for (var i = 0; i < labels.length; i++) {{ if (labels[i].textContent.trim().includes(val)) {{ var inp = labels[i].querySelector('input,textarea,select') || document.getElementById(labels[i].getAttribute('for')); if (inp) {{ inp.setAttribute('data-cdp-found', '1'); return true; }} }} }} return false; }})()",
                        safe_value
                    ),
                    "role" => format!(
                        "(function() {{ var val = {}; var els = document.querySelectorAll('*'); for (var i = 0; i < els.length; i++) {{ if (els[i].getAttribute('role') === val) {{ els[i].setAttribute('data-cdp-found', '1'); return true; }} }} return false; }})()",
                        safe_value
                    ),
                    "placeholder" => format!(
                        "(function() {{ var val = {}; var els = document.querySelectorAll('input,textarea'); for (var i = 0; i < els.length; i++) {{ if (els[i].getAttribute('placeholder') && els[i].getAttribute('placeholder').includes(val)) {{ els[i].setAttribute('data-cdp-found', '1'); return true; }} }} return false; }})()",
                        safe_value
                    ),
                    "testid" | "test_id" => format!(
                        "(function() {{ var val = {}; var els = document.querySelectorAll('*'); for (var i = 0; i < els.length; i++) {{ var el = els[i]; if (el.getAttribute('data-testid') === val || el.getAttribute('data-test-id') === val) {{ el.setAttribute('data-cdp-found', '1'); return true; }} }} return false; }})()",
                        safe_value
                    ),
                    _ => String::new(), // CSS selector — handled below
                };

                // For CSS selector (default), use find_element directly
                let js_based = matches!(
                    by.as_str(),
                    "text" | "label" | "role" | "placeholder" | "testid" | "test_id"
                );
                let use_css = !js_based;

                let find_action = action.as_str();

                if !use_css {
                    // Clean up stale markers from previous Find calls
                    let _ = page.evaluate("document.querySelectorAll('[data-cdp-found]').forEach(function(el) { el.removeAttribute('data-cdp-found'); })").await;

                    // JS-based find: mark element with data-cdp-found, then interact via CSS
                    let found: bool = page
                        .evaluate(find_js.clone())
                        .await
                        .map(|v| v.into_value().unwrap_or(false))
                        .unwrap_or(false);
                    if !found {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Element not found by {by}: {value}")),
                        });
                    }
                }

                let css_sel = if use_css {
                    value.clone()
                } else {
                    "[data-cdp-found]".into()
                };

                match find_action {
                    "click" => {
                        page.find_element(&css_sel)
                            .await
                            .map_err(|e| anyhow::anyhow!("Find element failed: {e}"))?
                            .click()
                            .await
                            .map_err(|e| anyhow::anyhow!("Click failed: {e}"))?;
                    }
                    "fill" => {
                        let el = page
                            .find_element(&css_sel)
                            .await
                            .map_err(|e| anyhow::anyhow!("Find element failed: {e}"))?;
                        el.click()
                            .await
                            .map_err(|e| anyhow::anyhow!("Focus failed: {e}"))?;
                        if let Some(ref val) = fill_value {
                            el.type_str(val)
                                .await
                                .map_err(|e| anyhow::anyhow!("Type failed: {e}"))?;
                        }
                    }
                    "get_text" => {
                        let el = page
                            .find_element(&css_sel)
                            .await
                            .map_err(|e| anyhow::anyhow!("Find element failed: {e}"))?;
                        let text = el
                            .inner_text()
                            .await
                            .map_err(|e| anyhow::anyhow!("GetText failed: {e}"))?
                            .unwrap_or_default();

                        // Clean up marker
                        let _ = page.evaluate("document.querySelectorAll('[data-cdp-found]').forEach(el => el.removeAttribute('data-cdp-found'))").await;

                        return Ok(ToolResult {
                            success: true,
                            output: serde_json::to_string_pretty(&json!({
                                "backend": "cdp_direct",
                                "action": "find",
                                "text": text,
                            }))?,
                            error: None,
                        });
                    }
                    _ => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Unknown find action: {find_action}")),
                        });
                    }
                }

                // Clean up find marker
                let _ = page.evaluate("document.querySelectorAll('[data-cdp-found]').forEach(el => el.removeAttribute('data-cdp-found'))").await;

                // Auto-snapshot after find action
                let snapshot = snapshot::take_snapshot(&page, true, true, None).await?;
                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "backend": "cdp_direct",
                        "action": "find",
                        "by": by,
                        "value": value,
                        "snapshot": snapshot,
                    }))?,
                    error: None,
                })
            }
        }
    }

    /// Graceful shutdown: close browser, abort handler, kill process.
    pub async fn shutdown(&mut self) {
        if let Some(mut browser) = self.browser.take() {
            let _ = browser.close().await;
        }
        if let Some(task) = self.handler_task.take() {
            task.abort();
        }
        if let Some(ref mut launcher) = self.launcher {
            let _ = launcher.shutdown().await;
        }
    }
}

impl Drop for CdpBackendState {
    fn drop(&mut self) {
        if let Some(task) = self.handler_task.take() {
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BrowserCdpDirectConfig;

    #[test]
    fn js_str_escapes_double_quotes() {
        let result = js_str(r#"hello "world""#);
        assert_eq!(result, r#""hello \"world\"""#);
    }

    #[test]
    fn js_str_escapes_backslashes() {
        let result = js_str(r"path\to\file");
        assert_eq!(result, r#""path\\to\\file""#);
    }

    #[test]
    fn js_str_escapes_single_quotes_safely() {
        // Single quotes should pass through (they're inside double quotes)
        let result = js_str("it's a test");
        assert!(result.starts_with('"'));
        assert!(result.ends_with('"'));
        assert!(result.contains("it's")); // single quote preserved
    }

    #[test]
    fn js_str_escapes_newlines() {
        let result = js_str("line1\nline2");
        assert!(result.contains("\\n"));
    }

    #[test]
    fn js_str_handles_empty() {
        assert_eq!(js_str(""), "\"\"");
    }

    #[test]
    fn js_str_handles_xss_attempt() {
        let result = js_str("'); alert('xss");
        // Must be safely quoted — no unescaped quotes
        assert!(result.starts_with('"'));
        assert!(result.ends_with('"'));
    }

    #[test]
    fn cdp_backend_state_new_defaults() {
        let config = BrowserCdpDirectConfig::default();
        let state = CdpBackendState::new(config);
        assert!(state.browser.is_none());
        assert!(state.handler_task.is_none());
        assert!(state.launcher.is_none());
    }

    #[test]
    fn cdp_config_defaults() {
        let config = BrowserCdpDirectConfig::default();
        assert_eq!(config.debug_port, 9222);
        assert!(!config.headless);
        assert!(config.chrome_path.is_none());
        assert!(config.user_data_dir.is_none());
        assert_eq!(config.window_size, "1280x720");
        assert_eq!(config.wam_inspector_port, 9998);
        assert_eq!(config.wam_app_id, "com.webos.app.browser");
        assert!(!config.cleanup_stale);
        assert_eq!(config.timeout_ms, 30_000);
    }
}
