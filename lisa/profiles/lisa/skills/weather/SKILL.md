---
name: weather
description: "Check current weather and forecasts. Used when the user asks about weather, temperature, rain/snow."
version: "2.0.0"
always: true
---

# Weather Skill

Check weather using Open-Meteo API (primary) with wttr.in fallback.

## When to Use

- "What's the weather like?"
- "Will it rain today?"
- "Tomorrow's temperature?"
- "This week's weather"

## Default Location

Seoul Gangseo-gu (latitude=37.55, longitude=126.85). Check USER.md for overrides.

## Primary: Open-Meteo (try this first)

No API key required. Returns JSON.

### Current Weather
```sh
curl -s "https://api.open-meteo.com/v1/forecast?latitude=37.55&longitude=126.85&current=temperature_2m,apparent_temperature,weather_code,wind_speed_10m,relative_humidity_2m,precipitation&timezone=Asia/Seoul"
```

### 3-Day Forecast
```sh
curl -s "https://api.open-meteo.com/v1/forecast?latitude=37.55&longitude=126.85&daily=temperature_2m_max,temperature_2m_min,precipitation_probability_max,weather_code&timezone=Asia/Seoul&forecast_days=3"
```

### Hourly (today)
```sh
curl -s "https://api.open-meteo.com/v1/forecast?latitude=37.55&longitude=126.85&hourly=temperature_2m,precipitation_probability,weather_code&timezone=Asia/Seoul&forecast_days=1"
```

### Other Locations
Change latitude/longitude. Examples:
- Busan: latitude=35.18, longitude=129.08
- Jeju: latitude=33.50, longitude=126.53
- New York: latitude=40.71, longitude=-74.01

### Weather Codes
- 0: Clear ☀️
- 1-3: Partly cloudy ⛅
- 45,48: Fog 🌫️
- 51-55: Drizzle 🌦️
- 61-65: Rain 🌧️
- 71-75: Snow ❄️
- 80-82: Showers 🌧️
- 95: Thunderstorm ⛈️

## Fallback: wttr.in (use ONLY if Open-Meteo fails)

```sh
curl -s "wttr.in/Seoul+Gangseo-gu?format=%c+%t+(feels+like+%f),+wind+%w,+humidity+%h&lang=ko"
```

## Rules

- Always try Open-Meteo first
- If Open-Meteo returns error or empty, fall back to wttr.in
- Include: temperature, feels like, wind, humidity, precipitation chance

## Output Format
Present weather like this example:

**📅 월요일 (3/9) 서울 강서구**
- ☁️ 흐림
- 🌡️ 최고 **8.5°C** / 최저 **0.2°C**
- 🥶 체감 5.6°C ~ -2.8°C
- 💨 바람 10.7km/h
- 🌧️ 강수 없음

Use emoji for weather codes. Bold the temperatures. Keep it concise and pretty.
