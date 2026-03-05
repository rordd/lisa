---
name: weather
description: "Check current weather and forecasts via wttr.in. Used when the user asks about weather, temperature, rain/snow."
version: "1.0.0"
always: true
---

# Weather Skill

Check weather via the wttr.in API. No API key required.

## When to Use

- "What's the weather like?"
- "Will it rain today?"
- "Tomorrow's temperature?"
- "This week's weather"

## Default Location

Refer to USER.md for the weather location. If no location is specified, use the default.

## Commands

### Current Weather
```bash
curl -s "wttr.in/Seoul+Gangseo-gu?format=%l:+%c+%t+(feels+like+%f),+wind+%w,+humidity+%h&lang=ko"
```

### Today/Tomorrow/Day After
```bash
# Today
curl -s "wttr.in/Seoul+Gangseo-gu?0&lang=ko"

# Tomorrow
curl -s "wttr.in/Seoul+Gangseo-gu?1&lang=ko"

# 3-day forecast
curl -s "wttr.in/Seoul+Gangseo-gu?lang=ko"
```

### JSON (for parsing)
```bash
curl -s "wttr.in/Seoul+Gangseo-gu?format=j1&lang=ko"
```

### Other Cities
```bash
curl -s "wttr.in/Busan?format=3&lang=ko"
curl -s "wttr.in/New+York?format=3"
```

## Format Codes
- `%c` — weather emoji
- `%t` — temperature
- `%f` — feels like
- `%w` — wind
- `%h` — humidity
- `%p` — precipitation

## Notes
- Avoid calling too frequently (rate limit)
- Korean cities can be searched in Korean
