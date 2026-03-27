//! LLM용 Tool 구현 — screen_snapshot + screen_input
//!
//! LLM은 이미지 좌표를 그대로 전달하면 tool 내부에서 scale 변환.

use super::ScreenController;
use crate::tools::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 마지막 캡처의 scale 정보 (snapshot ↔ input 공유)
#[derive(Debug, Clone)]
pub(crate) struct ScaleInfo {
    scale_x: f64,
    scale_y: f64,
}

impl Default for ScaleInfo {
    fn default() -> Self {
        Self {
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }
}

/// scale 공유를 위한 핸들
pub type ScaleHandle = Arc<RwLock<ScaleInfo>>;

pub fn new_scale_handle() -> ScaleHandle {
    Arc::new(RwLock::new(ScaleInfo::default()))
}

// ── screen_snapshot ──────────────────────────────────────────────

pub struct ScreenSnapshotTool {
    controller: Arc<dyn ScreenController>,
    default_width: u32,
    scale: ScaleHandle,
}

impl ScreenSnapshotTool {
    pub fn new(controller: Arc<dyn ScreenController>, default_width: u32, scale: ScaleHandle) -> Self {
        Self {
            controller,
            default_width,
            scale,
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
        JPEG 이미지(base64)와 해상도 정보를 리턴한다. \
        좌표 기반 조작 전에 반드시 먼저 호출할 것. \
        screen_input에 전달하는 좌표는 이 이미지에서 보이는 좌표 그대로 사용하면 된다 (자동 변환됨)."
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
                // scale 저장 — screen_input이 자동 변환에 사용
                {
                    let mut s = self.scale.write().await;
                    s.scale_x = result.scale_x;
                    s.scale_y = result.scale_y;
                }
                let text = format!(
                    "Screenshot: {}x{} (original: {}x{}, size: {} bytes)\n\
                    Use coordinates as seen in this image — they will be automatically converted to screen coordinates.\n\
                    {}",
                    result.resized_width,
                    result.resized_height,
                    result.orig_width,
                    result.orig_height,
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
    scale: ScaleHandle,
}

impl ScreenInputTool {
    pub fn new(controller: Arc<dyn ScreenController>, scale: ScaleHandle) -> Self {
        Self { controller, scale }
    }

    /// 이미지 좌표 → 실제 화면 좌표
    async fn to_screen_coords(&self, x: i32, y: i32) -> (i32, i32) {
        let s = self.scale.read().await;
        ((x as f64 * s.scale_x).round() as i32, (y as f64 * s.scale_y).round() as i32)
    }
}

#[async_trait]
impl Tool for ScreenInputTool {
    fn name(&self) -> &str {
        "screen_input"
    }

    fn description(&self) -> &str {
        "화면에 입력을 보낸다 (클릭, 텍스트 입력, 키 입력, 스크롤, 드래그). \
        좌표는 screen_snapshot 이미지에서 보이는 좌표를 그대로 사용 (자동으로 실제 화면 좌표로 변환됨). \
        주의: type 액션은 시스템 클립보드를 덮어쓴다."
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
                "x": { "type": "integer", "description": "X 좌표 (click/double_click/right_click/move/scroll 시, 이미지 좌표)" },
                "y": { "type": "integer", "description": "Y 좌표 (click/double_click/right_click/move/scroll 시, 이미지 좌표)" },
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
                let (ix, iy) = get_xy(&args)?;
                let (x, y) = self.to_screen_coords(ix, iy).await;
                self.controller.click(x, y).await?;
                Ok(format!("clicked image({ix},{iy}) → screen({x},{y})"))
            }
            "double_click" => {
                let (ix, iy) = get_xy(&args)?;
                let (x, y) = self.to_screen_coords(ix, iy).await;
                self.controller.double_click(x, y).await?;
                Ok(format!("double_clicked image({ix},{iy}) → screen({x},{y})"))
            }
            "right_click" => {
                let (ix, iy) = get_xy(&args)?;
                let (x, y) = self.to_screen_coords(ix, iy).await;
                self.controller.right_click(x, y).await?;
                Ok(format!("right_clicked image({ix},{iy}) → screen({x},{y})"))
            }
            "move" => {
                let (ix, iy) = get_xy(&args)?;
                let (x, y) = self.to_screen_coords(ix, iy).await;
                self.controller.move_cursor(x, y).await?;
                Ok(format!("moved cursor image({ix},{iy}) → screen({x},{y})"))
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
                    .get("key")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("missing 'key'"))?;
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
                    .unwrap_or(5) as u32;
                // x,y 지정 시 해당 위치로 먼저 이동 (미지정 시 화면 중앙)
                if let (Some(ix), Some(iy)) = (
                    args.get("x").and_then(Value::as_i64),
                    args.get("y").and_then(Value::as_i64),
                ) {
                    let (sx, sy) = self.to_screen_coords(ix as i32, iy as i32).await;
                    self.controller.move_cursor(sx, sy).await?;
                } else {
                    // 화면 중앙으로 이동
                    let (w, h) = self.controller.resolution();
                    self.controller.move_cursor(w as i32 / 2, h as i32 / 2).await?;
                }
                self.controller.scroll(dir, amount).await?;
                Ok(format!("scrolled {dir} {amount}"))
            }
            "drag" => {
                let ifx = args.get("from_x").and_then(Value::as_i64).ok_or_else(|| anyhow::anyhow!("missing 'from_x'"))? as i32;
                let ify = args.get("from_y").and_then(Value::as_i64).ok_or_else(|| anyhow::anyhow!("missing 'from_y'"))? as i32;
                let itx = args.get("to_x").and_then(Value::as_i64).ok_or_else(|| anyhow::anyhow!("missing 'to_x'"))? as i32;
                let ity = args.get("to_y").and_then(Value::as_i64).ok_or_else(|| anyhow::anyhow!("missing 'to_y'"))? as i32;
                let (fx, fy) = self.to_screen_coords(ifx, ify).await;
                let (tx, ty) = self.to_screen_coords(itx, ity).await;
                self.controller.drag((fx, fy), (tx, ty)).await?;
                Ok(format!("dragged image({ifx},{ify})→({itx},{ity}) screen({fx},{fy})→({tx},{ty})"))
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

    fn make_tools() -> (ScreenSnapshotTool, ScreenInputTool) {
        let ctrl = Arc::new(MockController);
        let scale = new_scale_handle();
        (
            ScreenSnapshotTool::new(ctrl.clone(), 1024, scale.clone()),
            ScreenInputTool::new(ctrl, scale),
        )
    }

    #[test]
    fn snapshot_tool_name() {
        let (snap, _) = make_tools();
        assert_eq!(snap.name(), "screen_snapshot");
    }

    #[test]
    fn input_tool_name() {
        let (_, input) = make_tools();
        assert_eq!(input.name(), "screen_input");
    }

    #[tokio::test]
    async fn snapshot_returns_data_uri() {
        let (snap, _) = make_tools();
        let result = snap.execute(json!({})).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("data:image/jpeg;base64,"));
    }

    #[tokio::test]
    async fn snapshot_updates_scale() {
        let (snap, input) = make_tools();
        snap.execute(json!({})).await.unwrap();
        // MockController returns 2560/1024=2.5 scale
        let (sx, sy) = input.to_screen_coords(100, 100).await;
        assert_eq!(sx, 250);
        assert_eq!(sy, 250);
    }

    #[tokio::test]
    async fn input_click_ok() {
        let (_, input) = make_tools();
        let result = input.execute(json!({"action": "click", "x": 100, "y": 200})).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn input_missing_action_fails() {
        let (_, input) = make_tools();
        let result = input.execute(json!({})).await.unwrap();
        assert!(!result.success);
    }

    #[tokio::test]
    async fn input_wait_ok() {
        let (_, input) = make_tools();
        let result = input.execute(json!({"action": "wait", "ms": 10})).await.unwrap();
        assert!(result.success);
    }
}
