# Snapshot Control — 스크린샷 기반 화면 제어 시스템

> 화면을 캡처하고 Vision LLM이 분석하여 좌표 기반으로 제어한다.
> Anthropic Computer Use API 호환. 아무 앱이나 제어 가능한 범용 시스템.

---

# 1부. 구현 현황

## 1.1 아키텍처

```
사용자 (텔레그램)
    ↓ "쿠팡에서 두부 장바구니 담아줘"
Lisa (ZeroClaw fork)
    ↓ screen-agent 스킬 로드 (always=true)
Claude Sonnet 4.6 (Anthropic API)
    ↓ computer tool 호출
ComputerTool (tool.rs)
    ↓ 좌표 변환 (이미지→화면)
MacScreenController (mac.rs)
    ↓ 실제 OS 명령 실행
macOS 화면
```

## 1.2 3계층 구조

```
┌─────────────────────────────────────────────┐
│  Layer 1: LLM Tool Interface (tool.rs)      │
│  - Anthropic Computer Use API 호환           │
│  - 단일 "computer" tool (computer_20251124)  │
│  - 좌표 자동 변환 (ScaleHandle)              │
│  - action 후 자동 screenshot 첨부            │
├─────────────────────────────────────────────┤
│  Layer 2: ScreenController Trait (mod.rs)    │
│  - 플랫폼 독립 인터페이스                     │
│  - capture, click, type, scroll, key 등      │
│  - 구현체: MacScreenController (완료)        │
│  - 미래: LinuxScreenController, WebOS        │
├─────────────────────────────────────────────┤
│  Layer 3: OS Native (mac.rs)                │
│  - screencapture + sips (캡처/리사이즈)      │
│  - cliclick (마우스 제어)                     │
│  - cgevent 프리컴파일 바이너리 (키보드/스크롤) │
│  - pbcopy + Cmd+V (한글 텍스트 입력)         │
└─────────────────────────────────────────────┘
```

## 1.3 ScreenController Trait

```rust
#[async_trait]
trait ScreenController: Send + Sync {
    async fn capture(&self, resize_width: Option<u32>) -> Result<CaptureResult>;
    async fn click(&self, x: i32, y: i32) -> Result<()>;
    async fn double_click(&self, x: i32, y: i32) -> Result<()>;
    async fn right_click(&self, x: i32, y: i32) -> Result<()>;
    async fn triple_click(&self, x: i32, y: i32) -> Result<()>;
    async fn type_text(&self, text: &str) -> Result<()>;
    async fn press_key(&self, key: &str) -> Result<()>;
    async fn scroll(&self, direction: &str, amount: u32) -> Result<()>;
    async fn drag(&self, from: (i32, i32), to: (i32, i32)) -> Result<()>;
    async fn move_cursor(&self, x: i32, y: i32) -> Result<()>;
    async fn cursor_position(&self) -> Result<(i32, i32)>;
    async fn mouse_down(&self, x: i32, y: i32) -> Result<()>;
    async fn mouse_up(&self, x: i32, y: i32) -> Result<()>;
    async fn hold_key(&self, key: &str, duration_secs: f64) -> Result<()>;
    fn resolution(&self) -> (u32, u32);
}
```

## 1.4 MacScreenController 구현

| Action | 구현 | 속도 |
|--------|------|------|
| screenshot | `screencapture -x` → `sips resize` → JPEG 40% | ~200ms |
| click/double/right/triple | `cliclick c/dc/rc/tc` | ~70ms |
| mouse_down/up | `cliclick dd/du` | ~70ms |
| move/drag | `cliclick m/dd+du` | ~70ms |
| key (특수키/콤보) | `cgevent key <code> [flags]` (프리컴파일) | ~15ms |
| key (ASCII) | `cgevent unicode <charcode>` | ~15ms |
| scroll | `cgevent scroll <w1> <w2>` | ~15ms |
| type (텍스트) | `pbcopy` + `osascript Cmd+V` (IME/한글) | ~120ms |
| hold_key | `cgevent keydown` → sleep → `cgevent keyup` | duration+30ms |
| cursor_position | `cliclick p` → 파싱 | ~70ms |

