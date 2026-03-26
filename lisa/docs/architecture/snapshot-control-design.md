# Snapshot Control — 스크린샷 기반 앱 제어 시스템

> TV 화면을 캡처하고 Vision LLM이 분석하여 좌표 기반으로 앱을 제어하는 아키텍처.
> appMCP(앱 협조)의 보완재로, appMCP.json이 없는 앱도 제어 가능.
>
> Claude Computer Use 아키텍처를 TV 환경에 적용.

---

## 1. 왜 필요한가

```
appMCP (앱 협조)                    Snapshot Control (앱 비협조)
─────────────────────────────────────────────────────────────
appMCP.json 있는 앱만               아무 앱이나
구조화된 데이터 리턴                 화면에 보이는 것만
빠름 (WS + JSON)                   느림 (캡처 + Vision LLM)
정확함 (API 호출)                   근사적 (좌표 추론)
앱 개발사 협조 필요                  협조 불필요
```

**둘 다 필요:**
- appMCP.json 있으면 → appMCP (빠르고 정확)
- appMCP.json 없으면 → Snapshot Control (느리지만 범용)
- 시스템 UI (설정, 홈) → Snapshot Control
- 앱 내부 예외 상황 → Snapshot Control (팝업, 에러 대화상자 등)

## 2. 아키텍처

```
┌───────────────────────────────────────────────────────────┐
│                       TV 화면                              │
│  ┌─────────────────────────────────────────────────────┐  │
│  │          넷플릭스 (appMCP.json 없는 경우)             │  │
│  │                                                     │  │
│  │   [검색]  [홈]  [NEW]  [내가 찜한 콘텐츠]              │  │
│  │                                                     │  │
│  │   ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐              │  │
│  │   │ 썸네일 │ │ 썸네일 │ │ 썸네일 │ │ 썸네일 │              │  │
│  │   │ 오징어 │ │ 더글  │ │ 블랙미 │ │ 기묘한 │              │  │
│  │   └──────┘ └──────┘ └──────┘ └──────┘              │  │
│  └─────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────┘
         │ 화면 캡처
         ▼
┌─────────────────┐      ┌──────────────────────────────┐
│  Capture Agent   │ ──► │  Vision LLM                   │
│  (프레임버퍼/     │      │  "오징어게임을 찾아서 클릭해"   │
│   HDMI 캡처)     │      │                              │
└─────────────────┘      │  분석 결과:                    │
                         │  click(245, 380)              │
                         └──────────────────────────────┘
                                    │
                                    ▼
                         ┌──────────────────────────────┐
                         │  Input Controller              │
                         │  마우스 포인터 이동 + 클릭       │
                         │  (luna-send / uinput / CEC)    │
                         └──────────────────────────────┘
```

## 3. 핵심 컴포넌트

### 3.1 Capture Agent — 화면 캡처

TV 화면을 이미지로 캡처하는 모듈.

| 방법 | 설명 | 장단점 |
|------|------|--------|
| **프레임버퍼** | `/dev/fb0` 직접 읽기 | 빠름, root 필요, webOS 지원 여부 확인 |
| **luna-send 캡처** | `luna://com.webos.service.capture/executeOneShot` | 공식 API, 안정적 |
| **HDMI 캡처카드** | 외부 장치로 캡처 | 어떤 기기든 가능, 별도 하드웨어 |
| **VNC/RFB** | 화면 스트리밍 프로토콜 | 실시간, 설정 복잡 |

**기본: luna-send 캡처 API 사용**

```sh
# webOS 화면 캡처
luna-send -n 1 -f luna://com.webos.service.capture/executeOneShot '{
  "path": "/tmp/screenshot.png",
  "format": "PNG"
}'
```

### 3.2 Vision Analyzer — 화면 분석

캡처된 이미지를 Vision LLM에 전송하여 UI 요소를 파악하고 행동을 결정.

**Claude Computer Use 방식 (tool 정의):**

```json
{
  "type": "computer_20251124",
  "name": "computer",
  "display_width_px": 1920,
  "display_height_px": 1080,
  "display_number": 1
}
```

**LLM 응답 예시:**

```json
{
  "type": "tool_use",
  "name": "computer",
  "input": {
    "action": "left_click",
    "coordinate": [245, 380]
  }
}
```

### 3.3 Input Controller — 입력 실행

LLM이 결정한 행동을 실제 TV에 실행.

| 액션 | 구현 |
|------|------|
| **좌표 클릭** | luna-send 커서 이동 + 클릭 이벤트 |
| **텍스트 입력** | 가상 키보드 키 입력 시뮬레이션 |
| **스크롤** | 스크롤 이벤트 또는 방향키 반복 |
| **키 입력** | 리모컨 키 시뮬레이션 (HOME, BACK 등) |
| **드래그** | 좌표 시작→끝 드래그 이벤트 |

```sh
# webOS 키 입력 시뮬레이션
luna-send -n 1 luna://com.webos.service.ime/sendEnterKey '{}'

# 커서 이동 (uinput 기반)
# 또는 luna-send로 포인터 제어
```

## 4. 에이전트 루프

