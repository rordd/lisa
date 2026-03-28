//! LLM용 Tool 구현 — Anthropic Computer Use 호환 `computer` tool
//!
//! LLM은 이미지 좌표를 그대로 전달하면 tool 내부에서 scale 변환.

use super::ScreenController;
use crate::tools::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 마지막 캡처의 scale 정보 (screenshot ↔ 다른 action 공유)
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

// ── computer (Anthropic Computer Use 통합) ──────────────────────

pub struct ComputerTool {
    controller: Arc<dyn ScreenController>,
    default_width: u32,
    display_height: u32,
    scale: ScaleHandle,
}

impl ComputerTool {
    pub fn new(controller: Arc<dyn ScreenController>, default_width: u32, scale: ScaleHandle) -> Self {
        let (res_w, res_h) = controller.resolution();
        let display_height = if default_width > 0 && res_w > 0 {
            (default_width as f64 * res_h as f64 / res_w as f64).round() as u32
        } else {
            res_h
        };
        Self {
            controller,
            default_width,
            display_height,
            scale,
        }
    }

    /// 이미지 좌표 → 실제 화면 좌표
    async fn to_screen_coords(&self, x: i32, y: i32) -> (i32, i32) {
        let s = self.scale.read().await;
        (
            (x as f64 * s.scale_x).round() as i32,
            (y as f64 * s.scale_y).round() as i32,
        )
    }

    /// coordinate 배열 [x, y] 파싱
    fn parse_coordinate(args: &Value) -> anyhow::Result<(i32, i32)> {
        let coord = args
            .get("coordinate")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow::anyhow!("missing 'coordinate' array"))?;
        if coord.len() != 2 {
            anyhow::bail!("'coordinate' must be [x, y]");
        }
        let x = coord[0].as_i64().ok_or_else(|| anyhow::anyhow!("coordinate[0] must be integer"))? as i32;
        let y = coord[1].as_i64().ok_or_else(|| anyhow::anyhow!("coordinate[1] must be integer"))? as i32;
        Ok((x, y))
    }
}

#[async_trait]
impl Tool for ComputerTool {
    fn name(&self) -> &str {
        "computer"
    }

    fn description(&self) -> &str {
        "화면을 캡처하고, 클릭·타이핑·키입력·스크롤·드래그 등 화면 조작을 수행한다. \
        Anthropic Computer Use 스펙 호환. \
        screenshot action으로 먼저 화면을 캡처한 뒤, 좌표 기반 action을 수행할 것. \
        좌표는 screenshot 이미지에서 보이는 좌표를 [x, y] 배열로 전달 (자동 변환됨)."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["action"],
            "__display_width_px": self.default_width,
            "__display_height_px": self.display_height,
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["screenshot", "cursor_position",
                             "left_click", "right_click", "middle_click",
                             "double_click", "triple_click",
                             "mouse_move", "left_click_drag",
                             "left_mouse_down", "left_mouse_up",
                             "type", "key", "hold_key",
                             "scroll", "wait"],
                    "description": "수행할 액션 (Anthropic Computer Use 호환)"
                },
                "coordinate": {
                    "type": "array",
                    "items": { "type": "integer" },
                    "description": "[x, y] 좌표 (click/mouse_move/scroll 시, 이미지 좌표)"
                },
                "text": {
                    "type": "string",
                    "description": "입력 텍스트 (type 시) 또는 키 이름 (key/hold_key 시, 예: Return, Escape, ctrl+c)"
                },
                "scroll_direction": {
                    "type": "string",
                    "enum": ["up", "down", "left", "right"],
                    "description": "스크롤 방향 (scroll 시)"
                },
                "scroll_amount": {
                    "type": "integer",
                    "description": "스크롤 클릭 수 (scroll 시, 기본 3)"
                },
                "start_coordinate": {
                    "type": "array",
                    "items": { "type": "integer" },
                    "description": "드래그 시작 [x, y] (left_click_drag 시)"
                },
                "duration": {
                    "type": "number",
                    "description": "초 단위 (wait/hold_key 시, 최대 10)"
                }
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

