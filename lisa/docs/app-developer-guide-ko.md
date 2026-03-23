# Lisa 앱 개발자 가이드

Lisa(ZeroClaw) WebSocket 연동 클라이언트 앱 개발 가이드.

## 1. 아키텍처

```
┌─────────────┐     WebSocket      ┌──────────────┐      LLM API       ┌─────────┐
│   클라이언트  │◄──────────────────►│    Lisa      │◄───────────────────►│Anthropic│
│  (브라우저/  │   /ws/chat         │  (ZeroClaw)  │                     │  Claude │
│   TV 앱)    │                    │   Gateway    │                     │         │
│             │  ◄── text/a2ui ──  │              │                     │         │
│             │  ── msg/action ──► │              │                     │         │
└─────────────┘                    └──────────────┘                     └─────────┘
```

Lisa는 단일 WebSocket 연결로 클라이언트와 통신한다. 세 가지 콘텐츠 타입이 이 채널을 통해 전달된다:

| 타입 | 설명 | 방향 |
|------|------|------|
| **Text** | 일반 텍스트 응답 | 서버 → 클라이언트 |
| **A2UI** | 구조화된 UI 카드 (v0.9 프로토콜) | 서버 → 클라이언트 |
| **a2web** | 리치 HTML 페이지 (차트, 게임 등) | 서버 → 클라이언트 |

---

## 2. WebSocket 프로토콜

### 2.1 연결

```
ws://<host>:<port>/ws/chat?session_id=<optional_id>
```

- **기본 포트**: `42617`
- **session_id**: 생략 시 UUID 자동 생성. 동일 session_id로 재연결하면 대화 히스토리 복원.

### 2.2 클라이언트 → 서버 메시지

**텍스트 메시지:**
```json
{"type": "message", "content": "서울 날씨 알려줘"}
```

**A2UI 버튼 액션 (event → 서버/LLM):**
```json
{
  "type": "a2ui_action",
  "payload": {
    "surfaceId": "quiz-1",
    "name": "answer",
    "sourceComponentId": "btn_a",
    "context": {"answer": "A"}
  }
}
```

### 2.3 서버 → 클라이언트 메시지

응답 턴 동안 순서대로 수신:

| type | 설명 | 시점 |
|------|------|------|
| `history` | 이전 대화 내역 | 연결 시 (히스토리 있을 때) |
| `thinking` | LLM 처리 시작 | 각 턴 시작 |
| `text` | 스트리밍 텍스트 청크 | 응답 중 |
| `a2ui` | A2UI 카드 데이터 | LLM이 카드 생성 시 |
| `done` | 응답 완료 + `full_response` 텍스트 | 각 턴 종료 |
| `error` | 에러 상세 | 실패 시 |

---

## 3. A2UI 카드 (v0.9)