### cgevent 프리컴파일 바이너리

`~/.zeroclaw/bin/cgevent` — Swift CGEvent API를 swiftc -O로 미리 컴파일.
`swift -e` 매번 실행 시 ~150ms → 프리컴파일 ~15ms (10배 향상).

지원 커맨드: `key`, `keydown`, `keyup`, `unicode`, `scroll`

### 한글 입력

macOS IME를 우회하기 위해 클립보드 방식 사용:
1. `pbcopy`로 텍스트를 클립보드에 복사
2. `osascript`로 Cmd+V 실행
3. ⚠️ 클립보드 덮어쓰기 부작용 있음

## 1.5 ComputerTool — Anthropic Computer Use 호환

### Tool 등록

- tool type: `computer_20251124`
- `display_width_px` / `display_height_px` 자동 설정
- `screen_control.enabled = true` → computer tool 등록, screenshot tool 비활성화 (exclusive)

### 지원 Action

| Action | 파라미터 |
|--------|---------|
| screenshot | — |
| cursor_position | — |
| left_click / right_click / middle_click | coordinate |
| double_click / triple_click | coordinate |
| mouse_move | coordinate |
| left_click_drag | start_coordinate, coordinate |
| left_mouse_down / left_mouse_up | coordinate |
| type | text |
| key | text (Return, Escape, cmd+c 등) |
| hold_key | text, duration |
| scroll | coordinate, scroll_direction, scroll_amount |
| wait | duration |

### 좌표 자동 변환

LLM은 리사이즈된 이미지 좌표를 전달 → `ScaleHandle`이 실제 화면 좌표로 자동 변환.

```
LLM: coordinate [384, 240] (768×480 이미지 기준)
  ↓ scale_x=3.33, scale_y=3.33
실제: (1280, 800) (2560×1600 화면)
```

scale 미설정(0.0) 시 좌표 그대로 반환 (방어 코드).

### 자동 Screenshot

click, type, key, scroll, drag 실행 후:
1. `screenshot_delay_ms` 대기 (기본 2000ms, config 변경 가능)
2. 자동 screenshot 캡처
3. tool result에 이미지 포함

mouse_move, wait, cursor_position은 자동 캡처 안 함.

## 1.6 Anthropic Provider 연동

### anthropic.rs 변경사항

- `NativeToolDef` enum: `Regular(NativeToolSpec)` | `ComputerUse(ComputerUseToolSpec)`
- `apply_auth()`: computer tool 있으면 `computer-use-2025-11-24` beta 헤더 추가
- `convert_tools()`: `computer` tool name 감지 → `computer_20251124` 타입 자동 변환
- `ToolResultContent`: 텍스트+이미지 혼합 블록 지원
- `token-efficient-tools-2025-02-19` beta 항상 적용

### Setup Token Auth (sk-ant-oat01-*)

`chat_with_system()`에서 `system`을 blocks array `[{"type":"text","text":"..."}]`로 전송 (plain string 아님).

Beta 헤더: `claude-code-20250219,oauth-2025-04-20,fine-grained-tool-streaming-2025-05-14,token-efficient-tools-2025-02-19[,computer-use-2025-11-24]`

## 1.7 Prompt Caching

- `cache_control: {"type": "ephemeral"}` on tools + messages
- 실측: 97-99% cache hit rate
- cache_read: 19k→28k (턴마다 ~600 증가)
- cache_creation: ~600/턴 (새 screenshot 1장)
- **비용 90% 절감** (multi-turn)

## 1.8 Config

```toml
[screen_control]
enabled = false              # true → computer tool, false → screenshot tool
backend = "mac"              # "mac" or "linux"
resize_width = 768           # 캡처 이미지 리사이즈 폭 (0=원본)
screenshot_delay_ms = 2000   # action 후 자동 screenshot 전 대기 (ms)
```

