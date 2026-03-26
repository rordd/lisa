# Snapshot Control — 스크린샷 기반 앱 제어 시스템

> 3계층 앱 제어의 **L3**. 화면을 캡처하고 Vision LLM이 분석하여 좌표 기반으로 제어한다.
> appMCP(L1)과 CDP(L2)의 보완재로, **아무 앱이나** 제어 가능한 범용 폴백.
>
> Claude Computer Use 아키텍처를 TV 환경에 적용.

---

# 1부. 비교 — OpenClaw / ZeroClaw / Lisa

## 1.1 왜 비교하는가

스크린샷 기반 제어(Computer Use)는 Anthropic이 개척하고, 이미 여러 프로젝트가 구현하고 있다.
어디서 뭘 빌려오고 어디서 차별화할지 알려면 비교가 필수.

## 1.2 아키텍처 비교

| | **Claude Computer Use** | **ZeroClaw computer_use** | **ZeroClaw screenshot** | **Lisa (우리)** |
|---|---|---|---|---|
| 구조 | API 호출자가 루프 구현 | **HTTP 사이드카 패턴** | 독립 tool (캡처만) | **내장 에이전트 루프** |
| 캡처 | Xvfb 스크린샷 | 사이드카에 위임 | 플랫폼별 네이티브 명령 | 플랫폼별 네이티브 명령 |
| 입력 | xdotool / pyautogui | 사이드카에 위임 | — (캡처만) | 플랫폼별 네이티브 명령 |
| 대상 | 데스크톱 (VM/컨테이너) | 데스크톱 | 데스크톱 | **PC + TV (webOS)** |
| 루프 | 외부 (API 호출자) | 외부 | — | **내장** |
| Vision | Claude 전용 | 모델 무관 | 모델 무관 | 모델 무관 |

### ZeroClaw의 2가지 스크린샷 관련 코드

**1) `computer_use` 백엔드** — 브라우저 tool의 하위 백엔드

```
Lisa/ZeroClaw ──HTTP POST──► 사이드카 서버 (localhost:8787)
                              │
                              ├── screen_capture (스크린샷)
                              ├── mouse_move, mouse_click (좌표)
                              ├── mouse_drag
                              ├── key_type, key_press
                              └── open (URL)
```

사이드카 서버는 upstream에 **포함되어 있지 않다**. Anthropic Computer Use 서버 등 외부를 쓰라는 구조.
액션 validate만 ZeroClaw이 하고, 실행은 HTTP로 위임.

**2) `screenshot.rs`** — 독립 tool (캡처만)

```rust
// macOS
"screencapture", "-x", output_path

// Linux (우선순위대로)
"gnome-screenshot" → "scrot" → "import" (ImageMagick)
```

캡처만 하고 base64로 LLM에 리턴. 입력 실행은 없음. `computer_use`와 별개.

## 1.3 플랫폼 비교

| | **Claude Computer Use** | **ZeroClaw** | **Lisa** |
|---|---|---|---|
| macOS | ⚠️ (VM 권장) | ✅ (screencapture) | ✅ |
| Linux | ✅ (Xvfb) | ✅ (scrot/gnome-screenshot) | ✅ |
| **webOS TV** | ❌ | ❌ | **✅** |
| 입력 방식 | 마우스 + 키보드 | 사이드카 위임 | **포인터 + 리모컨 키** |
| 해상도 | 1024×768 (기본) | 설정 가능 | **1920×1080 (4K 가능)** |

## 1.4 Claude Computer Use와의 비교

Claude Computer Use는 Anthropic이 만든 스크린샷 기반 데스크톱 제어 API. Lisa L3의 직접적인 참고 대상.

### 구조 차이

```
[Claude Computer Use]
호출자(개발자) ──API──► Claude API (beta)
    │                    └── tool_use: { action: "left_click", coordinate: [245, 380] }
    ▼
호출자가 직접 실행 + 루프 구현 (while stop_reason == "tool_use")

[Lisa L3]
사용자 ──텔레그램──► Lisa
                      └── 내장 에이전트 루프 (capture → LLM → action → repeat)
```

### 상세 비교

