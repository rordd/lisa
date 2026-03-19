# Voice Realtime 통합

## 개요

ZeroClaw는 OpenAI Realtime API를 통한 실시간 음성 대화를 지원한다.
하나의 Agent가 텍스트 채팅과 음성을 통합 웹 인터페이스로 동시 처리한다.

- **텍스트 채팅**: Agent 파이프라인 전체 (system prompt, tools, memory)
- **음성 대화**: WebSocket 릴레이로 Realtime API 연결, 자동 맥락 주입
- **맥락 공유**: 채팅과 음성이 대화 이력과 메모리를 공유

## 아키텍처

```
┌──────────────────────────────────────────────────────┐
│                      Agent                            │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────┐  │
│  │ chat_provider│  │ realtime_    │  │ voice_     │  │
│  │ (Provider)   │  │ provider     │  │ config     │  │
│  └──────┬───────┘  └──────┬───────┘  └────────────┘  │
│         │                 │                           │
│  ┌──────▼─────┐    ┌─────▼──────────────────────┐   │
│  │Agent::turn()│    │create_voice_session()      │   │
│  │ (텍스트)    │    │ → 이력 주입 (N턴)           │   │
│  └─────────────┘    │ → voice system prompt       │  │
│         │           └─────────────────────────────┘  │
│  ┌──────▼─────────────────────────────────────────┐  │
│  │              Agent.history                      │  │
│  │  (chat + [Voice] transcript 공유)               │  │
│  └─────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────┐  │
│  │              Memory (auto_save)                  │  │
│  │  chat: Agent::turn() 내 memory.store()           │  │
│  │  voice: relay_session에서 턴 페어 단위 저장       │  │
│  └─────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────┐
│               Voice Web Server (Axum)                 │
│                                                       │
│  GET /          → 통합 UI (채팅 + 🎤 마이크)          │
│  POST /api/chat → Agent::turn() (전체 파이프라인)      │
│  GET /ws        → WebSocket 릴레이 → Realtime API     │
│  GET /api/config→ 클라이언트 설정 (barge-in 등)       │
└──────────────────────────────────────────────────────┘
```

## 설계 결정

### 별도 RealtimeProvider trait
HTTP request-response와 WebSocket full-duplex는 근본적으로 다른 패턴이므로,
기존 `Provider` trait을 확장하지 않고 별도 `RealtimeProvider` trait으로 정의했다.

### 단일 Agent, 이중 인터페이스
Agent 1개가 `chat_provider`와 `realtime_provider`를 동시에 소유.
workspace 파일(SOUL.md, MEMORY.md 등)과 대화 이력이 자연스럽게 공유된다.

### workspace 기반 voice system prompt
`build_voice_system_prompt_from_workspace()`가 SOUL.md, IDENTITY.md, USER.md, TOOLS.md, MEMORY.md를 읽어서
voice 전용 compact prompt를 자동 구성한다. 정적 `system_prompt` 설정보다 우선.

### Transcript 페어 저장
voice transcript는 user+assistant 페어가 완성될 때 Memory에 저장.
합산 글자 수가 기준 미만이면 노이즈로 판단하고 스킵.

## 설정

### 사전 요구사항

```bash
# Rust (1.75+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Ubuntu 시스템 의존성
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev libasound2-dev
```

### 환경 설정

`.env.example`을 복사해서 값을 채운다:

```bash
cp .env.example .env
```

#### 챗 LLM

```bash
export ZEROCLAW_API_KEY=<your-api-key>
export ZEROCLAW_MODEL=gpt-4o
```

#### Voice (Realtime API — 별도 설정)

```bash
export ZEROCLAW_VOICE_ENABLED=true
export ZEROCLAW_VOICE_PROVIDER=azure        # "azure" 또는 "openai"
export ZEROCLAW_VOICE_API_KEY=<realtime-api-key>
export ZEROCLAW_VOICE_MODEL=gpt-realtime
export ZEROCLAW_VOICE_NAME=alloy
export ZEROCLAW_VOICE_LANGUAGE=ko
```

