# appMCP — 앱 매니페스트 기반 AI 도구 자동 등록 시스템

> 앱 설치 시 MCP 매니페스트(appMCP.json)를 드롭하면,
> AI 에이전트가 자동으로 도구를 인식하고 앱을 호출할 수 있는 아키텍처.
>
> Android Intent System + MCP의 결합.

---

## 1. 문제

| 상황 | 기존 MCP | 앱 환경 (TV/모바일) |
|------|----------|-------------------|
| 서버 수명 | 데몬이 소유, 항시 가동 | OS가 관리, 언제든 죽음 |
| 도구 등록 | config에 수동 등록 | 앱 설치/삭제 시 동적 |
| 스케일 | 서버 3-5개 | 앱 수십~수백 개 |
| 실행 | 항상 연결됨 | 필요 시 앱 실행 후 연결 |

**MCP는 "서버가 항상 떠있다"를 가정한다. 앱은 그렇지 않다.**

## 2. 해결: appMCP.json

앱이 설치될 때 자신의 AI 도구 스펙을 파일로 남긴다.
에이전트는 이 파일을 읽어 도구를 등록하고, 필요할 때 앱을 깨워 실행한다.

```
┌─────────────────────────────────────────────────────┐
│                    앱 스토어                          │
│  넷플릭스 설치 ──► /var/lisa/apps/com.netflix/       │
│                    └── appMCP.json                   │
│  유튜브 설치   ──► /var/lisa/apps/com.youtube/       │
│                    └── appMCP.json                   │
│  멜론 설치     ──► /var/lisa/apps/com.melon/         │
│                    └── appMCP.json                   │
└─────────────────────────────────────────────────────┘
                         │
                    inotify / FSEvent
                         │
                         ▼
┌─────────────────────────────────────────────────────┐
│                   Lisa (에이전트)                     │
│                                                     │
│  ┌── Tool Registry ──────────────────────────────┐  │
│  │ netflix__search    (from appMCP.json)          │  │
│  │ netflix__play      (from appMCP.json)          │  │
│  │ netflix__top10     (from appMCP.json)          │  │
│  │ youtube__search    (from appMCP.json)          │  │
│  │ youtube__play      (from appMCP.json)          │  │
│  │ melon__play        (from appMCP.json)          │  │
│  │ melon__playlist    (from appMCP.json)          │  │
│  │ weather            (built-in skill)            │  │
│  │ memory_search      (built-in)                  │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## 3. appMCP.json 스펙

```json
{
  "$schema": "https://lisa.ai/schemas/appMCP/v1.json",
  "appId": "com.netflix",
  "name": "Netflix",
  "version": "2.1.0",
  "mcpVersion": "2025-01-01",
  "category": "entertainment",
  "launch": {
    "method": "luna-send",
    "uri": "luna://com.webos.service.applicationManager/launch",
    "params": { "id": "com.netflix" }
  },
  "transport": {
    "type": "websocket",
    "endpoint": "ws://localhost:9100/mcp/{appId}",
    "direction": "app-to-lisa",
    "registration": "dynamic"
  },
  "tools": [
    {
      "name": "search",
      "description": "콘텐츠 검색 (영화, 시리즈, 다큐멘터리). 검색 결과는 contentId, title, year, rating을 포함하는 배열로 리턴.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "query": { "type": "string", "description": "검색어" },
          "category": {
            "type": "string",
            "enum": ["all", "movie", "series", "documentary"]
          }
        },
        "required": ["query"]
      }
    },
    {
      "name": "play",
      "description": "콘텐츠 재생. playing(boolean), title, duration(초)을 리턴.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "contentId": { "type": "string" },
          "resumePosition": { "type": "number", "description": "초 단위" }
        },
        "required": ["contentId"]
      },
      "timeoutMs": 10000
    },
    {
      "name": "top10",
      "description": "오늘의 TOP 10 콘텐츠",
      "inputSchema": {
        "type": "object",
        "properties": {
          "category": {
            "type": "string",
            "enum": ["all", "movie", "series"],
            "default": "all"
          }
        }
      }
    }
  ]
}
```

### 필드 설명

| 필드 | 필수 | 설명 |
|------|------|------|
| `appId` | ✅ | 앱 패키지 ID (고유 식별자) |
| `name` | ✅ | 사람이 읽을 수 있는 앱 이름 |
| `version` | ✅ | 앱 버전 |
| `mcpVersion` | ✅ | MCP 프로토콜 버전 |
| `category` | | 앱 카테고리 (프루닝 그룹 자동 생성에 사용) |
| `launch` | ✅ | 앱 실행 방법 |
| `launch.method` | ✅ | 실행 방식 (`luna-send`, `exec`, `dbus`) |
| `launch.uri` | | luna-send URI |
| `launch.params` | | 실행 파라미터 |
| `transport` | ✅ | MCP 통신 방법 |
| `transport.type` | ✅ | `websocket` |
| `transport.endpoint` | ✅ | Lisa MCP 엔드포인트 (고정: `ws://localhost:9100/mcp/{appId}`) |
| `transport.direction` | ✅ | `app-to-lisa`: 앱이 리사에 연결 (기본) |
| `transport.registration` | | `dynamic`: 앱이 연결 시 capability 교환 |
| `tools` | ✅ | MCP 도구 목록 |
| `tools[].name` | ✅ | 도구 이름 |
| `tools[].description` | ✅ | 도구 설명 (LLM이 읽음). 리턴값 형식도 여기에 자연어로 기술 |
| `tools[].inputSchema` | ✅ | JSON Schema (파라미터). MCP 표준 준수 |
| `tools[].timeoutMs` | | 실행 타임아웃 (기본: 10000) |

