# App Control Architecture — 3계층 앱 제어 통합 설계

> AI 에이전트(Lisa)가 TV/디바이스의 모든 앱을 제어하기 위한 3계층 아키텍처.
> 각 계층은 독립적으로 동작하되, 자동 폴백과 계층 전환을 지원한다.

---

## 1. 3계층 개요

```
┌────────────────────────────────────────────────────────────────┐
│                        사용자 요청                              │
│                  "넷플릭스에서 오징어게임 틀어줘"                  │
└────────────────────────────┬───────────────────────────────────┘
                             │
                             ▼
              ┌─── Lisa App Control Router ───┐
              │                               │
              │   1. appMCP 도구 있나?          │
              │      └→ YES → Layer 1         │
              │                               │
              │   2. 웹앱이고 CDP 가능?         │
              │      └→ YES → Layer 2         │
              │                               │
              │   3. 최후 수단                  │
              │      └→ Layer 3 (Snapshot)     │
              └───────────────────────────────┘
```

| Layer | 이름 | 방식 | 대상 | 속도 | 정확도 | 비용 |
|-------|------|------|------|------|--------|------|
| **L1** | **appMCP** | API 호출 (MCP) | MCP 지원 앱 | ⚡ ~1초 | ✅ 완벽 | 💰 낮음 |
| **L2** | **CDP** | DOM 제어 | 웹앱/웹뷰 | 🔥 ~2초 | ✅ 높음 | 💰 중간 |
| **L3** | **Snapshot** | 화면 캡처 + Vision | 모든 앱 | 🐢 ~5초 | ⚠️ 근사적 | 💰💰 높음 |

## 2. 각 계층 상세

### Layer 1: appMCP (앱 협조)

```
Lisa ──MCP JSON-RPC──► Flutter App (WS)
      ◄─── 결과 리턴 ──┘
```

- **설계 문서**: `appMCP-design.md`
- **프로토콜**: MCP 표준 (JSON-RPC over WebSocket)
- **도구 등록**: appMCP.json 매니페스트 → 핫로딩 → tool 자동 등록
- **앱 실행**: luna-send로 앱 깨우기 → MCP 서버 연결 대기
- **네이밍**: `{app}__{tool}` (예: `netflix__play`)
- **장점**: 구조화된 리턴값, 최소 LLM 토큰, 가장 빠름
- **한계**: 앱 개발사가 appMCP.json 제공해야 함

### Layer 2: CDP Browser Control (DOM 기반)

```
Lisa ──CDP──► WAM/Chromium ──► 웹앱 DOM
      ◄─ 접근성 트리/텍스트 ──┘
```

- **설계 문서**: `cdp-browser-design.md`
- **프로토콜**: Chrome DevTools Protocol (CDP)
- **도구**: `browser` tool (open, snapshot, click, type, ...)
- **핵심**: 접근성 트리 스냅샷 — 텍스트 기반 UI 이해 (이미지 불필요)
- **장점**: 앱 협조 없이 웹앱 구조적 제어, 토큰 효율적
- **한계**: 웹 기술 기반 앱만 가능, 네이티브 앱 불가

### Layer 3: Snapshot Control (시각적)

```
Lisa ──캡처──► 화면 이미지 ──► Vision LLM
      ◄── 좌표/행동 ────────┘
Lisa ──luna-send──► 포인터 클릭/키 입력
```

- **설계 문서**: `snapshot-control-design.md`
- **프로토콜**: 화면 캡처 + Vision LLM + 입력 시뮬레이션
- **도구**: `tv_snapshot` (캡처), `tv_input` (클릭/키/타이핑)
- **에이전트 루프**: 캡처 → 분석 → 행동 → 반복 (최대 20회)
- **장점**: 어떤 앱이든 제어 가능, 앱 협조 불필요
- **한계**: 느림, 비쌈 (Vision LLM), 좌표 정확도 이슈

## 3. 라우팅 로직

### 3.1 자동 계층 선택

```python
def select_layer(request):
    app = identify_app(request)  # LLM이 요청에서 앱 식별
    
    # L1: appMCP 도구 존재?
    if has_appmcp_tools(app):
        return Layer.APPMCP
    
    # L2: 웹앱이고 CDP 접근 가능?
    if is_web_app(app) and cdp_available(app):
        return Layer.CDP
    
    # L3: Snapshot (최후 수단)
    return Layer.SNAPSHOT
```

### 3.2 자동 폴백

```
L1 시도 → 실패 (앱 크래시, MCP 에러)
    │
    ▼
  L2 가능? → YES → CDP로 상태 확인/복구 시도
    │           └→ 실패 → L3
    └→ NO → L3 (화면 캡처로 상황 파악)
```

### 3.3 계층 간 전환

```
시나리오: Netflix 앱 제어

1. appMCP.json 없음 → L3 (Snapshot)으로 시작
2. 화면 캡처 → "Netflix 홈" 확인
3. 검색 입력 → 좌표 클릭 + 타이핑
4. 검색 결과에서 재생 → 좌표 클릭
   → 총 ~15초, Vision LLM 5회 호출

만약 Netflix가 appMCP.json 제공하면:
1. netflix__search("오징어게임") → 결과 JSON
2. netflix__play(contentId) → 재생 시작
   → 총 ~2초, LLM 1회 호출
```

