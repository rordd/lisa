# CDP Browser Control — 웹앱 DOM 기반 제어

> 3계층 앱 제어의 **L2**. 웹 기술로 만든 앱을 DOM으로 직접 제어한다.
> appMCP(L1)이 없는 웹앱에 대한 구조적 제어 수단.

---

# 1부. 비교 — OpenClaw / ZeroClaw / Lisa

## 1.1 왜 3가지를 비교하는가

같은 문제(AI가 브라우저/웹앱을 제어)를 세 프로젝트가 다르게 풀고 있다.
어디서 뭘 빌려오고, 어디서 차별화할지 알려면 비교가 필수.

## 1.2 아키텍처 비교

ZeroClaw upstream은 **4개 백엔드**를 지원한다. 스냅샷 방식이 완전히 다르므로 주요 2개를 분리 비교한다.

| | **OpenClaw** | **ZeroClaw agent_browser** (기본) | **ZeroClaw rust_native** (옵션) | **Lisa (우리)** |
|---|---|---|---|---|
| 언어 | Node.js | Rust CLI 외부호출 | Rust (feature flag) | Rust |
| 브라우저 엔진 | **Playwright** (내장) | **agent-browser** (Vercel, Rust CLI) | **fantoccini** (WebDriver) | **chromiumoxide** (CDP 직접) |
| 프로토콜 | Playwright API → CDP | CLI stdout/json | WebDriver Protocol | **CDP WebSocket 직접** |
| 의존성 | Playwright + Chromium 번들 | agent-browser CLI + Chrome for Testing | chromedriver + Chrome | chromiumoxide crate만 |
| 바이너리 추가 | ~200MB (Chromium) | ~50MB (CLI + Chrome) | chromedriver 바이너리 | **0** (시스템 Chrome/WAM 사용) |
| feature flag | — | — | `browser-native` | `browser-cdp` |

> **ZeroClaw config:** `[browser] backend = "agent_browser"` (기본) 또는 `"rust_native"` (옵션).
> `compact`/`interactive_only`는 config에 없고, LLM이 `snapshot` 호출 시 파라미터로 전달 (기본값 둘 다 `true`).

**핵심 차이:** OpenClaw과 ZeroClaw은 브라우저를 직접 들고 다닌다. Lisa는 **이미 있는 브라우저에 연결**한다. TV에는 이미 WAM(Chromium)이 있으니까.

## 1.3 스냅샷 방식 비교

LLM이 화면을 "이해"하는 방법. 여기서 가장 큰 차이가 난다.

| | **OpenClaw** | **ZeroClaw agent_browser** | **ZeroClaw rust_native** | **Lisa** |
|---|---|---|---|---|
| 데이터 소스 | **접근성 트리 (AX Tree)** | **접근성 트리 (AX Tree)** | **DOM 직접 순회 JS** | **DOM 직접 순회 JS** |
| API | `_snapshotForAI()` | `agent-browser snapshot` CLI | `snapshot_script()` JS evaluate | `snapshot_script()` JS evaluate |
| 필터링 | 17개 INTERACTIVE_ROLES | 접근성 트리 role 기반 | 6개 태그 + `el.onclick` | 6개 태그 + `el.onclick` |
| 포맷 | 들여쓰기 트리 + `[ref=e1]` | 접근성 트리 + `@ref` | 카테고리 그룹핑 | 카테고리 그룹핑 |
| compact 모드 | ✅ (구조적 노드 제거) | ✅ (`-c` 플래그) | ✅ (텍스트 없는 비인터랙티브 제거) | ✅ |

**OpenClaw과 agent_browser가 정확한 이유:** 브라우저의 접근성 엔진을 직접 쓴다. OS 스크린리더가 쓰는 것과 같은 데이터. 브라우저가 이미 계산한 role을 가져오니까, 개발자가 `<div onClick>` 써도 브라우저가 role을 추론해서 잡힌다.

**rust_native/Lisa(현재)의 한계:** DOM을 직접 걸어다니면서 태그 이름으로 판단. `addEventListener`로 등록된 이벤트, React/Vue synthetic event(root에 위임)는 못 잡는다. Lisa의 `snapshot_script()`는 upstream `rust_native` 백엔드에서 가져온 것으로, **동일한 코드**(`isInteractive` 로직, 400개 상한, `data-zc-ref`)이다. 차이는 전달 방식만: WebDriver evaluate(rust_native) vs CDP evaluate(Lisa).