| | **Claude Computer Use** | **Lisa L3** |
|---|---|---|
| **루프 주체** | **호출자가 구현** (API 클라이언트) | **Lisa가 내장** |
| **Tool 정의** | Anthropic 고유 (`computer_20251124`) | 표준 MCP tool (`tv_snapshot`, `tv_input`) |
| **모델 종속** | **Claude 전용** (beta header 필요) | **모델 무관** (Claude, Gemini, 로컬 등) |
| **Tool 형태** | 특수 타입 (API가 스크린샷 자동 요청) | 일반 function tool |
| **보조 tool** | bash, text_editor (내장) | L1(appMCP), L2(CDP) — **3계층 폴백** |
| **대상 환경** | VM/컨테이너 (데스크톱) | **PC + TV (webOS)** |
| **스크린샷 전달** | tool_result에 base64 | tool_result에 base64 (동일) |
| **zoom (영역 확대)** | ✅ (v20251124) | ROI 크롭으로 유사 구현 |
| **프롬프트 인젝션** | 내장 classifier (자동) | 시스템 프롬프트 방어 (수동) |

### Lisa의 구조적 장점

1. **모델 독립** — Claude CU는 Claude 전용. Lisa는 단순 확인은 Gemini Flash($0.005), 복잡한 건 Sonnet으로 자동 전환
2. **3계층 폴백** — Claude CU는 항상 스크린샷. Lisa는 L1(API)→L2(DOM)→L3(스크린샷). 대부분 L1/L2에서 해결되니 L3는 최후 수단
3. **루프 내장** — Claude CU는 호출자가 while 루프 짜야 함. Lisa는 "넷플릭스에서 오징어게임 틀어" 한마디면 끝
4. **TV 네이티브** — Claude CU는 VM 안 데스크톱 가정. Lisa는 매직 리모컨, 가상 키보드, luna-send API 직접 지원

### Claude CU에서 빌려올 설계