Azure 사용 시:
```bash
export ZEROCLAW_VOICE_AZURE_ENDPOINT=<endpoint>
export ZEROCLAW_VOICE_AZURE_DEPLOYMENT=<deployment-name>
```

튜닝 파라미터는 `config.toml`의 `[voice]` 섹션에서 설정 가능.
전체 목록은 [config-reference.md](config-reference.md) 참고.

### TLS 인증서 (선택, 권장)

브라우저 마이크 사용을 위해 HTTPS 필요:

```bash
openssl req -x509 -newkey rsa:2048 -keyout key.pem -out cert.pem -days 365 -nodes \
  -subj "/CN=$(hostname)"
```

### 빌드 & 실행

```bash
source .env
cargo build --release

# HTTP
./target/release/zeroclaw voice --port 3000

# HTTPS
./target/release/zeroclaw voice --port 3000 --tls-cert cert.pem --tls-key key.pem
```

| 플래그 | 기본값 | 설명 |
|--------|--------|------|
| `--port` | 3000 | 서버 포트 |
| `--host` | 0.0.0.0 | 바인드 주소 |
| `--tls-cert` | — | TLS 인증서 PEM 경로 |
| `--tls-key` | — | TLS 키 PEM 경로 |

## 사용법

브라우저에서 `https://<host>:3000` 접속.

- **채팅**: 메시지 입력 후 Enter 또는 전송 버튼 클릭
- **음성**: 🎤 클릭으로 세션 시작, 다시 클릭으로 종료
- **Barge-in**: AI 응답 중 말하면 500ms debounce 후 응답 취소
- **맥락 공유**: 최근 채팅 이력이 voice 세션에 자동 주입, voice transcript는 `[Voice]` prefix로 채팅 이력에 병합
- **Function calling**: 음성 모드에서는 아직 미지원 (향후 구현 예정)

## 데이터 흐름

### 텍스트 채팅
```
Browser → POST /api/chat → Agent::turn() → chat_provider → Memory → Response
```

### 음성
```
Browser → WebSocket /ws → relay_session → Realtime API
  ├─ 연결 시: Agent.history에서 최근 N턴 주입
  ├─ 대화 중: transcript → Agent.history에 merge + Memory 저장
  └─ Function calling: 아직 미지원 (향후 구현 예정)
```

## 파일 구조

```
src/providers/
  realtime.rs            — RealtimeProvider trait + OpenAI/Azure 구현
  realtime_types.rs      — RealtimeConfig, AudioChunk, TranscriptTurn 타입

src/voice/
  mod.rs                 — voice 모듈 exports
  session.rs             — VoiceSession (이력 주입, transcript 릴레이)
  web.rs                 — Axum 웹서버, AppState, WebSocket 릴레이,
                           /api/chat, /api/config, transcript 메모리 저장
  static/index.html      — 통합 UI (채팅 + 음성, 다크 테마)

src/agent/
  agent.rs               — Agent에 realtime_provider, voice_config 추가;
                           create_voice_session(), merge_voice_transcripts()
  prompt.rs              — VoiceIdentitySection, build_voice_prompt()

src/config/schema.rs     — VoiceConfig struct, ZEROCLAW_VOICE_* env overrides
src/main.rs              — Commands::Voice (--port, --host, --tls-cert, --tls-key)
```

## 트러블슈팅

### WebSocket 연결 실패
- Realtime API 엔드포인트 접근 가능한지 확인
- `ZEROCLAW_VOICE_AZURE_ENDPOINT` 값 확인

### 브라우저 마이크 차단
- HTTPS 사용 (권장) 또는 `chrome://flags/#unsafely-treat-insecure-origin-as-secure`에 URL 추가

### 채팅 502 에러
- 웹 채팅은 `ZEROCLAW_API_KEY` / `ZEROCLAW_MODEL` 사용 (voice 설정 아님)
- 챗 LLM 설정 확인

### 오디오 겹침 / 에코
- Barge-in이 현재 응답을 취소해야 함 (500ms debounce)
- 헤드셋 사용 권장 (스피커→마이크 피드백 방지)
