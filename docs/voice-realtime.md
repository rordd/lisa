# Voice Realtime Integration

## Overview

ZeroClaw supports real-time voice conversations via the OpenAI Realtime API.
A single Agent handles both text chat and voice through a unified web interface.

- **Text chat**: Full Agent pipeline (system prompt, tools, memory)
- **Voice**: WebSocket relay to Realtime API with automatic context injection
- **Shared context**: Chat and voice share conversation history and memory

## Architecture

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
│  │ (text chat) │    │ → history injection (N turns)│  │
│  └─────────────┘    │ → voice system prompt       │  │
│         │           └─────────────────────────────┘  │
│  ┌──────▼─────────────────────────────────────────┐  │
│  │              Agent.history                      │  │
│  │  (chat + [Voice] transcript shared)             │  │
│  └─────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────┐  │
│  │              Memory (auto_save)                  │  │
│  │  chat: memory.store() in Agent::turn()           │  │
│  │  voice: transcript pair persistence in relay      │  │
│  └─────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────┐
│               Voice Web Server (Axum)                 │
│                                                       │
│  GET /          → Unified UI (chat + 🎤 mic)          │
│  POST /api/chat → Agent::turn() (full pipeline)       │
│  GET /ws        → WebSocket relay → Realtime API      │
│  GET /api/config→ Client-side config (barge-in etc.)  │
└──────────────────────────────────────────────────────┘
```

## Design Decisions

### Separate RealtimeProvider trait
HTTP request-response and WebSocket full-duplex are fundamentally different patterns.
Instead of extending the existing `Provider` trait, a separate `RealtimeProvider` trait is defined.

### Single Agent, dual interface
One Agent owns both `chat_provider` and `realtime_provider`.
Workspace files (SOUL.md, MEMORY.md, etc.) and conversation history are naturally shared.

### Voice system prompt from workspace
`build_voice_system_prompt_from_workspace()` reads SOUL.md, IDENTITY.md, USER.md, TOOLS.md, MEMORY.md
to build a compact voice prompt. Takes priority over the static `system_prompt` config.

### Transcript pair persistence
Voice transcripts are saved to Memory when a user+assistant pair completes.
Pairs shorter than the minimum character threshold are skipped as noise.

## Setup

### Prerequisites

```bash
# Rust (1.75+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Ubuntu system dependencies
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev libasound2-dev
```

### Configuration

Copy `.env.example` and fill in your values:

```bash
cp .env.example .env
```

#### Chat LLM

```bash
export ZEROCLAW_API_KEY=<your-api-key>
export ZEROCLAW_MODEL=gpt-4o
```

#### Voice (Realtime API — separate config)

```bash
export ZEROCLAW_VOICE_ENABLED=true
export ZEROCLAW_VOICE_PROVIDER=azure        # "azure" or "openai"
export ZEROCLAW_VOICE_API_KEY=<realtime-api-key>
export ZEROCLAW_VOICE_MODEL=gpt-realtime
export ZEROCLAW_VOICE_NAME=alloy
export ZEROCLAW_VOICE_LANGUAGE=en
```

For Azure:
```bash
export ZEROCLAW_VOICE_AZURE_ENDPOINT=<endpoint>
export ZEROCLAW_VOICE_AZURE_DEPLOYMENT=<deployment-name>
```

Tuning parameters are available in `config.toml` `[voice]` section.
See [config-reference.md](config-reference.md) for the full list.

### TLS Certificate (optional, recommended)

Browsers require HTTPS for microphone access:

```bash
openssl req -x509 -newkey rsa:2048 -keyout key.pem -out cert.pem -days 365 -nodes \
  -subj "/CN=$(hostname)"
```

### Build & Run

```bash
source .env
cargo build --release

# HTTP
./target/release/zeroclaw voice --port 3000

# HTTPS
./target/release/zeroclaw voice --port 3000 --tls-cert cert.pem --tls-key key.pem
```

| Flag | Default | Description |
|------|---------|-------------|
| `--port` | 3000 | Server port |
| `--host` | 0.0.0.0 | Bind address |
| `--tls-cert` | — | TLS certificate PEM path |
| `--tls-key` | — | TLS private key PEM path |

## Usage

Open `https://<host>:3000` in a browser.

- **Chat**: Type a message and press Enter or click Send
- **Voice**: Click 🎤 to start a voice session, click again to stop
- **Barge-in**: Speak while the AI is responding to interrupt (500ms debounce)
- **Context sharing**: Recent chat history is injected into voice sessions; voice transcripts merge back into chat history with `[Voice]` prefix
- **Function calling**: Not yet supported in voice mode (planned for future release)

## Data Flow

### Text chat
```
Browser → POST /api/chat → Agent::turn() → chat_provider → Memory → Response
```

### Voice
```
Browser → WebSocket /ws → relay_session → Realtime API
  ├─ On connect: inject recent N turns from Agent.history
  ├─ During session: transcript → Agent.history merge + Memory persist
  └─ Function calling: not yet supported (planned)
```

## File Layout

```
src/providers/
  realtime.rs            — RealtimeProvider trait + OpenAI/Azure implementation
  realtime_types.rs      — RealtimeConfig, AudioChunk, TranscriptTurn types

src/voice/
  mod.rs                 — Voice module exports
  session.rs             — VoiceSession (history injection, transcript relay)
  web.rs                 — Axum web server, AppState, WebSocket relay,
                           /api/chat, /api/config, transcript memory persistence
  static/index.html      — Unified UI (chat + voice, dark theme)

src/agent/
  agent.rs               — Agent with realtime_provider, voice_config;
                           create_voice_session(), merge_voice_transcripts()
  prompt.rs              — VoiceIdentitySection, build_voice_prompt()

src/config/schema.rs     — VoiceConfig struct, ZEROCLAW_VOICE_* env overrides
src/main.rs              — Commands::Voice (--port, --host, --tls-cert, --tls-key)
```

## Troubleshooting

### WebSocket connection failure
- Check that the Realtime API endpoint is reachable
- Verify `ZEROCLAW_VOICE_AZURE_ENDPOINT` is correct

### Browser microphone blocked
- Use HTTPS (recommended) or add the URL to `chrome://flags/#unsafely-treat-insecure-origin-as-secure`

### Chat returns 502
- Web chat uses `ZEROCLAW_API_KEY` / `ZEROCLAW_MODEL` (not voice config)
- Ensure chat LLM settings are correct

### Audio overlap / echo
- Barge-in should cancel the current response (500ms debounce)
- Use a headset to prevent speaker-to-mic feedback