## 4. 앱 유형별 매핑

### webOS TV 앱 분류

| 앱 유형 | 예시 | 최적 계층 | 대체 계층 |
|---------|------|----------|----------|
| **자체 Flutter 앱** | Elvis App | L1 (appMCP) | — |
| **MCP 지원 앱** | 미래의 파트너 앱 | L1 (appMCP) | L2/L3 |
| **웹 기반 앱** | 일부 TV 앱, 웹 브라우저 | L2 (CDP) | L3 |
| **네이티브 앱** | Netflix, YouTube | L3 (Snapshot) | — |
| **시스템 UI** | 설정, 홈 런처 | L3 (Snapshot) | — |
| **하이브리드 앱** | 웹뷰 내장 네이티브 | L2 (웹뷰) + L3 (네이티브) | — |

### 제어 능력 비교

```
                    appMCP     CDP        Snapshot
                    ──────     ───        ────────
검색              ✅ API      ✅ DOM      ✅ 시각적
재생/정지         ✅ API      ⚠️ JS      ✅ 버튼 클릭
볼륨 조절         ✅ API      ❌         ✅ 슬라이더
채널 변경         ✅ API      ❌         ✅ 리모컨 키
텍스트 입력       ✅ API      ✅ DOM      ✅ 가상 키보드
데이터 추출       ✅ 구조화    ✅ DOM     ⚠️ OCR
상태 확인         ✅ 리턴값    ✅ DOM     ⚠️ 화면 분석
앱 간 전환        ❌          ❌         ✅ 홈 버튼
```

## 5. LLM Tool 통합

### 5.1 도구 목록 (LLM이 보는 것)

```
── appMCP 도구 (L1) ──
netflix__search      - Netflix 콘텐츠 검색
netflix__play        - Netflix 콘텐츠 재생
netflix__top10       - Netflix TOP 10
youtube__search      - YouTube 검색
youtube__play        - YouTube 재생
melon__play          - Melon 음악 재생
tv__setVolume        - TV 볼륨 조절
tv__channelChange    - TV 채널 변경

── browser 도구 (L2) ──
browser              - 웹앱 DOM 기반 제어 (open/snapshot/click/type)

── snapshot 도구 (L3) ──
tv_snapshot          - TV 화면 캡처
tv_input             - TV 입력 (클릭/키/타이핑)

── 공통 도구 ──
weather              - 날씨 조회
memory_search        - 기억 검색
web_fetch            - URL 내용 가져오기
```

### 5.2 Dynamic Tool Pruning 연계

앱 50개 × 도구 3개 = 150개 L1 도구 → 프루닝 필수.

```
요청: "넷플릭스에서 오징어게임 틀어줘"
  → 활성화: netflix__*, tv__setVolume, tv_snapshot, tv_input
  → 비활성화: youtube__*, melon__*, weather, ...

요청: "내일 날씨 어때?"
  → 활성화: weather
  → 비활성화: netflix__*, browser, tv_snapshot, ...
```

카테고리 기반 자동 그룹:
```toml
[[tool_filter_groups]]
name = "entertainment"
mode = "on_demand"
patterns = ["netflix__*", "youtube__*", "melon__*"]

[[tool_filter_groups]]
name = "tv_control"
mode = "on_demand"
patterns = ["tv__*"]

[[tool_filter_groups]]
name = "visual_control"
mode = "on_demand"
patterns = ["tv_snapshot", "tv_input", "browser"]
```

## 6. 아키텍처 다이어그램

```
┌──────────────────────────────────────────────────────────────┐
│                        Lisa (에이전트)                         │
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                    Tool Registry                        │ │
│  │                                                         │ │
│  │  ┌─────────────┐ ┌──────────────┐ ┌─────────────────┐  │ │
│  │  │ L1: appMCP   │ │ L2: browser   │ │ L3: snapshot     │  │ │
│  │  │ Tools        │ │ Tool          │ │ Tools            │  │ │
│  │  │              │ │               │ │                  │  │ │
│  │  │ netflix__*   │ │ open          │ │ tv_snapshot      │  │ │
│  │  │ youtube__*   │ │ snapshot      │ │ tv_input         │  │ │
│  │  │ melon__*     │ │ click         │ │                  │  │ │
│  │  │ tv__*        │ │ type          │ │                  │  │ │
│  │  └──────┬──────┘ └──────┬───────┘ └────────┬────────┘  │ │
│  └─────────┼───────────────┼──────────────────┼────────────┘ │
│            │               │                  │              │
└────────────┼───────────────┼──────────────────┼──────────────┘
             │               │                  │
             ▼               ▼                  ▼
     ┌──────────────┐ ┌────────────┐  ┌─────────────────────┐
     │ MCP Server    │ │ CDP        │  │ Capture + Input      │
     │ (WS)          │ │ (DevTools) │  │ (luna-send)          │
     │               │ │            │  │                     │
     │ Flutter App   │ │ WAM/       │  │ 화면 캡처 +          │
     │ 또는           │ │ Chromium   │  │ 포인터/키 시뮬레이션   │
     │ 파트너 앱      │ │            │  │                     │
     └──────────────┘ └────────────┘  └─────────────────────┘
```