> **Note:** `outputSchema`는 MCP 2025-01-01 표준에 없으므로 사용하지 않는다. 리턴값 형식은 `description`에 자연어로 기술한다.

## 4. 라이프사이클

```
앱 설치 ──────────────────────────────────────────────
    │
    ▼
  appMCP.json 생성 → /var/lisa/apps/{appId}/
    │
    ▼
  리사 감지 (inotify) → tool 등록 (netflix__search 등)
    │
    ├── LLM은 이 시점부터 tool 인식
    └── 앱은 아직 안 떠있어도 OK

사용자 요청 ──────────────────────────────────────────
    │
    "넷플릭스에서 오징어게임 찾아줘"
    │
    ▼
  LLM → netflix__search({ query: "오징어게임" })
    │
    ▼
  리사: 앱 연결 상태 확인
    │
    ├── 연결됨 → MCP tools/call → 결과 리턴
    │
    └── 미연결 ──┐
                 ▼
            luna-send로 앱 실행
                 │
                 ▼
            앱 기동 → ws://localhost:9100/mcp/com.netflix 연결
                 │
                 ▼
            리사가 MCP initialize 전송 → 앱 응답
                 │
                 ▼
            MCP tools/call → 결과 리턴

앱 삭제 ──────────────────────────────────────────────
    │
    ▼
  앱 프로세스 종료 → WS 끊김 → 진행 중 호출 타임아웃 처리
    │
    ▼
  appMCP.json 삭제 → 리사 감지 → tool 제거
```

## 5. 연결 핸드셰이크

Lisa는 고정 포트 `9100`에서 MCP WebSocket 엔드포인트를 제공한다.
앱은 `ws://localhost:9100/mcp/{appId}` 경로로 연결하여 자신을 식별한다.

```
앱 기동
  │
  ▼
앱 → ws://localhost:9100/mcp/com.netflix 연결
  │
  ▼
리사: URL path에서 appId 추출 → appMCP.json과 매칭 검증
  │
  ▼
리사 → 앱 (MCP initialize, Lisa가 client):
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-01-01",
    "capabilities": {},
    "clientInfo": {
      "name": "lisa",
      "version": "1.0.0"
    }
  },
  "id": 1
}
  │
  ▼
앱 → 리사 (initialize 응답):
{
  "jsonrpc": "2.0",
  "result": {
    "protocolVersion": "2025-01-01",
    "capabilities": { "tools": {} },
    "serverInfo": {
      "name": "com.netflix",
      "version": "2.1.0"
    }
  },
  "id": 1
}
  │
  ▼
연결 완료. 리사가 MCP tools/call 가능.
```

> **MCP 역할 정리:** Lisa = MCP client, 앱 = MCP server.
> `initialize`는 MCP 표준에 따라 client(Lisa)가 server(앱)에 보낸다.
> appId 식별은 WS path(`/mcp/{appId}`)로 처리하며, MCP 메시지에 비표준 필드를 추가하지 않는다.

## 6. 도구 네이밍

```
{appId의 마지막 세그먼트}__{tool_name}

com.netflix     → netflix__search, netflix__play
com.youtube     → youtube__search, youtube__play
com.melon       → melon__play, melon__playlist
com.webos.tv    → tv__setVolume, tv__channelChange
```

**충돌 처리:** 마지막 세그먼트가 동일한 경우 (예: `com.lge.netflix`와 `com.netflix`), 후순위 앱은 풀 appId를 prefix로 사용한다: `com.lge.netflix__search`. 등록 시 자동 감지.

LLM이 보는 도구 목록:
```
netflix__search     - 콘텐츠 검색 (영화, 시리즈, 다큐멘터리)
netflix__play       - 콘텐츠 재생
netflix__top10      - 오늘의 TOP 10 콘텐츠
youtube__search     - YouTube 동영상 검색
youtube__play       - YouTube 동영상 재생
melon__play         - 음악 재생
tv__setVolume       - TV 볼륨 조절
tv__channelChange   - TV 채널 변경
weather             - 날씨 조회 (built-in skill)
```

## 7. Dynamic Tool Pruning 연계

앱 50개 × 도구 3개 = 150개 도구 → LLM 컨텍스트 폭발.

**appMCP.json의 `category` 필드로 자동 프루닝 그룹 생성:**

```
appMCP.json 로드 시:
  category: "entertainment" → entertainment 그룹에 자동 배정
  category: "utility"       → utility 그룹에 자동 배정
  category 없음             → default 그룹 (항상 활성)
```

