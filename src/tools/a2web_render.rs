//! `a2web_render` tool — saves agent-generated HTML and serves it via the gateway.

use super::traits::{Tool, ToolResult};
use crate::config::A2webConfig;
use async_trait::async_trait;
use rand::RngExt;
use serde_json::json;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Tool that persists an HTML page and returns the gateway URL.
pub struct A2webRenderTool {
    /// Root directory for rendered pages (`{zeroclaw_dir}/web/`).
    web_dir: PathBuf,
    /// Gateway base URL (e.g. `http://127.0.0.1:42617`).
    gateway_base_url: String,
    /// Maximum pages kept on disk.
    max_pages: usize,
    /// Time-to-live for pages.
    ttl: Duration,
}

impl A2webRenderTool {
    pub fn new(
        zeroclaw_dir: &std::path::Path,
        gateway_host: &str,
        gateway_port: u16,
        config: &A2webConfig,
    ) -> Self {
        let host = if gateway_host == "0.0.0.0" {
            "127.0.0.1"
        } else {
            gateway_host
        };
        Self {
            web_dir: zeroclaw_dir.join("web"),
            gateway_base_url: format!("http://{host}:{gateway_port}"),
            max_pages: config.max_pages,
            ttl: Duration::from_secs(config.ttl_hours * 3600),
        }
    }

    /// Generate a short 8-character alphanumeric ID.
    fn generate_id() -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::rng();
        (0..8)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Remove expired pages and enforce `max_pages` cap (oldest first).
    fn cleanup(&self) -> anyhow::Result<()> {
        let web_dir = &self.web_dir;
        if !web_dir.exists() {
            return Ok(());
        }

        let now = SystemTime::now();
        let mut entries: Vec<(PathBuf, SystemTime)> = Vec::new();

        for entry in std::fs::read_dir(web_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let modified = entry.metadata().and_then(|m| m.modified()).unwrap_or(now);

            // Remove TTL-expired pages.
            if let Ok(age) = now.duration_since(modified) {
                if age > self.ttl {
                    let _ = std::fs::remove_dir_all(&path);
                    continue;
                }
            }
            entries.push((path, modified));
        }

        // Enforce max_pages — remove oldest first.
        if entries.len() > self.max_pages {
            entries.sort_by_key(|(_, t)| *t);
            let to_remove = entries.len() - self.max_pages;
            for (path, _) in entries.iter().take(to_remove) {
                let _ = std::fs::remove_dir_all(path);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Tool for A2webRenderTool {
    fn name(&self) -> &str {
        "a2web_render"
    }

    fn description(&self) -> &str {
        "Render a full HTML page (with custom CSS/JS) and serve it via the gateway. Use this for rich interactive content that exceeds A2UI card capabilities: dashboards with charts, games, animations, complex forms with validation, data visualizations, maps, or anything requiring custom HTML/CSS/JS. For simpler structured displays (weather, lists, quizzes, schedules), prefer A2UI cards instead. Returns the public URL."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Complete HTML content to render as a page"
                },
                "title": {
                    "type": "string",
                    "description": "Optional page title (injected into <title> if missing)"
                }
            },
            "required": ["content"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let content = match args.get("content").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing required parameter 'content'".into()),
                });
            }
        };
        let title = args.get("title").and_then(|v| v.as_str());

        // Optionally inject <title> if the HTML doesn't already have one.
        let html = if let Some(t) = title {
            if !content.to_lowercase().contains("<title>") {
                let title_tag = format!("<title>{}</title>", html_escape(t));
                if let Some(pos) = content.to_lowercase().find("<head>") {
                    let insert = pos + "<head>".len();
                    format!("{}{}{}", &content[..insert], title_tag, &content[insert..])
                } else {
                    // Prepend a minimal head with the title.
                    format!("<!DOCTYPE html><head>{title_tag}</head>{content}")
                }
            } else {
                content.to_string()
            }
        } else {
            content.to_string()
        };

        let id = Self::generate_id();
        let page_dir = self.web_dir.join(&id);
        std::fs::create_dir_all(&page_dir)?;
        std::fs::write(page_dir.join("index.html"), html.as_bytes())?;

        // Cleanup expired / over-limit pages (best-effort).
        if let Err(e) = self.cleanup() {
            tracing::warn!("a2web cleanup failed: {e}");
        }

        let url = format!("{}/web/{id}/", self.gateway_base_url);
        let title_str = title.as_deref().unwrap_or("");
        let a2web_data = json!({ "url": url, "id": id, "title": title_str });

