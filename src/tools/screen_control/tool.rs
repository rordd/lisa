//! LLM용 Tool 구현 — screen_snapshot + screen_input

use super::ScreenController;
use crate::tools::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

// ── screen_snapshot ──────────────────────────────────────────────

pub struct ScreenSnapshotTool {
    controller: Arc<dyn ScreenController>,
    default_width: u32,
}

impl ScreenSnapshotTool {
    pub fn new(controller: Arc<dyn ScreenController>, default_width: u32) -> Self {
        Self {
            controller,
            default_width,
        }
    }
}

#[async_trait]
impl Tool for ScreenSnapshotTool {
    fn name(&self) -> &str {
        "screen_snapshot"
    }

    fn description(&self) -> &str {
        "화면을 캡처하여 현재 상태를 시각적으로 확인한다. \
        JPEG 이미지(base64)와 해상도·스케일 정보를 리턴한다. \
        좌표 기반 조작 전에 반드시 먼저 호출할 것. \
        리턴된 scale_x/scale_y를 이미지 좌표에 곱하면 실제 화면 좌표가 된다."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "width": {
                    "type": "integer",
                    "description": "리사이즈 폭 (기본 1024). 0이면 원본 크기."
                }
            }
        })
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let width = args
            .get("width")
            .and_then(Value::as_u64)
            .map(|v| v as u32)
            .or(Some(self.default_width));

        let width = width.filter(|&w| w > 0);

        match self.controller.capture(width).await {
            Ok(result) => {
                let text = format!(
                    "Screenshot: {}x{} (original: {}x{}, scale_x: {:.4}, scale_y: {:.4}, size: {} bytes)\n\
                    Coordinates in this image × scale = actual screen coordinates.\n\
                    {}",
                    result.resized_width,
                    result.resized_height,
                    result.orig_width,
                    result.orig_height,
                    result.scale_x,
                    result.scale_y,
                    result.file_size_bytes,
                    result.data_uri,
                );
                Ok(ToolResult {
                    success: true,
                    output: text,
                    error: None,
                })
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
            }),
        }
    }
}

// ── screen_input ─────────────────────────────────────────────────

pub struct ScreenInputTool {
    controller: Arc<dyn ScreenController>,
}

impl ScreenInputTool {
    pub fn new(controller: Arc<dyn ScreenController>) -> Self {
        Self { controller }
    }
}

#[async_trait]
impl Tool for ScreenInputTool {
    fn name(&self) -> &str {
        "screen_input"
    }

