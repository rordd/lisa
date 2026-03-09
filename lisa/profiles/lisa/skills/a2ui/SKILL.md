---
name: a2ui
description: "A2UI v0.8 카드 렌더링. WS 채널에서 날씨, 태스크, 정보 등 시각적 UI 카드를 생성할 때 사용."
version: "2.1.0"
channels: ws
always: true
---

# A2UI v0.8 — Agent-to-UI Card Rendering

When presenting structured or visual information (weather, tasks, schedules, etc.), include an A2UI card alongside your text response using the delimiter format.

## Response Format

Your response MUST have two parts separated by the delimiter `---a2ui_JSON---`:

```
자연스러운 텍스트 응답 (사용자에게 보이는 대화)
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
- **Text** — `{text: {literalString|path}, usageHint?: "h1"|"h2"|"h3"|"body"|"caption"}`
- **Icon** — `{name: {literalString|path}}`
- **Image** — `{source: {literalString|path}, description?}`
- **Button** — `{child: "text-component-id", action: {name: "action_name", context: [{key: "k", value: {literalString: "v"}}]}}`
- **CheckBox** — `{checked: {literalBoolean|path}, label?: {literalString|path}}`
- **Slider** — `{value: {literalNumber|path}, minValue?, maxValue?}`
- **Divider** — `{}`

**IMPORTANT: Button requires `child` (id of a Text component for the label) and `action` (with `name` and `context`). Do NOT use `label` or `onClick` — those are invalid.**

## Example 1: Weather Card

```
지금 서울 강서구 날씨야. 최고 5도, 최저 -2도로 좀 쌀쌀해!
---a2ui_JSON---
[{"surfaceUpdate":{"surfaceId":"weather","components":[{"id":"root","component":{"Card":{"child":"col"}}},{"id":"col","component":{"Column":{"children":{"explicitList":["temp-row","location","desc"]},"alignment":"center"}}},{"id":"temp-row","component":{"Row":{"children":{"explicitList":["temp-high","temp-low"]},"alignment":"start"}}},{"id":"temp-high","component":{"Text":{"text":{"path":"/tempHigh"},"usageHint":"h1"}}},{"id":"temp-low","component":{"Text":{"text":{"path":"/tempLow"},"usageHint":"h2"}}},{"id":"location","component":{"Text":{"text":{"path":"/location"},"usageHint":"h3"}}},{"id":"desc","component":{"Text":{"text":{"path":"/description"},"usageHint":"caption"}}}]}},{"dataModelUpdate":{"surfaceId":"weather","contents":[{"key":"tempHigh","valueString":"5°C"},{"key":"tempLow","valueString":"-2°C"},{"key":"location","valueString":"서울 강서구"},{"key":"description","valueString":"맑음, 바람 9km/h"}]}},{"beginRendering":{"surfaceId":"weather","root":"root"}}]
```

## Example 2: Button Card (structure only — generate your own content!)

```
텍스트 응답 여기에
---a2ui_JSON---
[{"surfaceUpdate":{"surfaceId":"buttons-demo","components":[{"id":"root","component":{"Card":{"child":"col"}}},{"id":"col","component":{"Column":{"children":{"explicitList":["title","btn-row"]},"alignment":"center"}}},{"id":"title","component":{"Text":{"text":{"literalString":"YOUR TITLE HERE"},"usageHint":"h2"}}},{"id":"btn-row","component":{"Row":{"children":{"explicitList":["btn-a","btn-b"]},"distribution":"spaceAround"}}},{"id":"btn-a-text","component":{"Text":{"text":{"literalString":"Option A"}}}},{"id":"btn-a","component":{"Button":{"child":"btn-a-text","action":{"name":"select","context":[{"key":"choice","value":{"literalString":"a"}}]}}}},{"id":"btn-b-text","component":{"Text":{"text":{"literalString":"Option B"}}}},{"id":"btn-b","component":{"Button":{"child":"btn-b-text","action":{"name":"select","context":[{"key":"choice","value":{"literalString":"b"}}]}}}}]}},{"beginRendering":{"surfaceId":"buttons-demo","root":"root"}}]
```

**Note:** This is a structural template. Always generate unique content — different questions, options, surfaceIds, etc. Never copy examples verbatim.
