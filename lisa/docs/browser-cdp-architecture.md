# Browser CDP Direct Backend Architecture

## Overview

The `cdp_direct` backend enables browser automation via Chrome DevTools Protocol (CDP),
using the `chromiumoxide` Rust crate (v0.9). It supports both Linux (Chrome/Chromium)
and webOS TV (WAM browser).

Key design: **chromiumoxide handles CDP communication, our BrowserLauncher handles browser lifecycle.**

```
LLM Agent → BrowserTool (browser.rs) → resolve_backend()
  ├── AgentBrowser  (Node.js CLI — existing)
  ├── RustNative    (fantoccini/WebDriver — existing)
  ├── ComputerUse   (OS-level sidecar — existing)
  └── CdpDirect     (chromiumoxide — this backend)
        ↓
  BrowserLauncher (our code)            chromiumoxide
  ├── ChromeLauncher (Linux)    →      Browser::connect(ws_url)
  └── WamLauncher (webOS TV)   →      Browser::connect(ws_url)
        ↓                                    ↓
  Launch Chrome process              CDP WebSocket communication
  Discover WS endpoint               Page navigation, DOM interaction
  Manage process lifecycle            Screenshot, evaluate JS
```

## Module Structure

```
src/tools/browser_cdp/
├── mod.rs        — CdpBackendState: connection management, action dispatch
├── launcher.rs   — BrowserLauncher trait, ChromeLauncher, WamLauncher
└── snapshot.rs   — DOM snapshot (OpenClaw-style [role] labels for LLM)
```

## Key Design Decisions

### Navigation via JS (not CDP)

```rust
// CDP Page.navigate is detectable by anti-bot systems (Coupang, Naver, etc.)
// JS navigation looks like a user clicking a link.
page.evaluate("window.location.href = 'https://...'")
```

### Single-tab Workflow

Links with `target="_blank"` are rewritten to `target="_self"` before clicking.
All navigation happens in the same tab, avoiding complex multi-tab management
that chromiumoxide doesn't handle well.

### OpenClaw-style Snapshot Format

```
title: 해찬들 맛있는 재래식 된장 | 쿠팡
url: https://www.coupang.com/vp/products/8359528092
elements: 52
---
@e1 [link] "판매자 가입" href="/vendor-signup"
@e2 [input] placeholder="검색" [type=text]
@e3 [heading] "해찬들 맛있는 재래식 된장, 3kg" [level=1]
@e4 [button] "장바구니 담기"
@e5 [button] "바로구매"
@e6 [heading] "함께 구매하면 좋은 상품" [level=2]
@e7 [link] "곰곰 순두부 400g"
```

`[button]` vs `[link]` role labels let the LLM immediately distinguish
action buttons from navigation links. This prevents the LLM from clicking
recommended product links instead of the cart button.

## Connection Flow

```
1. ensure_connection()
   ├── Handler task alive? → if finished, invalidate connection
   ├── Browser exists? → return (already connected)
   ├── Launcher running? → reconnect (Chrome alive, WS dropped)
   └── Fresh launch:
       ├── ChromeLauncher: spawn Chrome → poll /json/version → browser-level ws_url
       └── WamLauncher: luna-send activate → poll /json/version → ws_url
       ↓
2. Browser::connect(ws_url) → (Browser, Handler)
3. tokio::spawn(handler) → background CDP event processing
4. navigator.webdriver override (anti-bot)
5. Store browser + handler_task
```

## Anti-Bot Measures

- **JS navigation** instead of CDP `Page.navigate` — undetectable
- **Minimal Chrome flags** — no `--disable-gpu`, `--disable-sync`, etc.
- **No custom User-Agent** — use Chrome's built-in UA (no version mismatch)
- **`navigator.webdriver` override** — set to `undefined` after each navigation
- **Wayland/X11 auto-detection** — Chrome opens as visible window for user interaction
- **Persistent profile** — cookies/login survive across restarts

## DOM Stabilization

SPA sites (React/Vue) render content dynamically after `readyState='complete'`.
We handle this with:

1. **readyState polling** — wait for `complete` (Rust-side, not JS Promise)
2. **Element count stabilization** — poll `querySelectorAll('*').length` until
   stable for 3 consecutive checks (1.5s initial wait + 300ms intervals)
3. **Snapshot retry** — if snapshot has < 5 refs, wait 2/3/4/5s and retry
   (no page reload — just re-read the same DOM)

## Click Handling

1. Remove `target="_blank"` from element + parent `<a>` tag
2. `scrollIntoView` to bring element into viewport
3. Record URL before click
4. Click element
5. Detect if URL changed (navigation occurred)
6. If navigation: wait for readyState + DOM stabilization
7. Auto-snapshot for LLM

## Dual Platform Support

### Linux (ChromeLauncher)

- Launches Chrome with `--remote-debugging-port={port}`
- Persistent profile: `--user-data-dir=~/.zeroclaw/browser-profile/`
- Headed mode (`headless = false`) — user can see and interact (login, 2FA, CAPTCHA)
- Wayland: auto-sets `WAYLAND_DISPLAY` + `--ozone-platform=wayland`
- X11: auto-sets `DISPLAY=:0`

### webOS TV (WamLauncher)

- Uses `luna-send` to activate WAM app (`com.webos.app.browser`)
- Connects to WAM inspector port (default: 9998)
- No process lifecycle management (WAM is system-managed)

## Configuration

```toml
[browser]
enabled = true
backend = "cdp_direct"
allowed_domains = ["*"]

[browser.cdp_direct]
debug_port = 9222
headless = false
# chrome_path = "/usr/bin/google-chrome"
# user_data_dir = "~/.zeroclaw/browser-profile/"
# wam_inspector_port = 9998
# wam_app_id = "com.webos.app.browser"
# cleanup_stale = false
# timeout_ms = 30000
```

## Feature Gate

`browser-cdp` is included in default features:

```toml
[features]
default = ["observability-prometheus", "channel-nostr", "browser-cdp"]
browser-cdp = ["dep:chromiumoxide"]
```

## Security

URL validation uses the shared `BrowserTool.validate_url()` pipeline:
- Domain allowlist/blocklist
- Private IP blocking (RFC 1918, loopback, link-local)
- SSRF prevention (hex/decimal/octal IP, userinfo stripping)

Screenshot path validation uses `std::fs::canonicalize()` to prevent path traversal.

## Reconnection Strategy

| Scenario | Action |
|---|---|
| WebSocket disconnect, Chrome alive | `Browser::connect()` to rediscovered endpoint |
| Chrome crash | `launcher.launch()` → new process → connect |
| WAM app killed by OS | `luna-send` reactivate → reconnect |

Session persistence: cookies/login stored in `user_data_dir`, survives reconnects.
