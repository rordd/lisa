//! macOS ScreenController 구현
//!
//! 의존성:
//! - `screencapture` — macOS 내장
//! - `sips`          — macOS 내장 (리사이즈, JPEG 변환)
//! - `cliclick`      — brew install cliclick (마우스 제어)
//! - `swift`         — macOS 내장 (CGEvent 키보드/스크롤)
//! - `pbcopy`        — macOS 내장 (클립보드)

use super::{CaptureResult, ScreenController};
use anyhow::{Context, Result};
use async_trait::async_trait;
use base64::Engine;
use std::time::Duration;
use tokio::process::Command;

const COMMAND_TIMEOUT_SECS: u64 = 15;

pub struct MacScreenController {
    pub default_resize_width: u32,
}

impl MacScreenController {
    pub fn new(default_resize_width: u32) -> Self {
        Self { default_resize_width }
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

    async fn sips_dimensions(&self, path: &str) -> Result<(u32, u32)> {
        let out = self.run("sips", &["-g", "pixelWidth", "-g", "pixelHeight", path]).await?;
        let w = out.lines().find(|l| l.contains("pixelWidth"))
            .and_then(|l| l.split_whitespace().last())
            .and_then(|v| v.parse().ok()).unwrap_or(0);
        let h = out.lines().find(|l| l.contains("pixelHeight"))
            .and_then(|l| l.split_whitespace().last())
            .and_then(|v| v.parse().ok()).unwrap_or(0);
        Ok((w, h))
    }

    /// CGEvent binary path (precompiled for speed: ~15ms vs swift -e ~150ms)
    fn cgevent_bin() -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        format!("{home}/.zeroclaw/bin/cgevent")
    }

    async fn key_code(&self, code: u16) -> Result<()> {
        self.run(&Self::cgevent_bin(), &["key", &code.to_string()]).await?;
        Ok(())
    }

    /// 콤보 키: "ctrl+c", "cmd+shift+s", "super+l" 등
    async fn press_combo(&self, combo: &str) -> Result<()> {
        let parts: Vec<&str> = combo.split('+').map(|s| s.trim()).collect();
        if parts.len() < 2 {
            anyhow::bail!("invalid combo: {combo}");
        }
        let mut flags: u64 = 0;
        for part in &parts[..parts.len() - 1] {
            match part.to_ascii_lowercase().as_str() {
                "cmd" | "command" | "super" => flags |= 0x100000,
                "ctrl" | "control" => flags |= 0x40000,
                "alt" | "option" | "opt" => flags |= 0x80000,
                "shift" => flags |= 0x20000,
                _ => anyhow::bail!("unknown modifier: {part}"),
            }
        }
        let key_part = parts.last().unwrap();
        let key_code: u16 = Self::name_to_keycode(key_part)?;
        self.run(&Self::cgevent_bin(), &["key", &key_code.to_string(), &flags.to_string()]).await?;
        Ok(())
    }

    fn name_to_keycode(name: &str) -> Result<u16> {
        Ok(match name.to_ascii_lowercase().as_str() {
            // 알파벳 (macOS virtual key codes)
            "a" => 0, "s" => 1, "d" => 2, "f" => 3, "h" => 4, "g" => 5,
            "z" => 6, "x" => 7, "c" => 8, "v" => 9, "b" => 11, "q" => 12,
            "w" => 13, "e" => 14, "r" => 15, "y" => 16, "t" => 17,
            "o" => 31, "u" => 32, "i" => 34, "p" => 35, "l" => 37,
            "j" => 38, "k" => 40, "n" => 45, "m" => 46,
            // 숫자
            "0" => 29, "1" => 18, "2" => 19, "3" => 20, "4" => 21,
            "5" => 23, "6" => 22, "7" => 26, "8" => 28, "9" => 25,
            // 특수문자
            "-" | "minus" => 27, "=" | "equal" => 24,
            "[" | "leftbracket" => 33, "]" | "rightbracket" => 30,
            "\\" | "backslash" => 42, ";" | "semicolon" => 41,
            "'" | "quote" => 39, "," | "comma" => 43,
            "." | "period" => 47, "/" | "slash" => 44, "`" | "grave" => 50,
            // 제어 키
            "return" | "enter" => 36, "tab" => 48, "space" => 49,
            "delete" | "backspace" => 51, "forwarddelete" => 117,
            "escape" | "esc" => 53,
            // 방향 / 내비게이션
            "up" => 126, "down" => 125, "left" => 123, "right" => 124,
            "home" => 115, "end" => 119,
            "pageup" | "page_up" => 116, "pagedown" | "page_down" => 121,
            // F키
            "f1" => 122, "f2" => 120, "f3" => 99, "f4" => 118,
            "f5" => 96, "f6" => 97, "f7" => 98, "f8" => 100,
            "f9" => 101, "f10" => 109, "f11" => 103, "f12" => 111,
            _ => anyhow::bail!("unsupported key: {name}"),
        })
    }
}

