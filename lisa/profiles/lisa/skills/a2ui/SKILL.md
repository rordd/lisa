---
name: a2ui
description: "A2UI v0.9 card rendering. Visually present structured data (weather, schedules, lists, comparisons, quizzes, forms). Proactively render cards when data benefits from visual display."
version: "2.0.0"
channels: ws,lisa
always: true
---

# A2UI v0.9 — Card Rendering

You know the **A2UI v0.9 specification** and its basic catalog (`https://a2ui.org/specification/v0_9/basic_catalog.json`). Use that knowledge directly. Do NOT invent custom syntax.

## Response Format

Include A2UI messages inside `<a2ui-json>...</a2ui-json>` tags alongside your text. Each tag = one A2UI message.

CRITICAL: If you mention a card but don't include `<a2ui-json>` tags, the user sees NOTHING.

## Surface Lifecycle

- `createSurface` once per flow → `updateComponents` / `updateDataModel` on same `surfaceId`
- Continuous flows (quiz, multi-step): reuse the same `surfaceId` across turns
- Independent lookups (weather, search): new `surfaceId` each time

## Rules

- URL buttons → `functionCall.openUrl` (server is headless)
- Quiz/choice buttons → `event` action
- Button `action` MUST use nested v0.9 format:
  - Event: `{"action": {"event": {"name": "choice", "context": {"answer": "A"}}}}`
  - Function: `{"action": {"functionCall": {"call": "openUrl", "args": {"url": "https://..."}, "returnType": "void"}}}`

## Examples

### Card with text

User: "How's the weather today?"

It's sunny and 12°C in Seoul!

<a2ui-json>{"version": "v0.9", "createSurface": {"surfaceId": "w1", "catalogId": "https://a2ui.org/specification/v0_9/basic_catalog.json"}}</a2ui-json>

<a2ui-json>{"version": "v0.9", "updateComponents": {"surfaceId": "w1", "components": [
  {"id": "root", "component": "Card", "child": "col"},
  {"id": "col", "component": "Column", "children": ["title", "temp"]},
  {"id": "title", "component": "Text", "text": "🌤️ Today's Weather in Seoul", "variant": "h3"},
  {"id": "temp", "component": "Text", "text": "12°C / Sunny", "variant": "body"}
]}}</a2ui-json>

### List pattern

`List > Column[Text(h4), Text, Button(openUrl)]` per item. Repeat Column for each result.
