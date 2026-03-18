---
name: a2ui
description: "A2UI v0.9 card rendering. Visually present structured data (weather, schedules, lists, comparisons, quizzes, forms). Proactively render cards when data benefits from visual display."
version: "2.0.0"
channels: ws
always: true
---

# A2UI v0.9 — Card Rendering

You know the **A2UI v0.9 specification** and its basic catalog (`https://a2ui.org/specification/v0_9/basic_catalog.json`). Use that knowledge directly. Do NOT invent custom syntax.

## Response Format

Include A2UI messages inside `<a2ui-json>...</a2ui-json>` tags alongside your text. Each tag = one A2UI message. Text goes before/between/after tags.

CRITICAL: If you say "카드 만들었어" but don't include `<a2ui-json>` tags, the user sees NOTHING.

## A2UI vs a2web

- **A2UI cards** (`<a2ui-json>`) — structured displays: weather, calendar, lists, quizzes, comparisons, recipes
- **a2web** (`a2web_render` tool) — rich/complex: charts, games, animations, custom HTML/CSS/JS

If it fits A2UI components → use A2UI. If it needs custom HTML/JS → use a2web.

## Message Types (all require `"version": "v0.9"`)

1. `createSurface` — init surface with `catalogId`
2. `updateComponents` — define component tree (one must be `id: "root"`)
3. `updateDataModel` — update data bindings
4. `deleteSurface` — remove surface

## Rules

- Always `createSurface` first, then `updateComponents`
- Use data bindings (`{"path": "/data/key"}`) for dynamic values
- URL buttons → `functionCall.openUrl` (server is headless, NO event actions for URLs)
- Quiz/choice buttons → `event` action (needs server reasoning)
- Use the FULL component range: Card, Column, Row, List, Tabs, Text, Image, Icon, Button, CheckBox, TextField, Slider, ChoicePicker, DateTimeInput, Divider, Modal, AudioPlayer, Video

## Example

User: "오늘 날씨 어때?"

오늘 서울은 맑고 12°C야!

<a2ui-json>{"version": "v0.9", "createSurface": {"surfaceId": "w1", "catalogId": "https://a2ui.org/specification/v0_9/basic_catalog.json"}}</a2ui-json>

<a2ui-json>{"version": "v0.9", "updateComponents": {"surfaceId": "w1", "components": [
  {"id": "root", "component": "Card", "child": "col"},
  {"id": "col", "component": "Column", "children": ["title", "temp"]},
  {"id": "title", "component": "Text", "text": "🌤️ 오늘의 서울 날씨", "variant": "h3"},
  {"id": "temp", "component": "Text", "text": "12°C / 맑음", "variant": "body"}
]}}</a2ui-json>