## 1.4 인터랙티브 감지 비교

"클릭 가능한 요소"를 찾는 정확도. 이게 스냅샷 품질을 결정한다.

| | **OpenClaw** | **ZeroClaw agent_browser** | **ZeroClaw rust_native** | **Lisa (현재)** | **Lisa (계획)** |
|---|---|---|---|---|---|
| 1차 | AX Tree role | AX Tree role | 태그/속성 체크 | 태그/속성 체크 | **AX Tree** (CDP API) |
| 2차 | — | — | compact (텍스트 유무) | compact (텍스트 유무) | **DOM 휴리스틱** |
| 3차 | — | — | — | — | **L3 Vision 폴백** |
| React `<div onClick>` | ✅ | ✅ | ❌ | ❌ | ✅ (AX Tree) |
| 접근성 미대응 div | ❌ | ❌ | ❌ | ❌ | ✅ (cursor:pointer) |
| 커버리지 | ~90% | ~90% | ~70% | ~70% | **~95%** |

**OpenClaw/agent_browser도 못 잡는 것:** `<div class="btn" style="cursor:pointer">` 같이 role도 aria도 없는 요소. Lisa의 3단계 전략이 이걸 커버하는 유일한 방법.

## 1.5 안티봇 비교

자동화 탐지를 얼마나 회피하는가.

| | **OpenClaw** | **ZeroClaw** | **Lisa** |
|---|---|---|---|
| 네비게이션 | Playwright `page.goto()` | agent-browser CLI | **JS `window.location.href`** (탐지 불가) |
| webdriver 속성 | Playwright가 관리 | — | **매 네비게이션마다 undefined 오버라이드** |
| User-Agent | Playwright 기본 (탐지 가능) | — | **Chrome 내장 UA 그대로** |
| window.open | Playwright가 관리 | — | **현재 탭으로 리다이렉트** |
| target="_blank" | — | — | **클릭 전 _self로 변환** |
| chrome:// 회피 | — | — | **초기 페이지를 about:blank으로** |

Lisa가 가장 공격적으로 안티봇을 회피한다. `Page.navigate` CDP 명령을 아예 안 쓰고 JS로 네비게이션하는 건 사이트 입장에서 일반 사용자와 구분 불가.

## 1.6 플랫폼 비교

| | **OpenClaw** | **ZeroClaw** | **Lisa** |
|---|---|---|---|
| macOS/Linux | ✅ | ✅ | ✅ (ChromeLauncher) |
| **webOS TV** | ❌ | ❌ | **✅ (WamLauncher)** |
| ARM 보드 | ⚠️ (Chromium 번들 무거움) | ⚠️ (Node.js 필요) | **✅ (추가 바이너리 0)** |
| 멀티 앱 타겟 | ❌ (탭 기반) | ❌ (탭 기반) | **✅ (앱별 CDP target)** |

## 1.7 종합 포지셔닝

```
OpenClaw:          가장 정확한 스냅샷 (AX Tree). 가장 무거움.
                   데스크톱 브라우저 자동화 도구. webOS 불가.

ZeroClaw:          2개 백엔드 선택 가능.
  agent_browser:   접근성 트리 (정확). 외부 CLI + Chrome for Testing 필요.
  rust_native:     DOM 순회 JS (~70%). chromedriver 필요.
                   범용 에이전트 프레임워크. 브라우저는 부가 기능.

Lisa:              가장 가벼움 (시스템 브라우저 직접 연결, 추가 0).
                   현재 스냅샷은 rust_native에서 가져온 DOM 순회 JS.
                   향후 AX Tree + 휴리스틱 + Vision 3단계로 최고 정확도 목표.
                   webOS 네이티브. 안티봇 최강. TV 앱 제어 플랫폼.
```

**Lisa의 차별점:** 다른 둘은 "브라우저를 자동화하는 도구". Lisa는 **"TV 위의 모든 앱을 AI가 제어하는 플랫폼"**. 멀티 앱 타겟, WamLauncher, 앱 OFF 시 자동 실행 — TV 환경에서만 의미 있는 기능이고, 다른 둘에는 없다.

