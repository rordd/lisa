#!/bin/sh
# weather.sh — Geocoding + Open-Meteo forecast
# Usage: weather.sh [location]
# Default: 강서구

LOCATION="${1:-강서구}"

# Fast path: default location → skip geocoding
if [ "$LOCATION" = "강서구" ] || [ "$LOCATION" = "서울 강서구" ] || [ "$LOCATION" = "Seoul Gangseo-gu" ]; then
    LAT="37.5633"
    LON="126.8214"
    NAME="강서구, 서울특별시, 대한민국"
else
    ENCODED=$(printf '%s' "$LOCATION" | jq -sRr @uri)
    GEO=$(curl -s "https://geocoding-api.open-meteo.com/v1/search?name=${ENCODED}&count=1&language=ko")
    LAT=$(echo "$GEO" | jq -r '.results[0].latitude // empty')
    LON=$(echo "$GEO" | jq -r '.results[0].longitude // empty')
    NAME=$(echo "$GEO" | jq -r '.results[0] | "\(.name), \(.admin1), \(.country)" // empty')
    if [ -z "$LAT" ] || [ -z "$LON" ]; then
        LAT="37.5633"
        LON="126.8214"
        NAME="강서구, 서울특별시, 대한민국 (fallback)"
    fi
fi

WEATHER=$(curl -s "https://api.open-meteo.com/v1/forecast?latitude=${LAT}&longitude=${LON}&current=temperature_2m,apparent_temperature,weather_code,wind_speed_10m,relative_humidity_2m,precipitation&daily=temperature_2m_max,temperature_2m_min,precipitation_probability_max,weather_code&timezone=Asia/Seoul&forecast_days=14")

echo "$WEATHER" | jq --arg name "$NAME" --arg lat "$LAT" --arg lon "$LON" '{
  location: $name,
  coordinates: {latitude: ($lat|tonumber), longitude: ($lon|tonumber)},
  current: .current,
  current_units: .current_units,
  daily: .daily,
  daily_units: .daily_units
}'
