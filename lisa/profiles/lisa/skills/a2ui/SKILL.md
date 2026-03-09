---
name: a2ui
description: "A2UI v0.8 card rendering. Generate visual UI cards (weather, tasks, info) on the WebSocket channel."
version: "2.3.0"
channels: ws
always: true
---

# A2UI v0.8 — Agent-to-UI Card Rendering

When presenting structured or visual information (weather, tasks, schedules, etc.), include an A2UI card alongside your text response using the delimiter format.

## Response Format

Your response MUST have two parts separated by the delimiter `---a2ui_JSON---`:

```
Your conversational text response here
---a2ui_JSON---
[{surfaceUpdate: ...}, {dataModelUpdate: ...}, {beginRendering: ...}]
```

**Rules:**
1. Always include conversational text BEFORE the delimiter.
2. The JSON after the delimiter is an array of A2UI messages.
3. If no card is needed, just respond with text (no delimiter).
4. When data would benefit from visual display, proactively include a card.

## A2UI Message Rules

1. Each message has exactly ONE action key: `surfaceUpdate`, `dataModelUpdate`, `beginRendering`, or `deleteSurface`.
2. Send order: `surfaceUpdate` → `dataModelUpdate` → `beginRendering`.
3. Components use `id` and `component` (NOT `componentId`/`componentType`).
4. `component` wraps one key = component type (e.g. `{"Text": {...}}`).
5. Text values: `{"literalString": "..."}` or `{"path": "/dataKey"}` for data binding.
6. Children: `{"explicitList": ["child-id-1", "child-id-2"]}`.
7. Same `surfaceId` = replace card. New `surfaceId` = new card. `deleteSurface` removes.

## Component Types

- **Card** — `{child: "id"}`
- **Column** — `{children: {explicitList: [...]}, alignment?: "center"|"start"|"end"}`
- **Row** — `{children: {explicitList: [...]}, alignment?, distribution?: "spaceAround"|"spaceBetween"}`
- **Text** — `{text: {literalString|path}, usageHint?: "h1"|"h2"|"h3"|"h4"|"h5"|"body"|"caption"}`
- **Icon** — `{name: {literalString|path}}`
- **Image** — `{source: {literalString|path}, description?}`
- **Button** — `{child: "text-component-id", action: {name: "action_name", context: [{key: "k", value: {literalString: "v"}}]}}`
- **CheckBox** — `{checked: {literalBoolean|path}, label?: {literalString|path}}`
- **Slider** — `{value: {literalNumber|path}, minValue?, maxValue?}`
- **Divider** — `{}`

## Minimal Example (delimiter + 3-message pattern)

```
Your text response here
---a2ui_JSON---
[{"surfaceUpdate":{"surfaceId":"demo","components":[{"id":"root","component":{"Card":{"child":"title"}}},{"id":"title","component":{"Text":{"text":{"literalString":"Title"},"usageHint":"h2"}}}]}},{"dataModelUpdate":{"surfaceId":"demo","contents":[]}},{"beginRendering":{"surfaceId":"demo","root":"root"}}]
```

## Official JSON Schema (Button & Text)

These are the authoritative schemas from the A2UI v0.8 specification. Follow them exactly.

```json
{
  "Button": {
    "type": "object",
    "additionalProperties": false,
    "properties": {
      "child": { "type": "string", "description": "The ID of the component to display in the button, typically a Text component." },
      "primary": { "type": "boolean" },
      "action": {
        "type": "object",
        "additionalProperties": false,
        "properties": {
          "name": { "type": "string" },
          "context": { "type": "array", "items": { "type": "object", "properties": { "key": { "type": "string" }, "value": { "type": "object", "properties": { "path": { "type": "string" }, "literalString": { "type": "string" }, "literalNumber": { "type": "number" }, "literalBoolean": { "type": "boolean" } } } }, "required": ["key", "value"] } }
        },
        "required": ["name"]
      }
    },
    "required": ["child", "action"]
  },
  "Text": {
    "type": "object",
    "additionalProperties": false,
    "properties": {
      "text": { "type": "object", "properties": { "literalString": { "type": "string" }, "path": { "type": "string" } } },
      "usageHint": { "type": "string", "enum": ["h1", "h2", "h3", "h4", "h5", "caption", "body"] }
    },
    "required": ["text"]
  }
}
```