**Lisa와 rust_native의 코드 관계:** PR #85의 `snapshot_script()`는 upstream `rust_native` 백엔드의 것을 가져온 것이다. 로직 동일, 전달 경로만 다름 (WebDriver evaluate → CDP evaluate).

---

# 2부. Lisa 동작 방식 상세

## 2.1 왜 CDP인가

TV 앱의 대다수는 웹 기술(React, Vue 등)로 만들어지고, webOS의 WAM은 Chromium 기반이다.
**Chromium이 있으면 CDP로 DOM에 직접 접근할 수 있다.**

```
L3 Snapshot:  화면 캡처 → "이 픽셀이 버튼인가?" → Vision LLM 추론 → 좌표 클릭
L2 CDP:       DOM 접근 → "이건 <button>이다" → 바로 클릭
```

**구조를 아는 것과 보이는 것만 아는 것의 차이.**

CDP는 이미지를 LLM에 보내지 않아도 UI를 이해할 수 있다:

```
L3: 1920×1080 PNG → Vision LLM → ~1,500 토큰 + 추론 3초
L2: 접근성 트리 텍스트 → ~200 토큰 + 추론 0.5초
```

**7.5배 토큰 절약, 6배 빠름.**

## 2.2 CDP 직접 연결 — WebDriver 제거

```
기존: Chrome ← WebDriver Protocol → chromedriver → fantoccini (Rust)
Lisa: Chrome ← CDP WebSocket → chromiumoxide (Rust) 직접 연결
```

chromedriver는 별도 바이너리. webOS ARM 보드에 올리기 번거롭고, WebDriver는 CDP 위에 만든 래퍼라 오버헤드만 추가됨. 중간을 빼고 직접 연결.

## 2.3 플랫폼 추상화 — BrowserLauncher

브라우저를 "실행하는 방법"만 다르고, 연결 후 제어는 동일:

| 플랫폼 | 런처 | 실행 방법 | CDP 포트 |
|--------|------|----------|---------|
| **Linux** | ChromeLauncher | Chrome 프로세스 직접 스폰 | `--remote-debugging-port=9222` |
| **webOS** | WamLauncher | `luna-send`로 WAM 앱 활성화 | WAM inspector (기본 9998) |

실행 후 `http://127.0.0.1:{port}/json/version` 폴링 → WS 엔드포인트 획득 → `Browser::connect(ws_url)`. 여기서부턴 플랫폼 무관.

`is_webos()` 함수가 런타임에 플랫폼을 자동 감지하여 적절한 런처를 선택한다.

## 2.4 webOS 멀티 웹앱 타겟

Linux에서 Chrome은 하나의 프로세스에 탭 여러 개. webOS는 다르다.
**WAM이 각 웹앱을 별도 프로세스로 관리하고, 하나의 CDP 포트에 모든 앱이 target으로 노출된다.**

```
GET http://127.0.0.1:9998/json →
[
  { "id": "target-1", "title": "YouTube",    "webSocketDebuggerUrl": "ws://..." },
  { "id": "target-2", "title": "Melon",      "webSocketDebuggerUrl": "ws://..." },
  { "id": "target-3", "title": "웹 브라우저", "webSocketDebuggerUrl": "ws://..." }
]
```

**Lisa의 연결 흐름:**
1. `GET /json` → 전체 target 목록
2. appId 또는 title로 필터링 → 원하는 앱의 target 선택
3. 해당 target의 `webSocketDebuggerUrl`로 CDP 연결
4. 앱 전환 시 → 다른 target의 WS URL로 재연결

**단일 앱 포커스 원칙:** Linux "단일 탭" → webOS "단일 앱 포커스". 복합 요청("넷플릭스에서 검색하고, 유튜브에서도 검색해줘")은 순차 처리. TV 화면이 하나라 동시 조작은 무의미.

## 2.5 안티봇 전략