        let result: anyhow::Result<String> = match action {
            // ── 캡처 ──
            "screenshot" => {
                let width = Some(self.default_width).filter(|&w| w > 0);
                match self.controller.capture(width).await {
                    Ok(capture) => {
                        {
                            let mut s = self.scale.write().await;
                            s.scale_x = capture.scale_x;
                            s.scale_y = capture.scale_y;
                        }
                        let now = chrono::Utc::now().format("%H:%M:%S%.3f");
                        Ok(format!(
                            "Screenshot at {now}: {}x{} (original: {}x{}, size: {} bytes)\n\
                            This is a FRESH capture of the current screen state.\n{}",
                            capture.resized_width, capture.resized_height,
                            capture.orig_width, capture.orig_height,
                            capture.file_size_bytes,
                            capture.data_uri,
                        ))
                    }
                    Err(e) => Err(e),
                }
            }
            "cursor_position" => {
                let (sx, sy) = self.controller.cursor_position().await?;
                // 화면 좌표 → 이미지 좌표로 역변환
                let s = self.scale.read().await;
                let ix = if s.scale_x > 0.0 { (sx as f64 / s.scale_x).round() as i32 } else { sx };
                let iy = if s.scale_y > 0.0 { (sy as f64 / s.scale_y).round() as i32 } else { sy };
                Ok(format!("X={ix},Y={iy}"))
            }

            // ── 클릭 ──
            "left_click" => {
                let (ix, iy) = Self::parse_coordinate(&args)?;
                let (x, y) = self.to_screen_coords(ix, iy).await;
                self.controller.click(x, y).await?;
                Ok(format!("left_clicked ({ix},{iy})→({x},{y})"))
            }
            "right_click" => {
                let (ix, iy) = Self::parse_coordinate(&args)?;
                let (x, y) = self.to_screen_coords(ix, iy).await;
                self.controller.right_click(x, y).await?;
                Ok(format!("right_clicked ({ix},{iy})→({x},{y})"))
            }
            "middle_click" => {
                let (ix, iy) = Self::parse_coordinate(&args)?;
                let (x, y) = self.to_screen_coords(ix, iy).await;
                self.controller.click(x, y).await?; // fallback to left click
                Ok(format!("middle_clicked→left ({ix},{iy})→({x},{y})"))
            }
            "double_click" => {
                let (ix, iy) = Self::parse_coordinate(&args)?;
                let (x, y) = self.to_screen_coords(ix, iy).await;
                self.controller.double_click(x, y).await?;
                Ok(format!("double_clicked ({ix},{iy})→({x},{y})"))
            }
            "triple_click" => {
                let (ix, iy) = Self::parse_coordinate(&args)?;
                let (x, y) = self.to_screen_coords(ix, iy).await;
                self.controller.triple_click(x, y).await?;
                Ok(format!("triple_clicked ({ix},{iy})→({x},{y})"))
            }

            // ── 마우스 이동/드래그 ──
            "mouse_move" => {
                let (ix, iy) = Self::parse_coordinate(&args)?;
                let (x, y) = self.to_screen_coords(ix, iy).await;
                self.controller.move_cursor(x, y).await?;
                Ok(format!("mouse_moved ({ix},{iy})→({x},{y})"))
            }
            "left_click_drag" => {
                let parse_coord = |key: &str| -> anyhow::Result<(i32, i32)> {
                    let coord = args.get(key).and_then(Value::as_array)
                        .ok_or_else(|| anyhow::anyhow!("missing '{key}'"))?;
                    if coord.len() != 2 { anyhow::bail!("'{key}' must be [x, y]"); }
                    Ok((
                        coord[0].as_i64().ok_or_else(|| anyhow::anyhow!("int"))? as i32,
                        coord[1].as_i64().ok_or_else(|| anyhow::anyhow!("int"))? as i32,
                    ))
                };
                let (ifx, ify) = parse_coord("start_coordinate")?;
                let (itx, ity) = parse_coord("coordinate")?;
                let (fx, fy) = self.to_screen_coords(ifx, ify).await;
                let (tx, ty) = self.to_screen_coords(itx, ity).await;
                self.controller.drag((fx, fy), (tx, ty)).await?;
                Ok(format!("dragged ({ifx},{ify})→({itx},{ity}) screen({fx},{fy})→({tx},{ty})"))
            }
            "left_mouse_down" => {
                let (ix, iy) = Self::parse_coordinate(&args)?;
                let (x, y) = self.to_screen_coords(ix, iy).await;
                self.controller.mouse_down(x, y).await?;
                Ok(format!("mouse_down ({ix},{iy})→({x},{y})"))
            }
            "left_mouse_up" => {
                let (ix, iy) = Self::parse_coordinate(&args)?;
                let (x, y) = self.to_screen_coords(ix, iy).await;
                self.controller.mouse_up(x, y).await?;
                Ok(format!("mouse_up ({ix},{iy})→({x},{y})"))
            }

            // ── 키보드 ──
            "type" => {
                let text = args.get("text").and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("missing 'text'"))?;
                self.controller.type_text(text).await?;
                Ok(format!("typed {} chars", text.len()))
            }
            "key" => {
                let key = args.get("text").and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("missing 'text'"))?;
                self.controller.press_key(key).await?;
                Ok(format!("pressed key: {key}"))
            }
            "hold_key" => {
                let key = args.get("text").and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("missing 'text'"))?;
                let duration_secs = args.get("duration").and_then(Value::as_f64).unwrap_or(1.0).min(10.0);
                self.controller.press_key(key).await?;
                tokio::time::sleep(std::time::Duration::from_secs_f64(duration_secs)).await;
                Ok(format!("held key: {key} for {duration_secs}s"))
            }

            // ── 스크롤 ──
            "scroll" => {
                // Anthropic: scroll_direction + scroll_amount
                // 폴백: direction + amount (기존 호환)
                let dir = args.get("scroll_direction").and_then(Value::as_str)
                    .or_else(|| args.get("direction").and_then(Value::as_str))
                    .unwrap_or("down");
                let amount = args.get("scroll_amount").and_then(Value::as_u64)
                    .or_else(|| args.get("amount").and_then(Value::as_u64))
                    .unwrap_or(3) as u32;
                if let Ok((ix, iy)) = Self::parse_coordinate(&args) {
                    let (sx, sy) = self.to_screen_coords(ix, iy).await;
                    self.controller.move_cursor(sx, sy).await?;
                } else {
                    let (w, h) = self.controller.resolution();
                    self.controller.move_cursor(w as i32 / 2, h as i32 / 2).await?;
                }
                self.controller.scroll(dir, amount).await?;
                Ok(format!("scrolled {dir} {amount}"))
            }

            // ── 대기 ──
            "wait" => {
                let duration_secs = args.get("duration").and_then(Value::as_f64).unwrap_or(1.0).min(10.0);
                let ms = (duration_secs * 1000.0) as u64;
                tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
                Ok(format!("waited {duration_secs}s"))
            }

            other => Err(anyhow::anyhow!("unknown action: {other}")),
        };

        match result {
            Ok(msg) => {
                // screenshot/cursor_position/wait → 그대로 리턴
                // 나머지 action → 자동 screenshot 첨부 (모델이 화면 변화를 볼 수 있도록)
                let needs_auto_screenshot = !matches!(action, "screenshot" | "cursor_position" | "wait");
                let output = if action == "screenshot" {
                    msg
                } else if needs_auto_screenshot {
                    // action 후 잠깐 대기 (UI 반영 시간)
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                    let width = Some(self.default_width).filter(|&w| w > 0);
                    match self.controller.capture(width).await {
                        Ok(capture) => {
                            {
                                let mut s = self.scale.write().await;
                                s.scale_x = capture.scale_x;
                                s.scale_y = capture.scale_y;
                            }
                            let now = chrono::Utc::now().format("%H:%M:%S%.3f");
                            format!(
                                "{}\nScreenshot at {now}: {}x{} (auto-capture after {action})\n{}",
                                msg,
                                capture.resized_width, capture.resized_height,
                                capture.data_uri,
                            )
                        }
                        Err(e) => {
                            format!("{}\n(auto-screenshot failed: {})", msg, e)
                        }
                    }
                } else {
                    json!({ "action": action, "result": msg, "ok": true }).to_string()
                };
                Ok(ToolResult { success: true, output, error: None })
            }
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
        async fn cursor_position(&self) -> anyhow::Result<(i32, i32)> { Ok((500, 300)) }
        async fn mouse_down(&self, _: i32, _: i32) -> anyhow::Result<()> { Ok(()) }
        async fn mouse_up(&self, _: i32, _: i32) -> anyhow::Result<()> { Ok(()) }
        async fn triple_click(&self, _: i32, _: i32) -> anyhow::Result<()> { Ok(()) }
        fn resolution(&self) -> (u32, u32) { (2560, 1600) }
    }

    fn make_tool() -> ComputerTool {
        let ctrl = Arc::new(MockController);
        let scale = new_scale_handle();
        ComputerTool::new(ctrl, 1024, scale)
    }

    #[test]
    fn computer_tool_name() {
        let tool = make_tool();
        assert_eq!(tool.name(), "computer");
    }

    #[tokio::test]
    async fn screenshot_returns_data_uri() {
        let tool = make_tool();
        let result = tool.execute(json!({"action": "screenshot"})).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("data:image/jpeg;base64,"));
    }

    #[tokio::test]
    async fn screenshot_updates_scale() {
        let tool = make_tool();
        tool.execute(json!({"action": "screenshot"})).await.unwrap();
        // MockController returns 2560/1024=2.5 scale
        let (sx, sy) = tool.to_screen_coords(100, 100).await;
        assert_eq!(sx, 250);
        assert_eq!(sy, 250);
    }

    #[tokio::test]
    async fn click_ok() {
        let tool = make_tool();
        let result = tool.execute(json!({"action": "left_click", "coordinate": [100, 200]})).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn missing_action_fails() {
        let tool = make_tool();
        let result = tool.execute(json!({})).await.unwrap();
        assert!(!result.success);
    }

    #[tokio::test]
    async fn wait_ok() {
        let tool = make_tool();
        let result = tool.execute(json!({"action": "wait", "duration": 10})).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn scroll_with_coordinate() {
        let tool = make_tool();
        let result = tool
            .execute(json!({"action": "scroll", "coordinate": [500, 300], "direction": "down", "amount": 3}))
            .await
            .unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn key_action_uses_text_field() {
        let tool = make_tool();
        let result = tool.execute(json!({"action": "key", "text": "Return"})).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn type_action() {
        let tool = make_tool();
        let result = tool.execute(json!({"action": "type", "text": "hello"})).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn drag_action() {
        let tool = make_tool();
        let result = tool
            .execute(json!({
                "action": "left_click_drag",
                "start_coordinate": [100, 100],
                "coordinate": [200, 200]
            }))
            .await
            .unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn mouse_move_action() {
        let tool = make_tool();
        let result = tool.execute(json!({"action": "mouse_move", "coordinate": [300, 400]})).await.unwrap();
        assert!(result.success);
    }
}