Claude Computer Use와 동일한 루프 구조:

```
사용자: "넷플릭스에서 오징어게임 틀어줘"
  │
  ▼
┌──────────────────────────────────────────────────────┐
│ Loop:                                                │
│                                                      │
│  1. 화면 캡처 → screenshot.png                        │
│  2. LLM에 전송 (이미지 + 지시)                         │
│  3. LLM 응답: { action: "left_click", coord: [x,y] } │
│  4. Input Controller로 실행                           │
│  5. 대기 (화면 전환 시간)                               │
│  6. 다시 캡처 → 목표 달성? → 아니면 Loop 반복           │
│                                                      │
│  종료 조건:                                           │
│  - LLM이 "완료" 판단                                  │
│  - 최대 반복 횟수 초과 (안전장치)                       │
│  - 에러 발생                                          │
└──────────────────────────────────────────────────────┘
```

**예시 실행:**

```
Step 1: 캡처 → 홈 화면 보임
        LLM: "넷플릭스 아이콘 보임" → click(850, 600)

Step 2: 캡처 → 넷플릭스 로딩 중
        LLM: "로딩 중, 대기" → wait(2000)

Step 3: 캡처 → 넷플릭스 홈
        LLM: "검색 아이콘 보임" → click(100, 50)

Step 4: 캡처 → 검색 화면 + 가상 키보드
        LLM: "검색창 활성화" → type("오징어게임")

Step 5: 캡처 → 검색 결과
        LLM: "오징어게임 시즌3 썸네일 보임" → click(300, 400)

Step 6: 캡처 → 콘텐츠 상세 페이지
        LLM: "재생 버튼 보임" → click(960, 500)

Step 7: 캡처 → 재생 시작
        LLM: "재생 중. 완료."
```

## 5. Tool 정의 (Lisa용)

### 5.1 snapshot tool

```json
{
  "name": "tv_snapshot",
  "description": "TV 화면을 캡처하여 현재 상태를 확인한다",
  "inputSchema": {
    "type": "object",
    "properties": {
      "resize": {
        "type": "object",
        "description": "리사이즈 (토큰 절약)",
        "properties": {
          "width": { "type": "number" },
          "height": { "type": "number" }
        }
      }
    }
  }
}
```

### 5.2 tv_input tool

```json
{
  "name": "tv_input",
  "description": "TV에 입력을 보낸다 (클릭, 키입력, 타이핑)",
  "inputSchema": {
    "type": "object",
    "properties": {
      "action": {
        "type": "string",
        "enum": ["click", "double_click", "right_click", "type", "key", "scroll", "drag", "move", "wait"]
      },
      "coordinate": {
        "type": "array",
        "items": { "type": "number" },
        "description": "[x, y] 좌표"
      },
      "text": {
        "type": "string",
        "description": "type 액션 시 입력할 텍스트"
      },
      "key": {
        "type": "string",
        "description": "key 액션 시 키 이름 (HOME, BACK, ENTER, UP, DOWN, LEFT, RIGHT, VOLUMEUP, VOLUMEDOWN)"
      },
      "duration": {
        "type": "number",
        "description": "wait 액션 시 대기 시간 (ms)"
      },
      "scroll_direction": {
        "type": "string",
        "enum": ["up", "down", "left", "right"]
      },
      "drag_end": {
        "type": "array",
        "items": { "type": "number" },
        "description": "드래그 끝 좌표 [x, y]"
      }
    },
    "required": ["action"]
  }
}
```

## 6. appMCP와의 통합

```
사용자 요청
    │
    ▼
┌─ Lisa 에이전트 ──────────────────────────────────┐
│                                                  │
│  1. appMCP 도구 있나?                              │
│     ├── YES → appMCP로 실행 (빠름, 정확)           │
│     └── NO  → Snapshot Control 진입               │
│                                                  │
│  2. Snapshot Control 중에도:                       │
│     앱이 MCP 서버 등록하면 → appMCP로 전환          │
│                                                  │
│  3. Fallback:                                    │
│     appMCP 실행 실패 (앱 크래시 등) → 스냅샷으로    │
│     화면 확인 후 복구 시도                          │
└──────────────────────────────────────────────────┘
```

**우선순위:**
1. **appMCP** — 구조화된 API 호출 (가장 빠르고 정확)
2. **Snapshot Control** — 시각적 UI 제어 (범용 폴백)

## 7. 최적화

### 7.1 이미지 크기 줄이기

TV는 1920×1080이지만 Vision LLM에 원본 전송하면 토큰 낭비.

```
원본: 1920×1080 → ~1.5MB PNG
리사이즈: 1280×720 → ~500KB (좌표 비율 보정)
리사이즈: 960×540 → ~250KB (충분한 경우 많음)
```

좌표 보정:
```
LLM 응답: click(640, 360)  // 1280×720 기준
실제 실행: click(960, 540)  // 1920×1080으로 스케일업 (×1.5)
```

### 7.2 변경 감지

매번 전체 화면 캡처 대신, 이전 프레임과 비교하여 변경 여부 확인.