| 조치 | 설계 이유 |
|------|----------|
| **JS 네비게이션** | `window.location.href = url` — CDP `Page.navigate`는 사이트가 감지 가능. JS 실행은 일반 사용자와 구분 불가 |
| **navigator.webdriver 제거** | 매 네비게이션 후 `undefined`로 오버라이드 — 자동화 탐지 1순위 대상 |
| **window.open → 현재 탭** | 새 탭을 열지 않고 현재 탭에서 이동 — LLM 컨텍스트 단순화 + chromiumoxide 멀티탭 불안정 회피 |
| **target="_blank" 제거** | 클릭 전 `_self`로 변환 — 같은 이유 |
| **chrome:// → about:blank** | 초기 탭이 `chrome://newtab`이면 origin 헤더로 자동화 탐지 가능 |
| **Chrome 내장 UA** | User-Agent 안 바꿈 — 버전 불일치로 탐지되는 걸 방지 |
| **최소 Chrome 플래그** | 불필요한 `--disable-*` 플래그 제거 — 플래그 조합 자체가 핑거프린트 |
| **영구 프로필** | 쿠키/로그인 유지 — 매번 빈 프로필이면 의심 |

## 2.6 DOM 안정화 (SPA 대응)

React/Vue 앱은 `readyState='complete'` 후에도 데이터를 비동기로 렌더링한다.

**전략:**
1. `readyState` 폴링 → 'complete' 대기
2. DOM 요소 수 안정화 — `querySelectorAll('*').length`를 300ms 간격으로 폴링, 3회 연속 동일하면 완료
3. 스냅샷 재시도 — ref < 5개면 2/3/4/5초 대기 후 재시도 (리로드 안 함, 같은 DOM 재읽기)

수치(300ms, 3회, 5개, 5초)는 **기본값이며 앱별 튜닝 가능.** Netflix 같은 무거운 앱은 안정화에 2초+, 간단한 설정 앱은 100ms면 충분.

## 2.7 LLM Tool 인터페이스

LLM에게는 **`browser`라는 단일 tool**이 주어진다.

### DOM 기반 액션 (L2 핵심)

| 액션 | 용도 | 핵심 파라미터 |
|------|------|-------------|
| `open` | URL 이동 | `url` |
| `snapshot` | 접근성 트리 스냅샷 | `compact`, `interactive_only`, `depth` |
| `click` | 요소 클릭 | `selector` (@e5, CSS, text=) |
| `fill` | 폼 필드 채우기 | `selector`, `value` |
| `type` | 텍스트 입력 | `selector`, `text` |
| `press` | 키 입력 | `key` (Enter, Tab, Escape 등) |
| `scroll` | 스크롤 | `direction`, `pixels` |
| `hover` | 호버 | `selector` |
| `get_text` | 텍스트 추출 | `selector` |
| `wait` | 대기 | `ms`, `selector`, `text` |
| `find` | 시맨틱 검색 | `by` (role/text/label), `value` |

### OS 레벨 액션 (L3 브릿지)

| 액션 | 용도 | 핵심 파라미터 |
|------|------|-------------|
| `mouse_move` | 좌표 커서 이동 | `x`, `y` |
| `mouse_click` | 좌표 클릭 | `x`, `y`, `button` |
| `mouse_drag` | 드래그 | `from_x/y`, `to_x/y` |
| `key_type` | 텍스트 타이핑 | `text` |
| `key_press` | 키 누르기 | `key` |
| `screen_capture` | 스크린샷 | — |
| `screenshot` | 페이지 캡처 | `full_page`, `path` |

### 셀렉터 체계

```
@e5          → 스냅샷에서 부여된 ref (가장 많이 사용)
#login-btn   → CSS id 셀렉터
.btn-primary → CSS class 셀렉터
text=로그인   → 텍스트 매칭
```

### LLM 동작 패턴

```
1. snapshot → 화면 구조 파악 + ref 획득
2. click @e5 → ref로 요소 클릭
3. snapshot → 결과 확인
4. 반복 또는 완료
```

### Tool이 하나인 이유

22개 액션을 하나의 tool에 넣은 건 의도적이다. tool 수가 늘면 LLM의 1차 호출(tool 선택) 시간이 느려진다. `browser` 하나만 있으면 선택은 즉시, action 파라미터로 세분화. Dynamic Tool Pruning에서도 `browser` 하나만 on/off하면 된다.

## 2.8 접근성 스냅샷 포맷

카테고리 그룹핑으로 LLM이 즉시 행동 가능:

```
title: 쿠팡 - 해찬들 된장
url: https://www.coupang.com/vp/products/...
elements: 52
---
── buttons ──
@e4 "장바구니 담기"
@e5 "바로구매"
── inputs ──
@e2 [type=text] placeholder="검색"
── links ──
@e1 "판매자 가입" href="/vendor-signup"
@e7 "곰곰 순두부 400g"
── other ──
@e3 [heading] "해찬들 맛있는 재래식 된장, 3kg" [level=1]
```

- **buttons** → 클릭할 수 있는 것
- **inputs** → 입력할 수 있는 것
- **links** → 이동할 수 있는 곳
- **other** → 헤딩, 텍스트 등 컨텍스트

HTML 태그 기반 분류라 **모든 웹사이트에서 동작.** 요소 400개 상한으로 토큰 폭발 방지.

## 2.9 연결 생명주기

```
┌─ 정상 ──────────────────────────────────────┐
│ launch → connect → 페이지 선택 → 사용 중     │
└─────────────────────────────────────────────┘
       │ WS 끊김           │ Chrome 크래시
       ▼                   ▼
  reconnect()          relaunch()
  (같은 Chrome에        (새 프로세스 →
   WS 재연결)            새 연결)
```

| 상황 | 처리 |
|------|------|
| WS 끊김, Chrome 살아있음 | `/json/version` 재디스커버리 → 재연결 |
| Chrome/WAM 크래시 | 런처로 재실행 → 새 연결 |
| handler task 죽음 | 다음 액션 시 자동 감지 → 재연결 |

**핵심:** 재연결해도 쿠키/로그인 유지 (영구 프로필 디렉토리).

## 2.10 설정

```toml
[browser]
enabled = true
backend = "cdp_direct"
allowed_domains = ["*"]

[browser.cdp_direct]
debug_port = 9222          # Linux CDP 포트
headless = false           # TV에선 당연히 화면 필요
# chrome_path = "/usr/bin/google-chrome"
# user_data_dir = "~/.zeroclaw/browser-profile/"
# wam_inspector_port = 9998  # webOS WAM 포트
# wam_app_id = "com.webos.app.browser"
# cleanup_stale = false
# timeout_ms = 30000
```

Feature gate: `browser-cdp = ["dep:chromiumoxide"]` (Cargo.toml)

## 2.11 보안

CDP는 DOM 전체에 접근한다. **쿠키, localStorage, 비밀번호 필드까지 읽을 수 있다.** appMCP(앱이 노출한 API만)보다 보안 임팩트가 크다.

| 대책 | 설명 |
|------|------|
| **localhost only** | CDP 포트는 127.0.0.1에서만 수신 (외부 접근 차단) |
| **Lisa 전용 접근** | firewall rule로 Lisa 프로세스만 CDP 포트 연결 허용 |
| **비밀번호 마스킹** | 스냅샷에서 `<input type="password">` 값 제거 |
| **민감 필드 필터링** | 신용카드, 주민번호 등 패턴 감지 시 value 마스킹 |
| **도메인 허용 목록** | `allowed_domains`로 접근 가능 사이트 제한 |
| **행동 로깅** | 모든 CDP 액션을 감사 로그에 기록 |
| **사용자 확인** | 결제/로그인 등 민감 행동 시 사용자 승인 요청 |

```
appMCP: 앱이 노출한 API만 사용 가능 → 앱이 통제
CDP:    DOM 전체 접근 가능 → Lisa가 자제해야 함
```

**권한이 클수록 가드레일이 중요.**

---

# 3부. 문제점 및 향후 방향

## 3.1 현재 스냅샷의 한계

**가장 큰 문제: 인터랙티브 요소 감지 정확도 (~70%)**

현재 DOM 순회 JS는 6개 태그(`a, button, input, select, textarea, summary`) + `el.onclick`으로 판단. 못 잡는 것이 너무 많다:

- **`addEventListener`** — 이벤트 등록의 대부분. `el.onclick`은 소수
- **React/Vue synthetic event** — 실제 DOM에 리스너 안 붙음 (root에 위임)
- **접근성 미대응 `<div>`** — role 없으면 AX Tree에서도 "generic"
- **이벤트 버블링** — 부모에서 받는 클릭

### 해결: 3단계 인터랙티브 감지 전략

