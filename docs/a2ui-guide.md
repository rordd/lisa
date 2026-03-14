# A2UI Integration Guide

ZeroClaw's A2UI v0.9 implementation guide. For the Google A2UI protocol itself, refer to the [official spec](https://github.com/anthropics/a2ui).

## Audience

- **App developers**: Client developers rendering A2UI cards via WebSocket
- **QA / Testers**: Verifying A2UI scenarios using the test app and E2E suite
- **Architects**: Reviewing the end-to-end flow

---

## 1. Architecture Overview

```
┌─────────────┐     WebSocket      ┌──────────────┐      LLM API       ┌─────────┐
│   Client    │◄──────────────────►│   ZeroClaw   │◄───────────────────►│  Azure  │
│  (Browser)  │   /ws/chat         │   Gateway    │                     │ OpenAI  │
│             │                    │              │                     │         │
│ A2UI Render │  ◄── a2ui msg ──  │  a2ui parser │  ◄── <a2ui-json> ── │  LLM    │
│             │  ── action ──►    │  action res. │  ── prompt ──►      │         │
└─────────────┘                    └──────────────┘                     └─────────┘
```

**Key points:**
- A2UI works **only on the WebSocket channel** (CLI, Telegram, etc. are text-only)
- ZeroClaw has no separate backend — clients connect directly to the WS endpoint
- When the LLM generates card data in `<a2ui-json>` tags, ZeroClaw parses and forwards it to the client

## 2. Client Integration

### 2.1 WebSocket Connection

```
ws://<host>:<port>/ws/chat?session_id=<optional_id>
```

- **Default port**: `42617`
- **session_id**: Auto-generated UUID if omitted. Reconnecting with the same session_id restores conversation history.
- **Bind**: Defaults to `127.0.0.1` (localhost only). For external access, set `ZEROCLAW_GATEWAY_HOST=0.0.0.0` in `.env`.

### 2.2 Message Protocol

#### Client → Server

**Text message:**
```json
{"type": "message", "content": "What's the weather in Seoul?"}
```

**Button / form action:**
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

#### Server → Client

Messages are received in order:

| type | Description | When |
|---|---|---|
| `history` | Restore previous conversation turns | On connect (if history exists) |
| `thinking` | LLM processing started | Start of each turn |
| `a2ui` | A2UI card data | When the LLM response includes a card |
| `done` | Response complete + full text | End of each turn |
| `error` | Error | On processing failure |

**`a2ui` message structure:**
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

### 2.3 Action Types

Button `action` has two types:

| Type | Runs on | Purpose | Examples |
|---|---|---|---|
| `functionCall` | **Client** | Open URLs, formatting, validation | `openUrl`, `formatDate` |
| `event` | **Server** (forwarded to LLM) | Choices, quiz answers, follow-ups | Quiz answer selection |

**Important:** Buttons that open URLs must use `functionCall.openUrl`. The server is headless and cannot open a browser.

### 2.4 Data Binding Paths in Event Context

A2UI buttons with `event` actions can reference dataModel values using **data binding paths** (`{"path": "/options/B"}`). The client MUST resolve these paths against the surface's dataModel before sending to the server.

**Example flow:**
```
dataModel: {"options": {"A": "Sahara Desert", "B": "Atlantic Ocean"}}

Button context: {"choice": {"path": "/options/B"}}
  ↓ client resolves path
Sent to server: {"choice": "Atlantic Ocean"}
  ↓ server forwards to LLM
LLM sees: "User selected: Atlantic Ocean"
```

### 2.5 App Developer Requirements

Clients rendering A2UI cards MUST implement the following:

1. **Resolve data binding paths in event context.** Before sending an `a2ui_action` to the server, iterate over all values in `context` and resolve any `{"path": "..."}` objects against the surface's `dataModel`. This ensures the server receives actual values, not path references.

2. **Use `functionCall.openUrl` for URL buttons.** Never use `event` for URL-opening actions — the server is headless.

3. **Preserve `surfaceId` and `sourceComponentId`.** These are required for the server to route actions correctly.

## 3. Configuration

### 3.1 Enabling A2UI

`config.toml`:
```toml
[a2ui]
enabled = true
```

### 3.2 Environment Variables (.env)