    fn description(&self) -> &str {
        "화면에 입력을 보낸다 (클릭, 텍스트 입력, 키 입력, 스크롤, 드래그). \
        좌표는 screen_snapshot의 scale_x/scale_y를 곱한 실제 화면 좌표를 사용할 것."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["action"],
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["click", "double_click", "right_click", "move",
                             "type", "key", "scroll", "drag", "wait"],
                    "description": "수행할 액션"
                },
                "x": { "type": "integer", "description": "X 좌표 (click/double_click/right_click/move/drag 시)" },
                "y": { "type": "integer", "description": "Y 좌표 (click/double_click/right_click/move/drag 시)" },
                "text": { "type": "string", "description": "입력 텍스트 (type 시) 또는 키 이름 (key 시)" },
                "direction": {
                    "type": "string",
                    "enum": ["up", "down", "left", "right"],
                    "description": "스크롤 방향 (scroll 시)"
                },
                "amount": { "type": "integer", "description": "스크롤 클릭 수 (기본 3)" },
                "from_x": { "type": "integer", "description": "드래그 시작 X (drag 시)" },
                "from_y": { "type": "integer", "description": "드래그 시작 Y (drag 시)" },
                "to_x": { "type": "integer", "description": "드래그 끝 X (drag 시)" },
                "to_y": { "type": "integer", "description": "드래그 끝 Y (drag 시)" },
                "ms": { "type": "integer", "description": "대기 시간 밀리초 (wait 시)" }
            }
        })
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let action = match args.get("action").and_then(Value::as_str) {
            Some(a) => a,
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("missing 'action' parameter".into()),
                })
            }
        };

        let get_xy = |args: &Value| -> anyhow::Result<(i32, i32)> {
            let x = args
                .get("x")
                .and_then(Value::as_i64)
                .ok_or_else(|| anyhow::anyhow!("missing 'x'"))? as i32;
            let y = args
                .get("y")
                .and_then(Value::as_i64)
                .ok_or_else(|| anyhow::anyhow!("missing 'y'"))? as i32;
            Ok((x, y))
        };

        let result: anyhow::Result<String> = match action {
            "click" => {
                let (x, y) = get_xy(&args)?;
                self.controller.click(x, y).await?;
                Ok(format!("clicked ({x}, {y})"))
            }
            "double_click" => {
                let (x, y) = get_xy(&args)?;
                self.controller.double_click(x, y).await?;
                Ok(format!("double_clicked ({x}, {y})"))
            }
            "right_click" => {
                let (x, y) = get_xy(&args)?;
                self.controller.right_click(x, y).await?;
                Ok(format!("right_clicked ({x}, {y})"))
            }
            "move" => {
                let (x, y) = get_xy(&args)?;
                self.controller.move_cursor(x, y).await?;
                Ok(format!("moved cursor to ({x}, {y})"))
            }
            "type" => {
                let text = args
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("missing 'text'"))?;
                self.controller.type_text(text).await?;
                Ok(format!("typed {} chars", text.len()))
            }
            "key" => {
                let key = args
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("missing 'text' (key name)"))?;
                self.controller.press_key(key).await?;
                Ok(format!("pressed key: {key}"))
            }
            "scroll" => {
                let dir = args
                    .get("direction")
                    .and_then(Value::as_str)
                    .unwrap_or("down");
                let amount = args
                    .get("amount")
                    .and_then(Value::as_u64)
                    .unwrap_or(3) as u32;
                self.controller.scroll(dir, amount).await?;
                Ok(format!("scrolled {dir} {amount}"))
            }
            "drag" => {
                let from_x = args.get("from_x").and_then(Value::as_i64).ok_or_else(|| anyhow::anyhow!("missing 'from_x'"))? as i32;
                let from_y = args.get("from_y").and_then(Value::as_i64).ok_or_else(|| anyhow::anyhow!("missing 'from_y'"))? as i32;
                let to_x = args.get("to_x").and_then(Value::as_i64).ok_or_else(|| anyhow::anyhow!("missing 'to_x'"))? as i32;
                let to_y = args.get("to_y").and_then(Value::as_i64).ok_or_else(|| anyhow::anyhow!("missing 'to_y'"))? as i32;
                self.controller.drag((from_x, from_y), (to_x, to_y)).await?;
                Ok(format!("dragged ({from_x},{from_y}) → ({to_x},{to_y})"))
            }
            "wait" => {
                const MAX_WAIT_MS: u64 = 10_000;
                let ms = args.get("ms").and_then(Value::as_u64).unwrap_or(500).min(MAX_WAIT_MS);
                tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
                Ok(format!("waited {ms}ms"))
            }
            other => Err(anyhow::anyhow!("unknown action: {other}")),
        };

        match result {
            Ok(msg) => Ok(ToolResult {
                success: true,
                output: json!({ "action": action, "result": msg, "ok": true }).to_string(),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::screen_control::CaptureResult;

    struct MockController;

    #[async_trait::async_trait]
    impl ScreenController for MockController {
        async fn capture(&self, _: Option<u32>) -> anyhow::Result<CaptureResult> {
            Ok(CaptureResult {
                data_uri: "data:image/jpeg;base64,abc".into(),
                orig_width: 2560,
                orig_height: 1600,
                resized_width: 1024,
                resized_height: 640,
                scale_x: 2.5,
                scale_y: 2.5,
                file_size_bytes: 1234,
            })
        }
        async fn click(&self, _x: i32, _y: i32) -> anyhow::Result<()> { Ok(()) }
        async fn double_click(&self, _x: i32, _y: i32) -> anyhow::Result<()> { Ok(()) }
        async fn right_click(&self, _x: i32, _y: i32) -> anyhow::Result<()> { Ok(()) }
        async fn type_text(&self, _: &str) -> anyhow::Result<()> { Ok(()) }
        async fn press_key(&self, _: &str) -> anyhow::Result<()> { Ok(()) }
        async fn scroll(&self, _: &str, _: u32) -> anyhow::Result<()> { Ok(()) }
        async fn drag(&self, _: (i32, i32), _: (i32, i32)) -> anyhow::Result<()> { Ok(()) }
        async fn move_cursor(&self, _: i32, _: i32) -> anyhow::Result<()> { Ok(()) }
        fn resolution(&self) -> (u32, u32) { (2560, 1600) }
    }

    #[test]
    fn snapshot_tool_name() {
        let tool = ScreenSnapshotTool::new(Arc::new(MockController), 1024);
        assert_eq!(tool.name(), "screen_snapshot");
    }

    #[test]
    fn input_tool_name() {
        let tool = ScreenInputTool::new(Arc::new(MockController));
        assert_eq!(tool.name(), "screen_input");
    }

    #[tokio::test]
    async fn snapshot_returns_data_uri() {
        let tool = ScreenSnapshotTool::new(Arc::new(MockController), 1024);
        let result = tool.execute(json!({})).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("data:image/jpeg;base64,"));
        assert!(result.output.contains("scale_x"));
    }

    #[tokio::test]
    async fn input_click_ok() {
        let tool = ScreenInputTool::new(Arc::new(MockController));
        let result = tool.execute(json!({"action": "click", "x": 100, "y": 200})).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn input_missing_action_fails() {
        let tool = ScreenInputTool::new(Arc::new(MockController));
        let result = tool.execute(json!({})).await.unwrap();
        assert!(!result.success);
    }

    #[tokio::test]
    async fn input_wait_ok() {
        let tool = ScreenInputTool::new(Arc::new(MockController));
        let result = tool.execute(json!({"action": "wait", "ms": 10})).await.unwrap();
        assert!(result.success);
    }
}