**기존 `tool_filter_groups`와 병합:**

```toml
# 자동 생성 (appMCP category 기반)
[[tool_filter_groups]]
name = "entertainment"
mode = "on_demand"
patterns = ["netflix__*", "youtube__*", "melon__*"]

# 수동 설정 (기존 built-in)
[[tool_filter_groups]]
name = "tv_control"
mode = "on_demand"
patterns = ["tv__*"]
```

리사가 카테고리별로 자동 프루닝 그룹 생성 → "영상 관련" 요청에만 entertainment 도구 활성화.

## 8. 보안

| 위협 | 대응 |
|------|------|
| 악성 앱이 타 appId로 위장 등록 | WS 연결 시 appId 매칭 검증: appMCP.json이 존재하는 appId만 연결 허용 |
| appMCP.json이 없는 앱이 연결 시도 | 연결 거부 + 로그 경고 |
| 앱이 선언하지 않은 tool 실행 요청 | appMCP.json에 선언된 tool만 라우팅 |
| 민감 기능 무단 호출 | Phase 2+에서 Autonomy Ladder 연계: L0(알림만) → L2(자동실행) 단계적 허용 |

> **Phase 1 최소 보안:** appMCP.json 존재 여부 + appId 매칭 검증.
> **향후 확장:** IPK 서명 검증으로 appId ↔ 앱 바이너리 바인딩, Autonomy Ladder 기반 per-tool 권한.

## 9. 에러 처리

| 상황 | 처리 |
|------|------|
| appMCP.json 파싱 실패 | 로그 경고, 해당 앱 도구 미등록 |
| 앱 실행 실패 (luna-send 에러) | LLM에 에러 리턴: "넷플릭스를 실행할 수 없습니다" |
| 앱 실행 후 WS 미연결 (타임아웃) | LLM에 에러 리턴: "앱이 응답하지 않습니다" |
| MCP tools/call 타임아웃 | LLM에 에러 리턴: "실행 시간이 초과되었습니다" |
| 앱 업데이트 중 도구 스키마 변경 | 핫리로드로 자동 갱신 |
| 앱 삭제 (프로세스 종료) | WS 끊김 → 진행 중 호출 타임아웃 처리 → appMCP.json 삭제 감지 → 도구 제거 |
| 도구 네이밍 충돌 | 후순위 앱은 풀 appId prefix로 폴백 |

## 10. 비교

### vs Android App Functions

| | Android App Functions | appMCP |
|---|---|---|
| 등록 | `@AppFunction` 어노테이션 → 컴파일 타임 | appMCP.json → 설치 타임 |
| 저장 | AppSearch DB | 파일시스템 (JSON) |
| 디스커버리 | AppFunctionManager.observe() | inotify + 핫로딩 |
| 통신 | Android Binder (IPC) | WebSocket (MCP) |
| 프로토콜 | 커스텀 (AppFunctionData) | **MCP 표준** |
| 플랫폼 | Android only | **플랫폼 독립** |
| AI 연결 | Gemini only | **모델 독립** |

### vs MCP 기본

| | MCP 기본 | appMCP |
|---|---|---|
| 서버 등록 | config에 수동 | **앱 설치 시 자동** |
| 서버 수명 | 항상 가동 | **필요 시 실행** |
| 도구 발견 | 서버 연결 후 tools/list | **파일에서 미리 로드** |
| 스케일 | 서버 수 개 | **앱 수십~수백** |
| 연결 끊김 | 도구 제거 | **도구 유지 (캐시)** |

## 11. 구현 계획

### Phase 1: 매니페스트 로더
- [ ] `/var/lisa/apps/` 디렉토리 스캔 → appMCP.json 파싱
- [ ] inotify/FSEvent 감시 → 핫리로드
- [ ] 파싱된 도구를 Tool Registry에 등록 (McpToolWrapper 재사용)
- [ ] 앱 미연결 시 에러 리턴 ("앱이 실행되지 않았습니다")
- [ ] 보안: appMCP.json 존재 여부 + appId 매칭 검증

### Phase 2: 앱 실행 + MCP 연결
- [ ] lisa MCP 엔드포인트 (`ws://localhost:9100/mcp/{appId}`) 구현
- [ ] luna-send 앱 실행 로직
- [ ] 앱 WS 연결 수신 → path에서 appId 추출 → appMCP.json 매칭
- [ ] Lisa → 앱 MCP initialize → tools/call 라우팅
- [ ] 연결 끊김 감지 → 상태 업데이트 (도구는 유지)

### Phase 3: 프루닝 + 보안 강화
- [ ] category 기반 자동 tool_filter_groups 생성
- [ ] 앱 실행 → WS 연결 타임아웃 조정
- [ ] 멀티 앱 동시 연결 관리
- [ ] Autonomy Ladder 연계: per-tool 권한 레벨

---

_v1.1 — 2026-03-26_
_Google App Functions + MCP 표준 기반 설계_
_Project Elvis 핵심 아키텍처_
