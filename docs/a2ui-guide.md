# A2UI Integration Guide

ZeroClaw의 A2UI v0.9 구현 가이드. Google A2UI 프로토콜 자체는 [공식 스펙](https://github.com/anthropics/a2ui)을 참조.

## 대상 독자

- **앱 개발자**: WS를 열어서 A2UI를 실제 렌더링할 클라이언트 담당자
- **테스트 담당자**: 테스트 앱을 통해 A2UI 시나리오를 검증할 QA
- **아키텍트**: 전체 플로우를 점검할 설계 담당자

---

## 1. 아키텍처 개요

```
┌─────────────┐     WebSocket      ┌──────────────┐      LLM API       ┌─────────┐
│  클라이언트   │◄──────────────────►│   ZeroClaw   │◄───────────────────►│  Azure  │
│  (브라우저)   │   /ws/chat         │   Gateway    │                     │ OpenAI  │
│              │                    │              │                     │         │
│  A2UI 렌더러  │  ◄── a2ui msg ──  │  a2ui 파서   │  ◄── <a2ui-json> ── │  LLM    │
│              │  ── action ──►    │  액션 해석    │  ── 프롬프트 ──►    │         │
└─────────────┘                    └──────────────┘                     └─────────┘
```

**핵심 포인트:**
- A2UI는 **WebSocket 채널에서만** 동작 (CLI, Telegram 등은 텍스트 전용)
- ZeroClaw는 별도 백엔드 없음 — 클라이언트가 직접 WS에 연결
- LLM이 `<a2ui-json>` 태그로 카드 데이터를 생성하면, ZeroClaw가 파싱하여 클라이언트에 전달

## 2. 클라이언트 연동

### 2.1 WebSocket 연결

```
ws://<host>:<port>/ws/chat?session_id=<optional_id>
```

- **기본 포트**: `42617`
- **session_id**: 생략 시 자동 UUID 생성. 동일 session_id로 재접속하면 이전 대화 히스토리 복원
- **바인드**: 기본 `127.0.0.1` (로컬 전용). 외부 접속 필요 시 `.env`에 `ZEROCLAW_GATEWAY_HOST=0.0.0.0` 설정

### 2.2 메시지 프로토콜

#### 클라이언트 → 서버

**텍스트 메시지:**
```json
{"type": "message", "content": "서울 날씨 알려줘"}
```

**버튼/폼 액션:**
```json
{
  "type": "a2ui_action",
  "payload": {
    "surfaceId": "weather_card",
    "name": "select_option",
    "sourceComponentId": "btn_hourly",
    "context": {"choice": "B"}
  }
}
```

#### 서버 → 클라이언트

메시지는 순서대로 수신됨:

| type | 설명 | 시점 |
|---|---|---|
| `history` | 이전 대화 턴 복원 | 연결 직후 (히스토리가 있을 때) |
| `thinking` | LLM 처리 시작 알림 | 매 턴 시작 |
| `a2ui` | A2UI 카드 데이터 | LLM 응답에 카드가 포함될 때 |
| `done` | 응답 완료 + 전체 텍스트 | 매 턴 끝 |
| `error` | 에러 | 처리 실패 시 |

**`a2ui` 메시지 구조:**
```json
{
  "type": "a2ui",
  "messages": [
    {"version": "v0.9", "createSurface": {"surfaceId": "w1", "catalogId": "basic"}},
    {"version": "v0.9", "updateComponents": {"surfaceId": "w1", "components": [...]}},
    {"version": "v0.9", "updateDataModel": {"surfaceId": "w1", "path": "/", "value": {"temp": "25°C"}}}
  ]
}
```

### 2.3 액션 타입

버튼의 `action`에는 두 가지 타입이 있음:

| 타입 | 실행 위치 | 용도 | 예시 |
|---|---|---|---|
| `functionCall` | **클라이언트** | URL 열기, 포맷팅, 밸리데이션 | `openUrl`, `formatDate` |
| `event` | **서버** (LLM에 전달) | 선택지, 퀴즈 답변, 후속 질문 | 퀴즈 정답 선택 |

**중요:** URL을 여는 버튼은 반드시 `functionCall.openUrl`을 사용해야 함. 서버는 헤드리스이므로 브라우저를 열 수 없음.

### 2.4 선택지 해석

ZeroClaw는 버튼 클릭 시 데이터 모델에서 선택지 텍스트를 자동 해석:

```
사용자가 "B" 클릭 → ZeroClaw가 dataModel에서 B = "해왕성" 매핑
→ LLM에 "User selected: B = 해왕성" 전달
```

지원하는 키 패턴: `options.B`, `optionB`, `optB`, `option_B`

## 3. 설정

### 3.1 A2UI 활성화

`config.toml`:
```toml
[a2ui]
enabled = true
```

### 3.2 환경 변수 (.env)

```bash
# 외부 디바이스에서 WS 접속이 필요한 경우
export ZEROCLAW_GATEWAY_HOST=0.0.0.0

# Reasoning level — A2UI 카드 생성에는 medium 이상 필수
export ZEROCLAW_PROVIDER_REASONING_LEVEL=medium
```

### 3.3 Reasoning Level과 A2UI 품질

| Level | A2UI PASS율 | 비고 |
|---|---|---|
| medium | **75%** | 권장 |
| minimal | **17%** | 카드 미생성 대량 발생 — 사용 금지 |

minimal에서는 LLM이 A2UI JSON 생성을 포기하고 텍스트로만 카드 내용을 설명함.

## 4. SKILL.md 관리

A2UI 스킬 정의는 Google A2UI SDK에서 자동 생성:

```bash
cd lisa/profiles/lisa/skills/a2ui
pip install a2ui   # Google A2UI SDK
python generate_skill.py --write
```

- `generate_skill.py`가 SDK에서 스키마를 가져와 `SKILL.md` 생성
- `SKILL.md`의 frontmatter에 `channels: ws`로 WS 전용 설정됨
- 커스터마이징: `generate_skill.py`의 `ROLE_DESCRIPTION` 수정

## 5. E2E 테스트

### 5.1 테스트 환경

```
lisa/test/a2ui-test/
├── tests/e2e/
│   └── multi_turn_test.py    # 12개 시나리오 멀티턴 테스트
├── src/
│   ├── a2ui-renderer.ts      # A2UI 렌더러 (Lit 컴포넌트)
│   ├── app.ts                # 테스트 웹앱
│   └── v09-adapter.ts        # v0.9 어댑터
├── index.html                # 테스트 UI
├── serve.py                  # 개발 서버
└── package.json
```

### 5.2 자동화 테스트 실행

```bash
# 사전 조건: ZeroClaw 데몬 가동 + a2ui.enabled = true
cd lisa/test/a2ui-test
pip install websockets
python tests/e2e/multi_turn_test.py
```

12개 시나리오를 순서대로 실행하며, 각 시나리오에서 최대 5턴의 멀티턴 대화를 수행.

### 5.3 테스트 시나리오

| 시나리오 | 프롬프트 | 검증 포인트 |
|---|---|---|
| weather_card | 서울 날씨 알려줘 | 카드 생성, dataModel 키 |
| quiz_geography | 세계 수도 퀴즈 내줘 | 멀티턴 버튼 인터랙션 |
| todo_checklist | 오늘 할일 체크리스트 만들어줘 | CheckBox, Slider 컴포넌트 |
| comparison_table | 아이폰 16 vs 갤럭시 S25 비교해줘 | 비교 카드 구조 |
| recipe_card | 김치찌개 레시피 카드로 보여줘 | 멀티턴 분량 조절 |
| schedule_weekly | 이번 주 운동 계획 세워줘 | 요일별 데이터 모델 |
| game_menu | 간단한 게임 하나 만들어줘 | TextField 입력 |
| survey_form | 만족도 설문조사 카드 만들어줘 | ChoicePicker, TextField |
| travel_itinerary | 제주도 2박3일 여행 계획 카드로 만들어줘 | 멀티턴 일정 네비게이션 |
| calculator | 간단한 계산기 카드 만들어줘 | 다수 버튼 레이아웃 |
| restaurant_recommendation | 강서구 맛집 추천해줘 | 할루시네이션 탐지 |
| music_playlist | 집중할 때 듣기 좋은 플레이리스트 추천해줘 | URL 버튼 (functionCall.openUrl) |

### 5.4 검출하는 이슈 유형

| 이슈 | 설명 |
|---|---|
| `NO_CARD_ON_FIRST_TURN` | 첫 턴에 A2UI 카드 미생성 |
| `HALLUCINATION` | LLM이 없는 능력을 약속 (검색, 재생, 캘린더 등) |
| `HALLUCINATION_BUTTON` | 실행 불가능한 기능의 버튼 생성 |
| `WRONG_ACTION_TYPE` | URL이 event에 포함됨 (functionCall이어야 함) |
| `CONVERSATION_LOOP` | 확인 질문만 반복하고 컨텐츠 미제공 |
| `EMPTY_DATA_MODEL` | 카드는 있으나 데이터 없음 |

### 5.5 테스트 웹앱 (수동 테스트)

브라우저에서 직접 A2UI 카드를 확인하고 버튼을 클릭할 수 있는 테스트 UI:

```bash
cd lisa/test/a2ui-test
npm install
python serve.py    # http://localhost:8765 에서 테스트 UI 실행
```

### 5.6 테스트 결과 리포트

자동화 테스트 실행 후 `tests/reports/multi_turn_report.json`에 결과 저장:

```json
{
  "summary": {"total": 12, "passed": 9, "failed": 3},
  "scenarios": [
    {
      "name": "weather_card",
      "passed": true,
      "turns": 1,
      "turn_details": [
        {
          "a2ui_count": 3,
          "components": ["Card", "Text", "Row", "Column"],
          "data_model_keys": ["temperature", "humidity", "wind"],
          "elapsed_ms": 34350
        }
      ]
    }
  ]
}
```

## 6. 알려진 제약사항

- **할루시네이션**: LLM이 간헐적으로 없는 능력을 약속 (실시간 검색, 캘린더 추가 등). reasoning level과 무관하게 발생.
- **NO_CARD 비결정성**: 동일 프롬프트에서도 카드가 생성되지 않는 경우 있음 (LLM 비결정적 출력).
- **세션 메모리 오염**: 같은 session_id로 동일 프롬프트를 반복하면 이전 fact가 주입되어 "다시 만들어줄게" 패턴 발생. 테스트 시 고유 session_id 사용 권장.
- **reasoning_level=minimal**: A2UI와 양립 불가. medium 이상 필수.