## 1.9 screen-agent 스킬

`~/.zeroclaw/workspace/skills/screen-agent/SKILL.toml` — `always: true`

시스템 프롬프트에 자동 포함:
- macOS 키보드 단축키 (cmd+r, cmd+l 등)
- capture→judge→act→verify 루프
- 스크롤 전략 (amount 10+, 안 보이면 즉시 스크롤)
- 주소창 입력 패턴 (cmd+l → cmd+a → type URL)

Linux용 프리셋: `SKILL.toml.linux` (cmd→ctrl 전환)

---

# 2부. 비교

## 2.1 아키텍처 비교

| | **Claude Computer Use** | **ZeroClaw upstream** | **Lisa (현재)** |
|---|---|---|---|
| 구조 | API 호출자가 루프 구현 | HTTP 사이드카 패턴 | **내장 에이전트 루프** |
| 캡처 | Xvfb 스크린샷 | 사이드카에 위임 | 플랫폼별 네이티브 |
| 입력 | xdotool / pyautogui | 사이드카에 위임 | 플랫폼별 네이티브 |
| Tool 형태 | 특수 `computer_20251124` 타입 | 일반 HTTP 위임 | **Anthropic 호환 네이티브** |
| Vision | Claude 전용 | 모델 무관 | **Claude 최적화 (캐시)** |
| 대상 | VM/컨테이너 | 데스크톱 | **PC + TV (webOS 계획)** |

## 2.2 Lisa의 차별점

1. **Anthropic Computer Use API 네이티브 호환** — 사이드카 없이 직접 구현
2. **프리컴파일 CGEvent** — swift -e 대비 10배 빠른 키/스크롤
3. **한글 지원** — pbcopy+Cmd+V로 IME 우회
4. **Prompt caching 97%+** — multi-turn 비용 90% 절감
5. **플랫폼 추상화** — trait만 구현하면 Linux/webOS 이식 가능

---

# 3부. 플랫폼 이식 계획

## 3.1 구현체 전략 — 커널 인터페이스 통일

macOS만 별도, Linux와 webOS는 **커널 인터페이스 2개**(`/dev/fb0` + `/dev/uinput`)로 통일:

```
┌─────────────────────────────────────────────────┐
│  MacScreenController (macOS 전용)               │
│  ├── capture: screencapture + sips              │
│  └── input:   cliclick + cgevent (프리컴파일)    │
├─────────────────────────────────────────────────┤
│  LinuxScreenController (Linux + webOS 통합)     │
│  ├── capture: /dev/fb0 → raw → JPEG 변환        │
│  └── input:   /dev/uinput (가상 입력 장치)       │
└─────────────────────────────────────────────────┘
```

### 왜 커널 인터페이스인가

| 접근 방식 | X11 | Wayland | webOS | TTY |
|-----------|-----|---------|-------|-----|
| xdotool + scrot | ✅ | ❌ | ❌ | ❌ |
| ydotool + grim | ❌ | ✅ | ❌ | ❌ |
| luna-send | ❌ | ❌ | ✅ | ❌ |
| **/dev/uinput + /dev/fb0** | **✅** | **✅** | **✅** | **✅** |

디스플레이 서버에 의존하지 않으므로 **어떤 Linux 환경에서든 동작**.
권한만 있으면 됨 (`input` 그룹 또는 root).

### /dev/uinput — 입력

커널 레벨 가상 입력 장치. 마우스, 키보드 이벤트를 직접 주입:
- 마우스 이동/클릭: `EV_REL` / `EV_ABS` + `EV_KEY`
- 키보드: `EV_KEY` (keycode 기반)
- 스크롤: `EV_REL` + `REL_WHEEL`
- X11, Wayland, TTY, webOS 전부 동작
- ydotool이 내부적으로 uinput 사용하는 구조

### /dev/fb0 — 캡처