```
캡처 → 이전 프레임과 diff
  ├── 변경 없음 → "아직 로딩 중" (LLM 호출 스킵)
  └── 변경 있음 → LLM에 전송
```

### 7.3 ROI (Region of Interest)

전체 화면 대신 관심 영역만 캡처/전송:

```
전체 화면 한 번 → LLM이 관심 영역 식별
→ 다음부터 해당 영역만 크롭하여 전송
→ 토큰 절약 + 정확도 향상
```

### 7.4 모델 선택

| 상황 | 모델 | 이유 |
|------|------|------|
| 복잡한 UI 분석 | Claude Sonnet / Opus | 정확한 좌표 추론 |
| 단순 상태 확인 | Flash / Haiku | 빠르고 저렴 |
| 텍스트 읽기 | 로컬 OCR (Tesseract) | 무료, 빠름 |

## 8. 보안 & 안전장치

| 항목 | 대책 |
|------|------|
| 무한 루프 방지 | 최대 반복 횟수 제한 (기본: 20) |
| 민감 정보 노출 | 스크린샷에 비밀번호 등 → LLM 전송 전 경고 |
| 오클릭 방지 | 결제/삭제 등 위험 행동 → 사용자 확인 요청 |
| 프롬프트 인젝션 | 화면 내 텍스트가 LLM 지시 오버라이드 가능 → 별도 방어 |
| 비용 제어 | 스냅샷당 토큰 비용 추적, 일일 한도 설정 |

## 9. webOS 구현 상세

### 9.1 화면 캡처

```sh
#!/bin/sh
# capture.sh — webOS 화면 캡처
OUTPUT="/tmp/tv_snapshot_$(date +%s).png"

luna-send -n 1 -f luna://com.webos.service.capture/executeOneShot \
  "{\"path\":\"$OUTPUT\",\"format\":\"PNG\"}"

echo "$OUTPUT"
```

### 9.2 입력 시뮬레이션

```sh
#!/bin/sh
# input.sh — webOS 입력 컨트롤러

ACTION="$1"
X="$2"
Y="$3"

case "$ACTION" in
  click)
    # 커서 이동 + 클릭 (luna-send 또는 uinput)
    luna-send -n 1 luna://com.webos.service.ime/injectCursorEvent \
      "{\"x\":$X,\"y\":$Y,\"type\":\"click\"}"
    ;;
  key)
    KEY="$2"
    luna-send -n 1 luna://com.webos.service.ime/injectKeyEvent \
      "{\"keyname\":\"$KEY\"}"
    ;;
  type)
    TEXT="$2"
    luna-send -n 1 luna://com.webos.service.ime/insertText \
      "{\"text\":\"$TEXT\"}"
    ;;
esac
```

### 9.3 Lisa 스킬로 래핑

```toml
# SKILL.toml
[skill]
name = "tv_snapshot"
description = "TV 화면 캡처 및 시각적 제어"
version = "1.0.0"

[[tools]]
name = "tv_snapshot"
description = "TV 화면을 캡처하여 현재 상태를 확인한다"
command = "sh /var/lisa/skills/snapshot/capture.sh"

[[tools]]
name = "tv_input"
description = "TV에 입력을 보낸다"
command = "sh /var/lisa/skills/snapshot/input.sh {action} {coordinate} {text}"
```

## 10. 비교: Claude Computer Use vs Lisa Snapshot Control

| | Claude Computer Use | Lisa Snapshot Control |
|---|---|---|
| 대상 | 데스크톱 (VM/컨테이너) | **TV (webOS)** |
| 캡처 | Xvfb 스크린샷 | luna-send 캡처 API |
| 입력 | xdotool / pyautogui | **luna-send / uinput** |
| 해상도 | 1024×768 기본 | **1920×1080 (4K 가능)** |
| 입력 방식 | 마우스 + 키보드 | **포인터 + 리모컨 키** |
| 보조 tool | bash, text_editor | **appMCP (구조적 API)** |
| 에이전트 루프 | API 호출자가 구현 | **리사 내장** |

## 11. 구현 계획

### Phase 1: 캡처 + 분석
- [ ] luna-send 기반 화면 캡처 스킬
- [ ] Vision LLM에 스크린샷 전송 + 분석 요청
- [ ] 분석 결과 파싱 (좌표, 텍스트 등)

### Phase 2: 입력 + 루프
- [ ] luna-send 기반 입력 컨트롤러 (클릭, 키, 타이핑)
- [ ] 에이전트 루프 구현 (캡처→분석→행동→반복)
- [ ] 최대 반복 횟수 + 타임아웃 안전장치

### Phase 3: 최적화 + 통합
- [ ] 이미지 리사이즈 + 좌표 보정
- [ ] 변경 감지 (diff 기반 LLM 호출 스킵)
- [ ] appMCP 우선 → Snapshot 폴백 자동 전환
- [ ] ROI 크롭 최적화

---

_v1.0 — 2026-03-26_
_Claude Computer Use 아키텍처 참고_
_appMCP의 보완재 — 앱 비협조 환경 범용 제어_
_Project Elvis 핵심 아키텍처_