## 7. 통합 시나리오

### 시나리오 1: "넷플릭스에서 오징어게임 틀어줘" (L1 또는 L3)

**appMCP 있을 때 (L1):**
```
LLM → netflix__search({query: "오징어게임"})
    → [{contentId: "abc", title: "오징어게임 시즌3"}]
LLM → netflix__play({contentId: "abc"})
    → {playing: true}
LLM → "오징어게임 시즌3 재생할게!"
총: ~2초, LLM 호출 1회
```

**appMCP 없을 때 (L3 → Snapshot):**
```
LLM → tv_snapshot() → [홈 화면 이미지]
LLM → tv_input({action: "click", coordinate: [850, 600]})  // Netflix 아이콘
LLM → tv_snapshot() → [Netflix 홈 이미지]
LLM → tv_input({action: "click", coordinate: [100, 50]})   // 검색
LLM → tv_input({action: "type", text: "오징어게임"})
LLM → tv_snapshot() → [검색 결과 이미지]
LLM → tv_input({action: "click", coordinate: [300, 400]})  // 오징어게임
LLM → tv_snapshot() → [상세 페이지 이미지]
LLM → tv_input({action: "click", coordinate: [960, 500]})  // 재생
LLM → "오징어게임 시즌3 재생할게!"
총: ~15초, LLM 호출 5회 (Vision)
```

### 시나리오 2: "웹 브라우저에서 뉴스 검색해줘" (L2)

```
LLM → browser({action: "open", url: "https://news.google.com"})
LLM → browser({action: "snapshot"})
    → @e1 [search] 뉴스 검색...
LLM → browser({action: "click", selector: "@e1"})
LLM → browser({action: "type", selector: "@e1", text: "AI 뉴스"})
LLM → browser({action: "press", key: "Enter"})
LLM → browser({action: "snapshot"})
    → @e5 [article] "AI가 바꾸는 2026년"
LLM → "첫번째 뉴스: AI가 바꾸는 2026년"
총: ~3초, 텍스트 기반 (이미지 불필요)
```

### 시나리오 3: "볼륨 줄이고 채널 KBS로 바꿔" (L1)

```
LLM → tv__setVolume({level: 20})
    → {level: 20, muted: false}
LLM → tv__channelChange({name: "KBS"})
    → {channel: 7, name: "KBS1"}
LLM → "볼륨 20으로 줄이고 KBS1로 바꿨어"
총: ~1초, 단순 API
```

### 시나리오 4: L1 실패 → L3 폴백

```
LLM → netflix__play({contentId: "abc"})
    → ERROR: "앱이 응답하지 않습니다"
LLM → tv_snapshot() → [앱 크래시 화면 이미지]
LLM → "넷플릭스가 멈춘 것 같아. 다시 실행할까?"
    → 사용자 확인
LLM → tv_input({action: "key", key: "HOME"})
    → 홈으로 이동 → 넷플릭스 재실행 → L1 재시도
```

## 8. 구현 우선순위

```
Phase 0 (현재): Lisa 코어 안정화
    └── 기본 스킬 (weather, calendar, news)

Phase 1: L1 — appMCP
    ├── appMCP.json 매니페스트 로더
    ├── MCP WS transport 추가
    ├── 자체 Flutter 앱 MCP 서버 구현
    └── tv__setVolume, tv__channelChange 등 기본 함수

Phase 2: L2 — CDP Browser
    ├── browser.enabled = true
    ├── PR #85 기반 CDP backend 통합
    ├── WamLauncher webOS 연동
    └── 접근성 스냅샷 최적화

Phase 3: L3 — Snapshot Control
    ├── luna-send 화면 캡처 스킬
    ├── Vision LLM 연동
    ├── 입력 컨트롤러 (포인터 + 키)
    └── 에이전트 루프 + 안전장치

Phase 4: 통합 + 라우팅
    ├── 자동 계층 선택 로직
    ├── 폴백 체인 (L1 → L2 → L3)
    ├── Dynamic Tool Pruning 연계
    └── 성능 모니터링 + 최적화
```

## 9. 설계 문서 목록

| 문서 | 계층 | 파일 |
|------|------|------|
| appMCP 설계 | L1 | `appMCP-design.md` |
| CDP Browser 설계 | L2 | `cdp-browser-design.md` |
| Snapshot Control 설계 | L3 | `snapshot-control-design.md` |
| **통합 아키텍처 (본 문서)** | 전체 | `app-control-architecture.md` |

---

_v1.0 — 2026-03-26_
_3계층 앱 제어: appMCP + CDP + Snapshot_
_Project Elvis 핵심 아키텍처_