A2UI는 Google의 [Agent-to-UI 프로토콜](https://github.com/google/A2UI)로, 구조화된 카드 렌더링을 위한 것이다. Lisa는 v0.9을 사용한다.

### 3.1 메시지 구조

`a2ui` WS 메시지는 v0.9 메시지 배열을 포함:

```json
{
  "type": "a2ui",
  "messages": [
    {"version": "v0.9", "createSurface": {"surfaceId": "w1", "catalogId": "https://a2ui.org/specification/v0_9/basic_catalog.json"}},
    {"version": "v0.9", "updateComponents": {"surfaceId": "w1", "components": [...]}}
  ]
}
```

### 3.2 Surface 라이프사이클

v0.9 스펙에 따라 surface는 지속적인 UI 세션이다:

1. **`createSurface`** — 플로우당 1회 생성 (surfaceId + catalogId 고정)
2. **`updateComponents`** / **`updateDataModel`** — 동일 surfaceId로 업데이트
3. **`deleteSurface`** — 완료 시 제거

**연속 플로우** (퀴즈, 멀티스텝)에서는 턴 1에서 `createSurface`, 이후 턴에서는 동일 `surfaceId`로 `updateComponents`만 전송.

**독립 조회** (날씨, 검색)에서는 매번 새 `surfaceId` 사용.

**클라이언트가 결정**: 동일 surfaceId 업데이트를 인플레이스 업데이트할지, 추가할지, 교체할지는 클라이언트 판단.

### 3.3 컴포넌트

기본 카탈로그: `Card`, `Column`, `Row`, `List`, `Tabs`, `Text`, `Image`, `Icon`, `Button`, `CheckBox`, `TextField`, `Slider`, `ChoicePicker`, `DateTimeInput`, `Divider`, `Modal`, `AudioPlayer`, `Video`.

컴포넌트 트리는 ID 참조 방식의 플랫 구조:
```json
{"id": "root", "component": "Card", "child": "col"},
{"id": "col", "component": "Column", "children": ["title", "body"]},
{"id": "title", "component": "Text", "text": "Hello", "variant": "h3"},
{"id": "body", "component": "Text", "text": "World", "variant": "body"}
```

### 3.4 버튼 액션

두 가지 타입:

| 타입 | 실행 위치 | 용도 |
|------|-----------|------|
| `event` | **서버** (LLM으로 전달) | 퀴즈 답변, 선택, 네비게이션 |
| `functionCall` | **클라이언트** | URL 열기, 로컬 포맷팅 |

```json
// Event (서버 측)
{"action": {"event": {"name": "answer", "context": {"choice": "A"}}}}

// FunctionCall (클라이언트 측)
{"action": {"functionCall": {"call": "openUrl", "args": {"url": "https://..."}, "returnType": "void"}}}
```

> **중요:** URL 버튼은 반드시 `functionCall.openUrl` 사용. 서버는 헤드리스라 브라우저를 열 수 없다.

### 3.5 데이터 바인딩

컴포넌트는 데이터 모델 값을 경로로 참조 가능:
```json
{"component": "Text", "text": {"path": "/weather/temp"}}
```

이벤트 context에 경로가 포함된 `a2ui_action` 전송 시, 클라이언트는 **반드시** surface의 dataModel에서 경로를 해석한 후 전송해야 한다.

### 3.6 sendDataModel

`createSurface`에 `sendDataModel: true`가 포함된 경우, 클라이언트는 action payload에 현재 `dataModel`을 포함해야 한다:

```json
{
  "type": "a2ui_action",
  "payload": {
    "surfaceId": "todo-1",
    "name": "submit",
    "sourceComponentId": "btn_save",
    "context": {},
    "dataModel": {
      "items": [
        {"text": "빨래", "checked": true},
        {"text": "장보기", "checked": false}
      ]
    }
  }
}
```

서버는 payload(dataModel 포함)를 그대로 LLM에 전달한다.

### 3.7 렌더링

Flutter 앱은 `flutter_genui_a2ui` 패키지 사용. 웹 앱은 A2UI SDK의 `a2ui-surface-v09` 웹 컴포넌트 사용.

---

## 4. a2web 페이지

a2web은 A2UI 카드에 맞지 않는 리치 콘텐츠용 — 차트, 게임, 애니메이션, 커스텀 HTML/CSS/JS.

### 4.1 동작 방식

LLM이 커스텀 HTML이 필요하면 `a2web_render` 도구로 페이지를 생성. 서버가 저장하고 URL 반환:

```
http://<host>:<port>/web/<page_id>/
```

### 4.2 클라이언트 연동

응답 텍스트에 a2web URL이 포함됨. iframe이나 webview로 표시:

```html
<iframe src="http://192.168.45.58:42617/web/abc123/" width="100%" height="400"></iframe>
```

### 4.3 a2web vs A2UI 구분

| 콘텐츠 | 렌더링 방식 |
|---------|-------------|
| 날씨, 리스트, 퀴즈, 일정 | **A2UI 카드** |
| 차트, 그래프, 게임, 애니메이션 | **a2web 페이지** |
| 인터랙티브 HTML/JS 앱 | **a2web 페이지** |

---

## 5. 테스트 앱

개발 및 디버깅을 위한 브라우저 기반 테스트 앱.

### 5.1 설정

```bash
cd lisa/test/a2ui-test
npm install
npx vite --host 0.0.0.0 --port 5173
```

브라우저에서 `http://<host>:5173` 접속.

### 5.2 기능

- Lisa WS 게이트웨이 자동 연결 (`ws://<host>:42617/ws/chat`)
- 공식 웹 컴포넌트로 A2UI 카드 렌더링
- Raw A2UI JSON 복사 버튼 (디버깅용)
- 버튼 액션 처리 (event → 서버, functionCall → 클라이언트)
- 턴별 응답 시간 표시

### 5.3 프로젝트 구조

```
lisa/test/a2ui-test/
├── src/
│   ├── app.ts            # 메인 앱 로직
│   └── v09-adapter.ts    # v0.9 메시지 → surface 어댑터
├── index.html            # UI + 스타일
├── package.json
└── dist/                 # 빌드 산출물
```

---

## 6. 설정

### 6.1 필수 설정

`config.toml`:
```toml
[a2ui]
enabled = true

[a2web]
enabled = true

[gateway]
port = 42617
host = "0.0.0.0"       # 외부 접속 허용
```

### 6.2 환경 변수

```bash
# 프로바이더
export ZEROCLAW_PROVIDER=anthropic
export ZEROCLAW_MODEL=claude-sonnet-4-6
export ANTHROPIC_API_KEY=sk-ant-...

# 게이트웨이
export ZEROCLAW_GATEWAY_HOST=0.0.0.0
```

---

## 7. 트러블슈팅

| 증상 | 원인 | 해결 |
|------|------|------|
| A2UI 카드 미생성 | A2UI 스킬 미로드 | config에서 `a2ui.enabled = true` 확인 |
| 텔레그램에서 카드가 텍스트로 표시 | A2UI는 WS 전용 | 테스트 앱 또는 WS 클라이언트 사용 |
| 버튼 미동작 | 잘못된 액션 타입 | event vs functionCall 확인 |
| 다른 기기에서 연결 불가 | 게이트웨이 localhost 바인딩 | `host = "0.0.0.0"` 설정 |
| a2web 페이지 404 | a2web 비활성 | `a2web.enabled = true` 설정 |
