//! Screen control — 화면 캡처 + 입력 제어 통합 모듈
//!
//! `ScreenController` trait으로 플랫폼 추상화:
//! - `MacScreenController`  — macOS (screencapture, cliclick, osascript)
//! - `LinuxScreenController` — Linux (scrot/gnome-screenshot, xdotool) — TODO
//! - `WebOSScreenController` — webOS TV (luna-send) — TODO
//!
//! LLM에는 하나의 tool로 노출:
//! - `computer` — Anthropic Computer Use 호환 (screenshot + click/type/key/scroll/drag)

use anyhow::Result;
use async_trait::async_trait;

pub mod mac;
pub mod tool;

pub use tool::ComputerTool;

/// 캡처 결과
#[derive(Debug, Clone)]
pub struct CaptureResult {
    /// `data:image/jpeg;base64,...` 형식 (vision provider가 이미지로 인식)
    pub data_uri: String,
    pub orig_width: u32,
    pub orig_height: u32,
    pub resized_width: u32,
    pub resized_height: u32,
    /// 좌표 역변환 계수: 이미지 좌표 × scale = 실제 화면 좌표
    pub scale_x: f64,
    pub scale_y: f64,
    pub file_size_bytes: u64,
}

/// 플랫폼별 화면 제어 추상화
#[async_trait]
pub trait ScreenController: Send + Sync {
    /// 화면 캡처 → JPEG base64
    /// `resize_width`: 리사이즈 폭 (None이면 기본값 사용)
    async fn capture(&self, resize_width: Option<u32>) -> Result<CaptureResult>;

    /// 좌표 클릭
    async fn click(&self, x: i32, y: i32) -> Result<()>;

    /// 더블 클릭
    async fn double_click(&self, x: i32, y: i32) -> Result<()>;

    /// 우클릭
    async fn right_click(&self, x: i32, y: i32) -> Result<()>;

    /// 텍스트 입력 (클립보드 paste 방식 — IME/한글 지원)
    async fn type_text(&self, text: &str) -> Result<()>;

    /// 키 입력 (return, escape, tab, space, delete, up, down, left, right, home, end)
    async fn press_key(&self, key: &str) -> Result<()>;

    /// 스크롤 (direction: up/down/left/right, amount: 클릭 수)
    async fn scroll(&self, direction: &str, amount: u32) -> Result<()>;

    /// 드래그
    async fn drag(&self, from: (i32, i32), to: (i32, i32)) -> Result<()>;

    /// 커서 이동 (클릭 없음)
    async fn move_cursor(&self, x: i32, y: i32) -> Result<()>;

    /// 화면 해상도
    fn resolution(&self) -> (u32, u32);
}