1. **좌표 스케일링** — 1024×768로 리사이즈해서 전송. 토큰 절약 + Anthropic이 이 해상도로 학습
2. **zoom 패턴** — 작은 요소 → 영역 확대 → 정확도 향상. Lisa의 ROI 크롭과 동일 개념
3. **프롬프트 인젝션 방어** — 화면 텍스트가 LLM 지시를 오버라이드하는 공격. classifier 또는 격리 프롬프트 필요
4. **에이전트 루프 구조** — [레퍼런스 구현](https://github.com/anthropics/anthropic-quickstarts/tree/main/computer-use-demo) 참고

## 1.5 Lisa의 차별점

```
Claude Computer Use:  Claude 전용. 호출자가 루프 구현. VM 데스크톱 대상.
OpenClaw:             L3 없음. 브라우저 안에서만 동작 (Playwright).
ZeroClaw:             사이드카 패턴으로 외부 위임, 에이전트 루프 없음.
Lisa:                 모델 무관. 루프 내장. PC→TV 이식.
                      L1(appMCP) + L2(CDP) 실패 시 L3 자동 진입.
                      셋 중 유일하게 Vision 에이전트 루프를 내장 구현.
```

---

# 2부. Lisa 동작 방식 상세

## 2.1 왜 필요한가

```
L1 appMCP  → appMCP.json 있는 앱만
L2 CDP     → 웹앱만 (DOM 있는 것)
L3 Snapshot → 아무거나 (네이티브, 시스템 UI, 팝업 전부)
```

- 시스템 UI (설정, 홈 런처) → L1/L2 불가 → **L3만 가능**
- 네이티브 앱 (C++, Qt, Flutter Skia) → DOM 없음 → **L3만 가능**
- 앱 내 예외 (팝업, 에러 대화상자) → appMCP에 핸들러 없을 수 있음 → **L3 폴백**
- appMCP.json 없는 3rd party 앱 → **L3만 가능**

## 2.2 플랫폼 추상화 — ScreenController trait

**PC(Mac/Linux)에서 먼저 구현하고, TV로 이식할 때 최소 변경.** 핵심은 플랫폼별 차이를 trait으로 추상화하는 것.

```rust
trait ScreenController {
    /// 화면 캡처 → PNG bytes
    async fn capture(&self) -> Result<Vec<u8>>;
    
    /// 좌표 클릭
    async fn click(&self, x: i32, y: i32) -> Result<()>;
    
    /// 텍스트 입력
    async fn type_text(&self, text: &str) -> Result<()>;
    
    /// 키 입력 (Enter, Back, Home, 방향키 등)
    async fn press_key(&self, key: &str) -> Result<()>;
    
    /// 드래그
    async fn drag(&self, from: (i32, i32), to: (i32, i32)) -> Result<()>;
    
    /// 스크롤
    async fn scroll(&self, direction: &str, amount: i32) -> Result<()>;
    
    /// 화면 해상도
    fn resolution(&self) -> (u32, u32);
}
```

### 플랫폼별 구현

```
MacScreenController (개발용)
├── capture:    screencapture -x /tmp/snap.png
├── click:      cliclick c:x,y  또는  osascript
├── type_text:  osascript -e 'tell application "System Events" to keystroke'
├── press_key:  osascript
└── 해상도:     system_profiler SPDisplaysDataType

LinuxScreenController (개발용)
├── capture:    scrot /tmp/snap.png  (또는 gnome-screenshot, import)
├── click:      xdotool mousemove x y && xdotool click 1
├── type_text:  xdotool type --delay 50 "text"
├── press_key:  xdotool key Return
└── 해상도:     xdpyinfo | grep dimensions

WebOSScreenController (TV — 이식 대상)
├── capture:    luna-send capture/executeOneShot
├── click:      luna-send ime/injectCursorEvent (매직 리모컨 포인터)
├── type_text:  luna-send ime/insertText
├── press_key:  luna-send ime/injectKeyEvent
└── 해상도:     1920×1080 (고정) 또는 luna-send로 조회
```

**PC→TV 이식 시 변경:** `WebOSScreenController` 구현체 하나 추가. 에이전트 루프, Vision 호출, 좌표 파싱, 최적화 — 전부 공통 코드 그대로.

### ZeroClaw upstream 코드 재활용

`screenshot.rs`의 플랫폼 감지 로직을 `MacScreenController`/`LinuxScreenController`의 `capture()` 구현에 그대로 가져올 수 있다:

```rust
// upstream screenshot.rs에서 가져온 플랫폼 감지
if cfg!(target_os = "macos") {
    // screencapture -x
} else if cfg!(target_os = "linux") {
    // gnome-screenshot → scrot → import (우선순위)
}
```

`computer_use`의 사이드카 프로토콜도 호환 가능하지만, Lisa는 사이드카 없이 직접 실행하는 게 낫다 — HTTP 라운드트립 오버헤드 없이 네이티브 명령 직접 호출.

## 2.3 에이전트 루프

```
사용자: "넷플릭스에서 오징어게임 틀어줘"
  │
  ▼
┌──────────────────────────────────────────────────────┐
│ Loop (최대 20회):                                     │
│                                                      │
│  1. controller.capture() → PNG bytes                 │
│  2. 이미지 리사이즈 (토큰 절약)                         │
│  3. Vision LLM에 전송 (이미지 + 지시 + 이전 행동 기록)  │
│  4. LLM 응답 파싱: { action, coordinate, text, ... }  │
│  5. controller.click(x, y) 또는 type_text() 등 실행   │
│  6. 대기 (화면 전환 시간)                               │
│  7. 목표 달성? → 완료 / 아니면 Loop 반복               │
│                                                      │
│  종료 조건:                                           │
│  - LLM이 "완료" 판단                                  │
│  - 최대 반복 횟수 초과 (안전장치, 기본 20회)             │
│  - 에러 발생                                          │
│  - 사용자 취소                                        │
└──────────────────────────────────────────────────────┘
```

**예시 실행 (7 스텝):**

```
Step 1: 캡처 → 홈 화면        → LLM: click(850, 600) "넷플릭스 아이콘"
Step 2: 캡처 → 넷플릭스 로딩   → LLM: wait(2000) "로딩 중"
Step 3: 캡처 → 넷플릭스 홈     → LLM: click(100, 50) "검색 아이콘"
Step 4: 캡처 → 검색 + 키보드   → LLM: type("오징어게임")
Step 5: 캡처 → 검색 결과       → LLM: click(300, 400) "오징어게임 시즌3"
Step 6: 캡처 → 상세 페이지     → LLM: click(960, 500) "재생 버튼"
Step 7: 캡처 → 재생 시작       → LLM: "완료"
```

## 2.4 LLM Tool 인터페이스

LLM에게는 **2개 tool**이 주어진다:

### tv_snapshot — 화면 캡처

```json
{
  "name": "tv_snapshot",
  "description": "화면을 캡처하여 현재 상태를 확인한다. PNG 이미지를 리턴.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "resize": {
        "type": "object",
        "properties": {
          "width": { "type": "number" },
          "height": { "type": "number" }
        }
      }
    }
  }
}
```

### tv_input — 입력 실행

```json
{
  "name": "tv_input",
  "description": "화면에 입력을 보낸다 (클릭, 키입력, 타이핑, 스크롤)",
  "inputSchema": {
    "type": "object",
    "properties": {
      "action": {
        "type": "string",
        "enum": ["click", "double_click", "right_click", "type", "key",
                 "scroll", "drag", "move", "wait"]
      },
      "coordinate": {
        "type": "array", "items": { "type": "number" },
        "description": "[x, y] 좌표"
      },
      "text": { "type": "string", "description": "type 시 입력 텍스트" },
      "key": {
        "type": "string",
        "description": "HOME, BACK, ENTER, UP, DOWN, LEFT, RIGHT, VOLUMEUP, VOLUMEDOWN 등"
      },
      "duration": { "type": "number", "description": "wait 시 대기 ms" },
      "scroll_direction": { "type": "string", "enum": ["up", "down", "left", "right"] },
      "drag_end": { "type": "array", "items": { "type": "number" } }
    },
    "required": ["action"]
  }
}
```

## 2.5 L1/L2와의 통합

```
사용자 요청
    │
    ▼
  L1 appMCP 도구 있나? ──YES──► appMCP 실행 (~1초)
    │ NO
    ▼
  L2 CDP 가능? (웹앱?) ──YES──► CDP 스냅샷+클릭 (~2초)
    │ NO
    ▼
  L3 Snapshot Control ──────► 캡처+Vision+클릭 (~5초/스텝)
```

L3 진입 후에도:
- 앱이 MCP 서버 등록하면 → L1으로 전환
- 웹앱 DOM 접근 가능해지면 → L2로 전환
- L1/L2 실행 실패 (앱 크래시 등) → L3로 폴백

## 2.6 최적화

### 이미지 리사이즈

```
원본: 1920×1080 → ~1.5MB PNG → ~1,500 토큰
리사이즈: 1280×720 → ~500KB (좌표 ×1.5 보정)
리사이즈: 960×540 → ~250KB (대부분 충분)
```

좌표 보정: `실제좌표 = LLM좌표 × (원본해상도 / 리사이즈해상도)`

### 변경 감지

```
캡처 → 이전 프레임과 pixel diff
  ├── 변경률 < 5% → "아직 로딩 중" (LLM 호출 스킵, 재대기)
  └── 변경률 ≥ 5% → LLM에 전송
```

### ROI (Region of Interest)

```
1회차: 전체 화면 → LLM이 관심 영역 식별
2회차~: 해당 영역만 크롭 → 토큰 절약 + 정확도 향상
```

### 모델 선택

| 상황 | 모델 | 이유 |
|------|------|------|
| 복잡한 UI 분석 | Claude Sonnet / Opus | 정확한 좌표 추론 |
| 단순 상태 확인 | Flash / Haiku | 빠르고 저렴 |
| 텍스트 읽기만 | 로컬 OCR (Tesseract) | 무료, Vision 불필요 |

## 2.7 보안 & 안전장치

| 항목 | 대책 |
|------|------|
| 무한 루프 방지 | 최대 반복 횟수 제한 (기본 20회) |
| 민감 정보 노출 | 스크린샷에 비밀번호 보이면 → LLM에 경고 프롬프트 |
| 오클릭 방지 | 결제/삭제 등 위험 행동 → 사용자 확인 요청 |
| 프롬프트 인젝션 | 화면 내 텍스트가 LLM 지시 오버라이드 가능 → 시스템 프롬프트에 방어 |
| 비용 제어 | 스냅샷당 토큰 비용 추적, 일일 한도 설정 |

---

# 3부. 문제점 및 향후 방향

## 3.1 속도 — L3의 근본적 한계

```
L1 appMCP:  ~1초 (API 한 번)
L2 CDP:     ~2초 (DOM 스냅샷 + LLM)
L3 Snapshot: ~5초/스텝 × 7스텝 = ~35초 (넷플릭스 예시)
```

Vision LLM 호출이 스텝당 ~3초, 캡처+입력이 ~2초. **구조적으로 느리다.**

**완화 방향:**
- L1/L2가 가능하면 항상 먼저 시도 (자동 라우팅)
- 변경 감지로 불필요한 LLM 호출 스킵
- 빠른 모델(Flash/Haiku)로 단순 상태 확인
- ROI 크롭으로 토큰 줄이기

## 3.2 좌표 정확도

Vision LLM의 좌표 추론은 완벽하지 않다. 특히:
- 작은 버튼, 밀집된 UI 요소
- 비슷하게 생긴 반복 요소 (썸네일 그리드)
- 텍스트가 없는 아이콘 전용 버튼

**완화 방향:**
- 클릭 후 상태 변화 없으면 → 좌표 미세 조정 후 재시도
- L2(CDP) 스냅샷으로 보완: 클릭 대상 근처 DOM 정보 참조
- 리사이즈 시 너무 줄이지 않기 (최소 960px 폭)

## 3.3 비용

스크린샷 이미지 토큰이 비싸다.

```
1회 스냅샷: ~1,500 토큰 (이미지) + ~500 토큰 (프롬프트) = ~2,000 토큰
7스텝 작업: ~14,000 토큰 ≈ $0.04 (Sonnet) ~ $0.21 (Opus)
```

**완화 방향:**
- L3는 최후 수단 (L1/L2 우선)
- 단순 확인은 Flash ($0.005/7스텝)
- 변경 감지 + ROI로 실제 전송량 줄이기

## 3.4 TV 입력의 특수성

TV는 마우스가 아니라 **매직 리모컨 포인터**다:

- 포인터가 화면 위에 항상 보이는 건 아님 (숨겨진 상태 가능)
- 포인터 활성화가 필요할 수 있음
- 방향키 네비게이션이 더 자연스러운 UI가 많음 (리스트 포커스)
- 가상 키보드가 화면 하단에 뜸 (좌표 재계산 필요)

**완화 방향:**
- 좌표 클릭 전에 포인터 활성화 명령
- UI에 따라 방향키 네비게이션 전략 자동 선택
- 가상 키보드 감지 시 좌표 오프셋 보정

## 3.5 구현 순서

### Phase 1: PC 캡처 + 분석 (Mac/Linux)
- [ ] `ScreenController` trait 정의
- [ ] `MacScreenController` 구현 (screencapture + cliclick)
- [ ] `LinuxScreenController` 구현 (scrot + xdotool)
- [ ] upstream `screenshot.rs` 플랫폼 감지 로직 재활용
- [ ] Vision LLM에 스크린샷 전송 + 좌표 파싱

### Phase 2: 에이전트 루프 (PC)
- [ ] 캡처→분석→행동→반복 루프 구현
- [ ] 최대 반복 횟수 + 타임아웃 안전장치
- [ ] 이미지 리사이즈 + 좌표 보정
- [ ] 변경 감지 (pixel diff)

### Phase 3: TV 이식
- [ ] `WebOSScreenController` 구현 (luna-send)
- [ ] 매직 리모컨 포인터 제어
- [ ] TV 입력 특수성 처리 (가상 키보드, 포인터 활성화)
- [ ] L1/L2 폴백 연동

### Phase 4: 최적화
- [ ] ROI 크롭
- [ ] 모델 자동 선택 (복잡도에 따라 Sonnet/Flash)
- [ ] 로컬 OCR 보조

---

_v3.0 — 2026-03-27_
_3부 구성: 비교 / 동작 상세 / 문제점 및 방향_
_Claude Computer Use + ZeroClaw upstream 분석 기반_
_ScreenController trait로 PC→TV 이식 최소 변경 설계_
_Project Elvis L3 계층_
