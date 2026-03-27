//! macOS ScreenController 구현
//!
//! 의존성:
//! - `screencapture` — macOS 내장
//! - `sips`          — macOS 내장 (리사이즈, JPEG 변환)
//! - `cliclick`      — brew install cliclick (마우스 제어)
//! - `osascript`     — macOS 내장 (키보드, 클립보드 paste)
//! - `pbcopy`        — macOS 내장 (클립보드)

use super::{CaptureResult, ScreenController};
use anyhow::{Context, Result};
use async_trait::async_trait;
use base64::Engine;
use std::time::Duration;
use tokio::process::Command;

const DEFAULT_RESIZE_WIDTH: u32 = 1024;
const COMMAND_TIMEOUT_SECS: u64 = 15;

pub struct MacScreenController {
    /// 기본 리사이즈 폭
    pub default_resize_width: u32,
}

impl MacScreenController {
    pub fn new(default_resize_width: u32) -> Self {
        Self {
            default_resize_width,
        }
    }

    async fn run(&self, program: &str, args: &[&str]) -> Result<String> {
        let output = tokio::time::timeout(
            Duration::from_secs(COMMAND_TIMEOUT_SECS),
            Command::new(program).args(args).output(),
        )
        .await
        .with_context(|| format!("{program} timed out"))?
        .with_context(|| format!("failed to run {program}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("{program} failed: {stderr}");
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// sips로 픽셀 크기 읽기
    async fn sips_dimensions(&self, path: &str) -> Result<(u32, u32)> {
        let out = self
            .run("sips", &["-g", "pixelWidth", "-g", "pixelHeight", path])
            .await?;
        let w = out
            .lines()
            .find(|l| l.contains("pixelWidth"))
            .and_then(|l| l.split_whitespace().last())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let h = out
            .lines()
            .find(|l| l.contains("pixelHeight"))
            .and_then(|l| l.split_whitespace().last())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        Ok((w, h))
    }

    /// osascript key code 전송
    async fn key_code(&self, code: u16) -> Result<()> {
        self.run(
            "osascript",
            &[
                "-e",
                &format!(
                    "tell application \"System Events\" to key code {code}"
                ),
            ],
        )
        .await?;
        Ok(())
    }
}

#[async_trait]
impl ScreenController for MacScreenController {
    async fn capture(&self, resize_width: Option<u32>) -> Result<CaptureResult> {
        let target_width = resize_width.unwrap_or(self.default_resize_width);
        let png_tmp = tempfile::Builder::new()
            .prefix("lisa_snap_")
            .suffix(".png")
            .tempfile()
            .context("failed to create temp png")?;
        let jpg_tmp = tempfile::Builder::new()
            .prefix("lisa_snap_")
            .suffix(".jpg")
            .tempfile()
            .context("failed to create temp jpg")?;
        let png = png_tmp.path().to_string_lossy().to_string();
        let jpg = jpg_tmp.path().to_string_lossy().to_string();

        // 1. 캡처 (-x: 소리 없음)
        self.run("screencapture", &["-x", &png]).await?;

        // 2. 원본 해상도
        let (orig_w, orig_h) = self.sips_dimensions(&png).await?;

        // 3. 리사이즈 (원본보다 클 때만)
        if target_width > 0 && orig_w > target_width {
            self.run(
                "sips",
                &["--resampleWidth", &target_width.to_string(), &png],
            )
            .await?;
        }

        // 4. PNG → JPEG (크기 대폭 감소)
        self.run(
            "sips",
            &[
                "-s", "format", "jpeg",
                "-s", "formatOptions", "70",
                &png, "--out", &jpg,
            ],
        )
        .await?;

        // 5. 리사이즈 후 해상도
        let (resized_w, resized_h) = self.sips_dimensions(&jpg).await?;

        // 6. base64
        let bytes = tokio::fs::read(&jpg)
            .await
            .context("failed to read screenshot jpeg")?;
        let file_size = bytes.len() as u64;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

        // 7. 정리 — tempfile이 drop 시 자동 삭제, 명시적으로도 해둠
        drop(png_tmp);
        drop(jpg_tmp);

        let scale_x = if resized_w > 0 {
            orig_w as f64 / resized_w as f64
        } else {
            1.0
        };
        let scale_y = if resized_h > 0 {
            orig_h as f64 / resized_h as f64
        } else {
            1.0
        };

        Ok(CaptureResult {
            data_uri: format!("data:image/jpeg;base64,{b64}"),
            orig_width: orig_w,
            orig_height: orig_h,
            resized_width: resized_w,
            resized_height: resized_h,
            scale_x,
            scale_y,
            file_size_bytes: file_size,
        })
    }

    async fn click(&self, x: i32, y: i32) -> Result<()> {
        self.run("cliclick", &[&format!("c:{x},{y}")]).await?;
        Ok(())
    }

    async fn double_click(&self, x: i32, y: i32) -> Result<()> {
        self.run("cliclick", &[&format!("dc:{x},{y}")]).await?;
        Ok(())
    }

    async fn right_click(&self, x: i32, y: i32) -> Result<()> {
        self.run("cliclick", &[&format!("rc:{x},{y}")]).await?;
        Ok(())
    }

    async fn type_text(&self, text: &str) -> Result<()> {
        // ⚠️ 사이드이펙트: 시스템 클립보드를 덮어씀
        // 클립보드에 넣고 Cmd+V — IME 없이 한글 포함 모든 텍스트 입력 가능
        let mut child = tokio::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("failed to spawn pbcopy")?;

        if let Some(stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            let mut stdin = stdin;
            stdin
                .write_all(text.as_bytes())
                .await
                .context("failed to write to pbcopy")?;
        }
        child.wait().await.context("pbcopy failed")?;

        // Cmd+V
        self.run(
            "osascript",
            &[
                "-e",
                "tell application \"System Events\" to keystroke \"v\" using command down",
            ],
        )
        .await?;
        Ok(())
    }

    async fn press_key(&self, key: &str) -> Result<()> {
        let code: u16 = match key.to_ascii_lowercase().as_str() {
            "return" | "enter" => 36,
            "escape" | "esc" => 53,
            "tab" => 48,
            "space" => 49,
            "delete" | "backspace" => 51,
            "up" => 126,
            "down" => 125,
            "left" => 123,
            "right" => 124,
            "home" => 115,
            "end" => 119,
            "pageup" => 116,
            "pagedown" => 121,
            _ => {
                // 단일 ASCII 영숫자/기호만 허용 — injection 방지
                let ch = key.chars().next();
                if key.len() == 1 && ch.map_or(false, |c| c.is_ascii_graphic()) {
                    self.run(
                        "osascript",
                        &[
                            "-e",
                            &format!(
                                "tell application \"System Events\" to keystroke \"{key}\""
                            ),
                        ],
                    )
                    .await?;
                    return Ok(());
                }
                anyhow::bail!("unsupported key: {key}");
            }
        };
        self.key_code(code).await
    }

    async fn scroll(&self, direction: &str, amount: u32) -> Result<()> {
        let prefix = match direction.to_ascii_lowercase().as_str() {
            "up" => "su",
            "down" => "sd",
            "left" => "sl",
            "right" => "sr",
            _ => anyhow::bail!("unknown scroll direction: {direction}"),
        };
        self.run("cliclick", &[&format!("{prefix}:{amount}")]).await?;
        Ok(())
    }

    async fn drag(&self, from: (i32, i32), to: (i32, i32)) -> Result<()> {
        self.run(
            "cliclick",
            &[
                &format!("dd:{},{}", from.0, from.1),
                &format!("du:{},{}", to.0, to.1),
            ],
        )
        .await?;
        Ok(())
    }

    async fn move_cursor(&self, x: i32, y: i32) -> Result<()> {
        self.run("cliclick", &[&format!("m:{x},{y}")]).await?;
        Ok(())
    }

    fn resolution(&self) -> (u32, u32) {
        // capture() 결과의 orig_width/orig_height가 실제 해상도이므로
        // 이 함수는 힌트용 — capture 전 대략적 해상도 제공
        // system_profiler 파싱은 느리고 복잡하므로 screencapture로 직접 측정
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            if let Ok(result) = rt.block_on(async {
                let tmp = tempfile::Builder::new()
                    .prefix("lisa_res_")
                    .suffix(".png")
                    .tempfile()?;
                let path = tmp.path().to_string_lossy().to_string();
                tokio::process::Command::new("screencapture")
                    .args(&["-x", &path])
                    .output()
                    .await?;
                self.sips_dimensions(&path).await
            }) {
                return result;
            }
        }
        (2560, 1600) // fallback
    }
}
