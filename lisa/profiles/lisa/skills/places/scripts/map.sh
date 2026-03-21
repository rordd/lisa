#!/bin/bash
# Google Static Map 이미지 URL 생성
# Usage: map.sh <lat>,<lng> [lat,lng ...] [zoom]
# Env: GOOGLE_MAPS_API_KEY
# Output: {"url": "https://..."}

set -euo pipefail

if [[ -z "${GOOGLE_MAPS_API_KEY:-}" ]]; then
  echo '{"error": "GOOGLE_MAPS_API_KEY not set"}' >&2
  exit 1
fi

coords=()
zoom=14
for arg in "$@"; do
  if [[ "$arg" =~ ^[0-9]+$ ]]; then
    zoom="$arg"
  else
    coords+=("$arg")
  fi
done

if [[ ${#coords[@]} -eq 0 ]]; then
  echo '{"error": "usage: map.sh lat,lng [lat,lng ...] [zoom]"}' >&2
  exit 1
fi

center="${coords[0]}"
markers=""
for c in "${coords[@]}"; do
  markers="${markers}&markers=color:red|${c}"
done

echo "{\"url\": \"https://maps.googleapis.com/maps/api/staticmap?center=${center}&zoom=${zoom}&size=600x400${markers}&key=${GOOGLE_MAPS_API_KEY}\"}"