        Ok(ToolResult {
            success: true,
            output: format!(
                "<a2web-result>{}</a2web-result>\nPage created: {}\n\nIMPORTANT: You MUST include the exact <a2web-result> tag above in your response so the client can render the page inline. Copy the entire <a2web-result>...</a2web-result> line into your reply.",
                a2web_data, url
            ),
            error: None,
        })
    }
}

/// Minimal HTML entity escaping for title text.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Extract JSON from `<a2web-result>...</a2web-result>` in tool output.
    fn parse_output(output: &str) -> serde_json::Value {
        let start = output.find("<a2web-result>").expect("missing a2web-result tag") + "<a2web-result>".len();
        let end = output[start..].find("</a2web-result>").expect("missing closing tag");
        serde_json::from_str(&output[start..start + end]).expect("invalid JSON in a2web-result")
    }

    fn make_tool(tmp: &TempDir) -> A2webRenderTool {
        let config = A2webConfig {
            enabled: true,
            max_pages: 3,
            ttl_hours: 24,
        };
        A2webRenderTool::new(tmp.path(), "127.0.0.1", 42617, &config)
    }

    #[tokio::test]
    async fn renders_page_and_returns_url() {
        let tmp = TempDir::new().unwrap();
        let tool = make_tool(&tmp);

        let result = tool
            .execute(json!({ "content": "<h1>hello</h1>" }))
            .await
            .unwrap();

        assert!(result.success);
        let out = parse_output(&result.output);
        let id = out["id"].as_str().unwrap();
        assert_eq!(id.len(), 8);
        assert!(out["url"]
            .as_str()
            .unwrap()
            .contains(&format!("/web/{id}/")));

        // File should exist on disk.
        let index = tmp.path().join("web").join(id).join("index.html");
        assert!(index.exists());
        let saved = std::fs::read_to_string(&index).unwrap();
        assert!(saved.contains("<h1>hello</h1>"));
    }

    #[tokio::test]
    async fn injects_title_when_missing() {
        let tmp = TempDir::new().unwrap();
        let tool = make_tool(&tmp);

        let result = tool
            .execute(json!({
                "content": "<html><head></head><body>hi</body></html>",
                "title": "Test Page"
            }))
            .await
            .unwrap();

        assert!(result.success);
        let out = parse_output(&result.output);
        let id = out["id"].as_str().unwrap();
        let saved =
            std::fs::read_to_string(tmp.path().join("web").join(id).join("index.html")).unwrap();
        assert!(saved.contains("<title>Test Page</title>"));
    }

    #[tokio::test]
    async fn does_not_overwrite_existing_title() {
        let tmp = TempDir::new().unwrap();
        let tool = make_tool(&tmp);

        let html = "<html><head><title>Original</title></head><body></body></html>";
        let result = tool
            .execute(json!({ "content": html, "title": "New" }))
            .await
            .unwrap();

        assert!(result.success);
        let out = parse_output(&result.output);
        let id = out["id"].as_str().unwrap();
        let saved =
            std::fs::read_to_string(tmp.path().join("web").join(id).join("index.html")).unwrap();
        assert!(saved.contains("<title>Original</title>"));
        assert!(!saved.contains("New"));
    }

    #[tokio::test]
    async fn missing_content_returns_error() {
        let tmp = TempDir::new().unwrap();
        let tool = make_tool(&tmp);

        let result = tool.execute(json!({})).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("content"));
    }

    #[tokio::test]
    async fn cleanup_enforces_max_pages() {
        let tmp = TempDir::new().unwrap();
        let tool = make_tool(&tmp); // max_pages = 3

        // Create 5 pages.
        for _ in 0..5 {
            let r = tool
                .execute(json!({ "content": "<p>page</p>" }))
                .await
                .unwrap();
            assert!(r.success);
        }

        // After cleanup, at most 3 should remain.
        let remaining: Vec<_> = std::fs::read_dir(tmp.path().join("web"))
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();
        assert!(
            remaining.len() <= 3,
            "expected <= 3 pages, got {}",
            remaining.len()
        );
    }

    #[test]
    fn tool_metadata() {
        let tmp = TempDir::new().unwrap();
        let tool = make_tool(&tmp);
        assert_eq!(tool.name(), "a2web_render");
        assert!(!tool.description().is_empty());
        let schema = tool.parameters_schema();
        assert_eq!(schema["required"][0], "content");
    }

    #[test]
    fn gateway_host_0000_maps_to_localhost() {
        let tmp = TempDir::new().unwrap();
        let config = A2webConfig::default();
        let tool = A2webRenderTool::new(tmp.path(), "0.0.0.0", 42617, &config);
        assert!(tool.gateway_base_url.contains("127.0.0.1"));
    }
}