리눅스 framebuffer 직접 읽기:
- raw RGBX 데이터 → JPEG/PNG 인코딩
- 해상도는 `/sys/class/graphics/fb0/virtual_size` 에서 조회
- GPU 컴포지팅 환경에서도 webOS TV는 fb0에 최종 합성 화면 출력 (확인됨)

### 구현 예상

```rust
struct LinuxScreenController {
    uinput_fd: File,        // /dev/uinput
    fb_path: String,        // /dev/fb0
    width: u32,
    height: u32,
}

impl ScreenController for LinuxScreenController {
    async fn capture(&self, resize_width: Option<u32>) -> Result<CaptureResult> {
        // /dev/fb0 → raw read → JPEG encode → resize
    }
    async fn click(&self, x: i32, y: i32) -> Result<()> {
        // uinput: EV_ABS(x,y) → EV_KEY(BTN_LEFT) → EV_SYN
    }
    async fn type_text(&self, text: &str) -> Result<()> {
        // uinput: char → keycode 변환 → EV_KEY
    }
    // ...
}
```

## 3.2 Docker 테스트 환경

macOS에서 Linux 환경 테스트:

```dockerfile
FROM ubuntu:24.04
RUN apt-get install -y xvfb firefox-esr
# /dev/uinput + /dev/fb0 사용
```

```bash
docker run --device /dev/uinput -e DISPLAY=:99 lisa-test
```

## 3.3 webOS TV 특수 고려사항

uinput + fb0 기반이지만 TV 고유 사항 존재:
- 매직 리모컨 포인터 활성화가 필요할 수 있음 (uinput으로 커서 이벤트 시 자동 활성화 여부 확인 필요)
- 방향키 네비게이션이 자연스러운 UI가 많음 (리스트 포커스)
- 가상 키보드 감지 시 좌표 오프셋 보정
- 해상도: 1920×1080 고정 (4K TV도 UI는 1080p)
- luna-send는 **fallback**으로 유지 — uinput 안 되는 특수 케이스 대응

## 3.4 Config

```toml
# macOS
[screen_control]
backend = "mac"

# Linux (PC, Docker, webOS TV 전부)
[screen_control]
backend = "linux"
```

SKILL.toml은 OS별로 교체: `cp SKILL.toml.linux SKILL.toml`

---

# 4부. 모델 테스트 결과

| 모델 | Computer Use 지원 | 속도 | 정확도 | 비용 | 결론 |
|------|---|---|---|---|---|
| Claude Haiku 4.5 | ❌ (400 에러) | — | — | — | 사용 불가 |
| Claude Sonnet 4.6 | ✅ | 5-10s/턴 | 양호 | $3/MTok in | **현재 사용** |
| Claude Opus 4.6 | ✅ | 15-30s/턴 | 양호 | $15/MTok in | 효과 없음 (속도 대비) |

**결론:** Sonnet 4.6이 가성비 최적. Opus는 삽질 감소 효과 미미.

---

# 5부. 성능 프로파일

## Action당 소요 시간

| 구간 | 시간 |
|------|------|
| cgevent (key/scroll) | ~15ms |
| cliclick (click) | ~70ms |
| screencapture + sips | ~200ms |
| screenshot delay | 2000ms (config) |
| **LLM 응답** | **5,000-30,000ms (90%)** |

**병목: LLM 응답 시간.** 로컬 최적화로는 한계. 캐싱으로 비용은 줄이되 속도는 모델 의존.

## Prompt Caching 효과

| 턴 | input | cache_read | cache_creation | hit rate |
|---|---|---|---|---|
| 1 | 1 | 19,676 | 103 | 99.5% |
| 10 | 1 | 23,055 | 595 | 97.5% |
| 20 | 1 | 28,182 | 625 | 97.8% |

---

_v4.0 — 2026-03-28_
_구현 완료 기준 갱신. PR #89 머지._
_설계 문서 → 구현 문서로 전환._