#[async_trait]
impl ScreenController for MacScreenController {
    async fn capture(&self, resize_width: Option<u32>) -> Result<CaptureResult> {
        let target_width = resize_width.unwrap_or(self.default_resize_width);
        let png_tmp = tempfile::Builder::new().prefix("lisa_snap_").suffix(".png")
            .tempfile().context("failed to create temp png")?;
        let jpg_tmp = tempfile::Builder::new().prefix("lisa_snap_").suffix(".jpg")
            .tempfile().context("failed to create temp jpg")?;
        let png = png_tmp.path().to_string_lossy().to_string();
        let jpg = jpg_tmp.path().to_string_lossy().to_string();

        self.run("screencapture", &["-x", &png]).await?;
        let (orig_w, orig_h) = self.sips_dimensions(&png).await?;

        if target_width > 0 && orig_w > target_width {
            self.run("sips", &["--resampleWidth", &target_width.to_string(), &png]).await?;
        }
        self.run("sips", &["-s", "format", "jpeg", "-s", "formatOptions", "40", &png, "--out", &jpg]).await?;

        let (resized_w, resized_h) = self.sips_dimensions(&jpg).await?;
        let bytes = tokio::fs::read(&jpg).await.context("failed to read jpeg")?;
        let file_size = bytes.len() as u64;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        drop(png_tmp);
        drop(jpg_tmp);

        let scale_x = if resized_w > 0 { orig_w as f64 / resized_w as f64 } else { 1.0 };
        let scale_y = if resized_h > 0 { orig_h as f64 / resized_h as f64 } else { 1.0 };

        Ok(CaptureResult {
            data_uri: format!("data:image/jpeg;base64,{b64}"),
            orig_width: orig_w, orig_height: orig_h,
            resized_width: resized_w, resized_height: resized_h,
            scale_x, scale_y, file_size_bytes: file_size,
        })
    }

    async fn click(&self, x: i32, y: i32) -> Result<()> {
        self.run("cliclick", &[&format!("c:{x},{y}")]).await?; Ok(())
    }
    async fn double_click(&self, x: i32, y: i32) -> Result<()> {
        self.run("cliclick", &[&format!("dc:{x},{y}")]).await?; Ok(())
    }
    async fn right_click(&self, x: i32, y: i32) -> Result<()> {
        self.run("cliclick", &[&format!("rc:{x},{y}")]).await?; Ok(())
    }
    async fn triple_click(&self, x: i32, y: i32) -> Result<()> {
        self.run("cliclick", &[&format!("tc:{x},{y}")]).await?; Ok(())
    }

    async fn type_text(&self, text: &str) -> Result<()> {
        // ⚠️ 클립보드 덮어씀 — pbcopy + Cmd+V (IME/한글 지원)
        let mut child = tokio::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn().context("failed to spawn pbcopy")?;
        if let Some(stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            let mut stdin = stdin;
            stdin.write_all(text.as_bytes()).await.context("pbcopy write")?;
        }
        child.wait().await.context("pbcopy failed")?;
        self.run("osascript", &["-e", "tell application \"System Events\" to keystroke \"v\" using command down"]).await?;
        Ok(())
    }

    async fn press_key(&self, key: &str) -> Result<()> {
        if key.contains('+') {
            return self.press_combo(key).await;
        }
        // name_to_keycode 통합 사용 (중복 매핑 제거)
        match Self::name_to_keycode(key) {
            Ok(code) => self.key_code(code).await,
            Err(_) => {
                // 단일 ASCII 문자 → unicode 방식
                let ch = key.chars().next();
                if key.len() == 1 && ch.map_or(false, |c| c.is_ascii_graphic()) {
                    self.run(&Self::cgevent_bin(), &["unicode", &(ch.unwrap() as u32).to_string()]).await?;
                    Ok(())
                } else {
                    anyhow::bail!("unsupported key: {key}")
                }
            }
        }
    }