```
1단계: 접근성 트리 (AX Tree)
├── CDP Accessibility.getFullAXTree API
├── 17개 INTERACTIVE_ROLES 필터링
├── 접근성 잘 된 사이트: 90%+ 커버
└── 비용: 낮음 (CDP 한 번 호출)
         │
         ▼ 못 잡은 요소들
2단계: DOM 휴리스틱
├── cursor: pointer → 클릭 의도의 가장 강한 신호
├── CDP DOMDebugger.getEventListeners → 실제 리스너 확인
├── CSS 클래스 패턴 (btn, button, click, toggle)
├── data-* 속성 (data-action, data-click)
├── 부모가 <a>나 [role="button"] 안에 있는지
└── 추가 20~30% 커버
         │
         ▼ 그래도 못 잡으면
3단계: L3 Vision 폴백
├── 스크린샷 → Vision LLM에 "클릭 가능한 요소 어딨어?"
└── 나머지 전부 커버 (비용 높음)
```

**핵심 인사이트:** `cursor: pointer`가 가장 현실적인 휴리스틱. 접근성을 전혀 안 해도 클릭 가능한 요소는 거의 다 `cursor: pointer`를 준다. 개발자가 "이건 클릭할 수 있어요"라고 **사용자에게** 알려주는 유일한 시각적 신호.

### 구현 우선순위

1. CDP `Accessibility.getFullAXTree` 도입 → 현재 DOM 순회 JS 대체
2. `cursor: pointer` 휴리스틱 추가 → AX Tree 보완
3. L3 자동 폴백 → 1+2로 ref < 3개면 스크린샷으로 전환

## 3.2 DOM 안정화 수치 하드코딩

300ms, 3회, ref<5, 5초 — 현재 코드에 하드코딩. 앱마다 적정 값이 다르다.

**방향:** config에 튜닝 파라미터 노출. 또는 앱별 프로필 (`appMCP.json`에 `stabilization` 섹션).

## 3.3 webOS 프로덕션 CDP 접근

| 환경 | CDP 접근 |
|------|---------|
| 개발 모드 | ✅ |
| 자체 앱 (Flutter InAppWebView) | ✅ (`isInspectable`) |
| 프로덕션 3rd party | ⚠️ 제한적 |

프로덕션 webOS에서 3rd party 웹앱의 CDP 포트에 접근할 수 있는지는 보안 정책에 따라 다르다. **접근 불가 시 → L3(Snapshot)으로 폴백.**

## 3.4 한계 — L3(Snapshot)이 필요한 이유

| 한계 | 이유 |
|------|------|
| 네이티브 앱 제어 불가 | DOM 없음 (C++, Qt, Flutter Skia) |
| 시스템 UI 제어 불가 | 홈 런처, 설정 등 비웹 |
| 프로덕션 CDP 접근 제한 | webOS 보안 정책 |
| 복잡한 SPA 안정화 | 무한 스크롤, 지연 로딩 등 엣지 케이스 |

**CDP가 못 하는 건 화면 캡처로.** 이게 3계층이 필요한 이유.

## 3.5 PR #85 리뷰 미반영 사항

1. **`max_tool_iterations = 100`** → 30-50으로 줄여야. 무한 루프 위험
2. **`allowed_domains = ["*"]`** → 보안 정책 재고 필요
3. **`prompt.rs` 브라우저 규칙 하드코딩** → config로 분리
4. **`browser-cdp` default feature** → optional로 전환 (webOS 외에는 불필요할 수 있음)

## 3.6 3계층에서의 위치

```
L1 appMCP ──── "앱아, 오징어게임 틀어" → API → 결과
                가장 빠르고 정확, 앱 협조 필요

L2 CDP ─────── "DOM에서 검색창 찾아서 입력하고 클릭" → 구조적
                앱 협조 없이 웹앱 제어, 텍스트 기반

L3 Snapshot ── "화면 보고 어디 클릭할지 판단" → 시각적
                아무거나 되지만 느리고 비쌈
```

---

_v8.0 — 2026-03-27_
_3부 구성: 비교 / 동작 상세 / 문제점 및 방향_
_PR #85 코드 분석 + OpenClaw Playwright 분석 기반_
_Project Elvis L2 계층_
