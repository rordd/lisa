# Lisa App Developer Guide

Guide for client app developers integrating with Lisa (ZeroClaw) via WebSocket.

## 1. Architecture

```
┌─────────────┐     WebSocket      ┌──────────────┐      LLM API       ┌─────────┐
│   Client    │◄──────────────────►│    Lisa      │◄───────────────────►│Anthropic│
│  (Browser/  │   /ws/chat         │  (ZeroClaw)  │                     │  Claude │
│   TV App)   │                    │   Gateway    │                     │         │
│             │  ◄── text/a2ui ──  │              │                     │         │
│             │  ── msg/action ──► │              │                     │         │
└─────────────┘                    └──────────────┘                     └─────────┘
```

Lisa communicates with clients via a single WebSocket connection. Three content types flow through this channel:

| Type | Description | Direction |
|------|-------------|-----------|
| **Text** | Plain text responses | Server → Client |
| **A2UI** | Structured UI cards (v0.9 protocol) | Server → Client |
| **a2web** | Rich HTML pages (charts, games, etc.) | Server → Client |

---

## 2. WebSocket Protocol

### 2.1 Connection

```
ws://<host>:<port>/ws/chat?session_id=<optional_id>
```

- **Default port**: `42617`
- **session_id**: Auto-generated UUID if omitted. Same session_id restores conversation history on reconnect.

### 2.2 Client → Server Messages

**Text message:**
```json
{"type": "message", "content": "서울 날씨 알려줘"}
```

**A2UI button action (event → server/LLM):**
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

### 2.3 Server → Client Messages

Messages arrive in order during a response turn:

| type | Description | When |
|------|-------------|------|
| `history` | Previous conversation turns | On connect (if history exists) |
| `thinking` | LLM processing started | Start of each turn |
| `text` | Streamed text chunk | During response |
| `a2ui` | A2UI card data | When LLM generates a card |
| `done` | Response complete + `full_response` text | End of each turn |
| `error` | Error details | On failure |

---

## 3. A2UI Cards (v0.9)

A2UI is Google's [Agent-to-UI protocol](https://github.com/google/A2UI) for structured card rendering. Lisa uses v0.9.

### 3.1 Message Structure

The `a2ui` WS message contains an array of v0.9 messages:

```json
{
  "type": "a2ui",
  "messages": [
    {"version": "v0.9", "createSurface": {"surfaceId": "w1", "catalogId": "https://a2ui.org/specification/v0_9/basic_catalog.json"}},
    {"version": "v0.9", "updateComponents": {"surfaceId": "w1", "components": [...]}}
  ]
}
```

### 3.2 Surface Lifecycle

Per the v0.9 spec, a surface is a persistent UI session:

1. **`createSurface`** — Create once per flow (surfaceId + catalogId are fixed)
2. **`updateComponents`** / **`updateDataModel`** — Update the same surfaceId
3. **`deleteSurface`** — Remove when done

For **continuous flows** (quiz, multi-step), Lisa sends `createSurface` on turn 1, then only `updateComponents` on subsequent turns with the same `surfaceId`.

For **independent lookups** (weather, search), Lisa uses a new `surfaceId` each time.

**Client decides** how to handle same-surfaceId updates — in-place update, append, or replace.

### 3.3 Components

The basic catalog includes: `Card`, `Column`, `Row`, `List`, `Tabs`, `Text`, `Image`, `Icon`, `Button`, `CheckBox`, `TextField`, `Slider`, `ChoicePicker`, `DateTimeInput`, `Divider`, `Modal`, `AudioPlayer`, `Video`.

Component tree is flat with ID references:
```json
{"id": "root", "component": "Card", "child": "col"},
{"id": "col", "component": "Column", "children": ["title", "body"]},
{"id": "title", "component": "Text", "text": "Hello", "variant": "h3"},
{"id": "body", "component": "Text", "text": "World", "variant": "body"}
```

### 3.4 Button Actions

Two types:

| Type | Runs on | Use for |
|------|---------|---------|
| `event` | **Server** (forwarded to LLM) | Quiz answers, choices, navigation |
| `functionCall` | **Client** | Open URLs, local formatting |

```json
// Event (server-side)
{"action": {"event": {"name": "answer", "context": {"choice": "A"}}}}

// FunctionCall (client-side)
{"action": {"functionCall": {"call": "openUrl", "args": {"url": "https://..."}, "returnType": "void"}}}
```

> **Important:** URL buttons MUST use `functionCall.openUrl`. The server is headless.

### 3.5 Data Binding

Components can reference data model values via paths:
```json
{"component": "Text", "text": {"path": "/weather/temp"}}
```

When sending `a2ui_action` with event context containing paths, the client **MUST** resolve paths against the surface's dataModel before sending.

### 3.6 sendDataModel

When `createSurface` includes `sendDataModel: true`, the client must include the current `dataModel` in action payloads:

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
        {"text": "Laundry", "checked": true},
        {"text": "Groceries", "checked": false}
      ]
    }
  }
}
```

The server passes the payload (including `dataModel`) directly to the LLM as-is.

### 3.7 Rendering

For Flutter apps, use the `flutter_genui_a2ui` package. For web apps, use the `a2ui-surface-v09` web component from the A2UI SDK.

---

## 4. a2web Pages

a2web is for rich content that doesn't fit A2UI cards — charts, games, animations, custom HTML/CSS/JS.

### 4.1 How It Works

When the LLM needs custom HTML, it uses the `a2web_render` tool to generate a page. The server stores it and returns a URL:

```
http://<host>:<port>/web/<page_id>/
```

### 4.2 Client Integration

The client receives an a2web URL in the response text. Display it in an iframe or webview:

```html
<iframe src="http://192.168.45.58:42617/web/abc123/" width="100%" height="400"></iframe>
```

### 4.3 When to Expect a2web vs A2UI

| Content | Rendered as |
|---------|-------------|
| Weather, lists, quizzes, schedules | **A2UI card** |
| Charts, graphs, games, animations | **a2web page** |
| Interactive HTML/JS apps | **a2web page** |

---

## 5. Test App

A browser-based test app for development and debugging.

### 5.1 Setup

```bash
cd lisa/test/a2ui-test
npm install
npx vite --host 0.0.0.0 --port 5173
```

Open `http://<host>:5173` in a browser.

### 5.2 Features

- Auto-connects to Lisa WS gateway (`ws://<host>:42617/ws/chat`)
- Renders A2UI cards using the official web component
- Shows raw A2UI JSON with copy button (for debugging)
- Handles button actions (event → server, functionCall → client)
- Displays response time per turn

### 5.3 Project Structure

```
lisa/test/a2ui-test/
├── src/
│   ├── app.ts            # Main app logic
│   └── v09-adapter.ts    # v0.9 message → surface adapter
├── index.html            # UI + styles
├── package.json
└── dist/                 # Build output
```

---

## 6. Configuration

### 6.1 Required Settings

`config.toml`:
```toml
[a2ui]
enabled = true

[a2web]
enabled = true

[gateway]
port = 42617
host = "0.0.0.0"       # For external access
```

### 6.2 Environment Variables

```bash
# Provider
export ZEROCLAW_PROVIDER=anthropic
export ZEROCLAW_MODEL=claude-sonnet-4-6
export ANTHROPIC_API_KEY=sk-ant-...

# Gateway
export ZEROCLAW_GATEWAY_HOST=0.0.0.0
```

---

## 7. Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| No A2UI cards generated | A2UI skill not loaded | Check `a2ui.enabled = true` in config |
| Cards show as text in Telegram | A2UI is WS-only | Use the test app or a WS client |
| Buttons don't work | Wrong action type | Check event vs functionCall |
| Can't connect from another device | Gateway bound to localhost | Set `host = "0.0.0.0"` |
| a2web pages 404 | a2web disabled | Set `a2web.enabled = true` |