    async fn scroll(&self, direction: &str, amount: u32) -> Result<()> {
        let (wheel1, wheel2) = match direction.to_ascii_lowercase().as_str() {
            "down" => (-(amount as i32), 0),
            "up" => (amount as i32, 0),
            "left" => (0, amount as i32),
            "right" => (0, -(amount as i32)),
            _ => anyhow::bail!("unknown scroll direction: {direction}"),
        };
        self.run(&Self::cgevent_bin(), &["scroll", &wheel1.to_string(), &wheel2.to_string()]).await?;
        Ok(())
    }

    async fn drag(&self, from: (i32, i32), to: (i32, i32)) -> Result<()> {
        self.run("cliclick", &[&format!("dd:{},{}", from.0, from.1), &format!("du:{},{}", to.0, to.1)]).await?;
        Ok(())
    }

    async fn move_cursor(&self, x: i32, y: i32) -> Result<()> {
        self.run("cliclick", &[&format!("m:{x},{y}")]).await?; Ok(())
    }

    async fn cursor_position(&self) -> Result<(i32, i32)> {
        let text = self.run("cliclick", &["p"]).await?;
        let parts: Vec<&str> = text.trim().split(',').collect();
        if parts.len() >= 2 {
            Ok((parts[0].trim().parse().unwrap_or(0), parts[1].trim().parse().unwrap_or(0)))
        } else {
            anyhow::bail!("failed to parse cursor position: {text}");
        }
    }

    async fn hold_key(&self, key: &str, duration_secs: f64) -> Result<()> {
        // 콤보 키 or 단일 키 → keycode 결정
        let (code, flags) = if key.contains('+') {
            let parts: Vec<&str> = key.split('+').map(|s| s.trim()).collect();
            let mut f: u64 = 0;
            for part in &parts[..parts.len() - 1] {
                match part.to_ascii_lowercase().as_str() {
                    "cmd" | "command" | "super" => f |= 0x100000,
                    "ctrl" | "control" => f |= 0x40000,
                    "alt" | "option" | "opt" => f |= 0x80000,
                    "shift" => f |= 0x20000,
                    _ => anyhow::bail!("unknown modifier: {part}"),
                }
            }
            (Self::name_to_keycode(parts.last().unwrap())?, f)
        } else {
            (Self::name_to_keycode(key)?, 0u64)
        };

        let bin = Self::cgevent_bin();
        // keydown
        if flags > 0 {
            self.run(&bin, &["keydown", &code.to_string(), &flags.to_string()]).await?;
        } else {
            self.run(&bin, &["keydown", &code.to_string()]).await?;
        }
        // hold
        tokio::time::sleep(std::time::Duration::from_secs_f64(duration_secs.min(10.0))).await;
        // keyup
        if flags > 0 {
            self.run(&bin, &["keyup", &code.to_string(), &flags.to_string()]).await?;
        } else {
            self.run(&bin, &["keyup", &code.to_string()]).await?;
        }
        Ok(())
    }

    async fn mouse_down(&self, x: i32, y: i32) -> Result<()> {
        self.run("cliclick", &[&format!("dd:{x},{y}")]).await?; Ok(())
    }
    async fn mouse_up(&self, x: i32, y: i32) -> Result<()> {
        self.run("cliclick", &[&format!("du:{x},{y}")]).await?; Ok(())
    }

    fn resolution(&self) -> (u32, u32) {
        // sync blocking — ComputerTool::new()에서 초기화 시 1회만 호출.
        // tokio 런타임 시작 전 or tool 등록 시점이라 blocking OK.
        {
            let tmp = match tempfile::Builder::new().prefix("lisa_res_").suffix(".png").tempfile() {
                Ok(t) => t, Err(_) => return (2560, 1600),
            };
            let path = tmp.path().to_string_lossy().to_string();
            let ok = std::process::Command::new("screencapture").args(["-x", &path])
                .output().map(|o| o.status.success()).unwrap_or(false);
            if !ok { return (2560, 1600); }
            if let Ok(output) = std::process::Command::new("sips")
                .args(["-g", "pixelWidth", "-g", "pixelHeight", &path]).output()
            {
                let text = String::from_utf8_lossy(&output.stdout);
                let mut w = 0u32;
                let mut h = 0u32;
                for line in text.lines() {
                    if let Some(val) = line.strip_prefix("  pixelWidth: ") { w = val.trim().parse().unwrap_or(0); }
                    else if let Some(val) = line.strip_prefix("  pixelHeight: ") { h = val.trim().parse().unwrap_or(0); }
                }
                if w > 0 && h > 0 { return (w, h); }
            }
            (2560, 1600)
        }
    }
}