```bash
# Allow WS connections from external devices
export ZEROCLAW_GATEWAY_HOST=0.0.0.0

# Reasoning level — medium or higher required for A2UI card generation
export ZEROCLAW_PROVIDER_REASONING_LEVEL=medium
```

### 3.3 Reasoning Level vs A2UI Quality

| Level | A2UI Pass Rate | Notes |
|---|---|---|
| medium | **75%** | Recommended |
| minimal | **17%** | Massive card generation failure — do not use |

At minimal reasoning, the LLM skips A2UI JSON generation and describes card content as plain text instead.

## 4. SKILL.md Management

The A2UI skill definition is auto-generated from the Google A2UI SDK:

```bash
cd lisa/profiles/lisa/skills/a2ui
pip install a2ui   # Google A2UI SDK
python generate_skill.py --write
```

- `generate_skill.py` fetches the schema from the SDK and generates `SKILL.md`
- `SKILL.md` frontmatter sets `channels: ws` (WebSocket only)
- To customize: edit `ROLE_DESCRIPTION` in `generate_skill.py`

## 5. E2E Testing

### 5.1 Test Structure

```
lisa/test/a2ui-test/
├── tests/e2e/
│   └── multi_turn_test.py    # 12-scenario multi-turn test suite
├── src/
│   ├── a2ui-renderer.ts      # A2UI renderer (Lit component)
│   ├── app.ts                # Test web app
│   └── v09-adapter.ts        # v0.9 adapter
├── index.html                # Test UI
├── serve.py                  # Dev server
└── package.json
```

### 5.2 Running Automated Tests

```bash
# Prerequisites: ZeroClaw daemon running + a2ui.enabled = true
cd lisa/test/a2ui-test
pip install websockets
python tests/e2e/multi_turn_test.py
```

Runs 12 scenarios sequentially, each with up to 5 multi-turn interactions.

### 5.3 Test Scenarios

| Scenario | Prompt | Validation |
|---|---|---|
| weather_card | Weather query | Card generation, dataModel keys |
| quiz_geography | Geography quiz | Multi-turn button interaction |
| todo_checklist | To-do checklist | CheckBox, Slider components |
| comparison_table | Phone comparison | Comparison card structure |
| recipe_card | Recipe card | Multi-turn portion adjustment |
| schedule_weekly | Weekly schedule | Day-based data model |
| game_menu | Simple game | TextField input |
| survey_form | Survey card | ChoicePicker, TextField |
| travel_itinerary | Travel plan card | Multi-turn itinerary navigation |
| calculator | Calculator card | Multi-button layout |
| restaurant_recommendation | Restaurant recommendation | Hallucination detection |
| music_playlist | Playlist recommendation | URL buttons (functionCall.openUrl) |

### 5.4 Detected Issue Types

| Issue | Description |
|---|---|
| `NO_CARD_ON_FIRST_TURN` | No A2UI card generated on the first turn |
| `HALLUCINATION` | LLM promises non-existent capabilities (search, playback, calendar, etc.) |
| `HALLUCINATION_BUTTON` | Button created for an impossible action |
| `WRONG_ACTION_TYPE` | URL found in event context (should be functionCall) |
| `CONVERSATION_LOOP` | Repeated confirmation questions without providing content |
| `EMPTY_DATA_MODEL` | Card exists but contains no data |

### 5.5 Test Web App (Manual Testing)

A browser-based UI for visually inspecting A2UI cards and clicking buttons:

```bash
cd lisa/test/a2ui-test
npm install
python serve.py    # Test UI at http://localhost:8765
```

### 5.6 Test Report

After running automated tests, results are saved to `tests/reports/multi_turn_report.json`:

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

## 6. Known Limitations

- **Hallucination**: LLM intermittently promises non-existent capabilities (real-time search, calendar add, etc.). Occurs regardless of reasoning level.
- **NO_CARD non-determinism**: Identical prompts may occasionally fail to produce a card (LLM non-deterministic output).
- **Session memory pollution**: Repeating the same prompt with the same session_id causes prior facts to be injected, producing "I'll make it again" patterns. Use unique session_ids for testing.
- **reasoning_level=minimal**: Incompatible with A2UI. Use medium or higher.
