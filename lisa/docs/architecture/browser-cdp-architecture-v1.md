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
└── snapshot.rs   — DOM snapshot (role-based category grouping for LLM)
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
`window.open()` is overridden to navigate in the current tab instead of opening new tabs.
All navigation happens in the same tab, avoiding complex multi-tab management
that chromiumoxide doesn't handle well.

### Role-based Category Snapshot Format

Elements are grouped by category so the LLM can immediately find action
buttons without scanning hundreds of elements:

```
title: 해찬들 맛있는 재래식 된장 | 쿠팡
url: https://www.coupang.com/vp/products/8359528092
elements: 52
---
── text ──
@e3 [h1] "해찬들 맛있는 재래식 된장, 3kg"
@e15 [h2] "상품 정보"
@e30 [h2] "추천 상품"
── buttons ──
@e4 "장바구니 담기"
@e5 "바로구매" [disabled]
── inputs ──
@e2 [type=text] placeholder="검색"
@e11 [type=number] value="1" [readonly]
── links ──
@e1 "판매자 가입"
@e7 "곰곰 순두부 400g" [추천 상품]
── other ──
@e20 [alert] "로그인이 필요합니다"
@e25 [tab] "상품정보"
```

**Categories:**
- `text` — headings (h1-h6) for page structure. Always included even in interactive-only mode.
- `buttons` — action buttons (`<button>`, `input[submit]`, `role="button"`).
- `inputs` — form fields (`<input>`, `<select>`, `<textarea>`).
- `links` — navigation links (`<a>`, `role="link"`). Includes `[section]` context from nearest heading.
- `other` — alerts (`role="alert"`/`role="status"`), tabs, misc interactive elements.

**Element state:** `[disabled]`, `[checked]`, `[readonly]`, `[selected]` — prevents LLM from clicking inactive elements.

**Link context:** Each link shows its nearest heading text in `[brackets]`, so the LLM can distinguish
"추천 상품" links from "검색 결과" links without site-specific logic.

**Interactive detection:** Elements are considered interactive if they match:
1. Standard tags: `a`, `button`, `input`, `select`, `textarea`, `summary`
2. ARIA: `[role]`, `[tabindex]`
3. JS handler: `el.onclick`
4. CSS: `cursor: pointer` — catches role-less `<div>` click targets (~85% coverage)

Category grouping works on ALL websites (uses HTML tag types and standard attributes).

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
4. Select active page (retry once if chromiumoxide hasn't discovered targets yet)
5. Navigate chrome:// → about:blank (anti-bot: avoid chrome:// origin headers)
6. Apply overrides: navigator.webdriver + window.open
7. Store browser + handler_task + page
```

## Anti-Bot Measures

- **JS navigation** instead of CDP `Page.navigate` — undetectable
- **Minimal Chrome flags** — no `--disable-gpu`, `--disable-sync`, etc.
- **No custom User-Agent** — use Chrome's built-in UA (no version mismatch)
- **`navigator.webdriver` override** — set to `undefined` after each navigation
- **`window.open` override** — redirects new tab opens to current tab navigation
- **Page caching** — `active_page()` returns cached page, avoids CDP `Target.getTargets` calls during browsing
- **`chrome://` → `about:blank`** — navigating from `chrome://newtab` triggers bot detection; initial page is moved to `about:blank` first
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
6. If navigation: re-apply overrides (webdriver + window.open) + wait for readyState + DOM stabilization
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
