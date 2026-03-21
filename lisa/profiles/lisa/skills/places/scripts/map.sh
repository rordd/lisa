#!/bin/sh
# Google Static Map 이미지 URL 생성
# Usage: map.sh <lat>,<lng> [lat,lng ...] [zoom]
# Env: GOOGLE_MAPS_API_KEY
# Output: {"url": "https://..."}

set -eu

if [ -z "${GOOGLE_MAPS_API_KEY:-}" ]; then
  echo '{"error": "GOOGLE_MAPS_API_KEY not set"}' >&2
  exit 1
fi

zoom=14
center=""
markers=""
for arg in "$@"; do
  if echo "$arg" | grep -qE '^[0-9]+$'; then
    zoom="$arg"
  else
    if [ -z "$center" ]; then
      center="$arg"
    fi
    markers="${markers}&markers=color:red|${arg}"
  fi
done

if [ -z "$center" ]; then
  echo '{"error": "usage: map.sh lat,lng [lat,lng ...] [zoom]"}' >&2
  exit 1
fi

echo "{\"url\": \"https://maps.googleapis.com/maps/api/staticmap?center=${center}&zoom=${zoom}&size=600x400${markers}&key=${GOOGLE_MAPS_API_KEY}\"}"
