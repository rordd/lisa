#!/bin/sh
# 한국 길찾기 통합
# Usage: all.sh <출발지> <도착지> [mode]
# mode: all (기본), drive, transit
# Env: KAKAO_REST_API_KEY, GOOGLE_MAPS_API_KEY

set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
origin="$1"
destination="$2"
mode="${3:-drive}"

# Validate mode
case "$mode" in
  drive|transit|all) ;;
  *) echo '{"error":"invalid mode. use: drive, transit, all"}'; exit 1 ;;
esac

results="[]"

if [ "$mode" = "all" ] || [ "$mode" = "drive" ]; then
  drive=$("$SCRIPT_DIR/drive.sh" "$origin" "$destination" 2>/dev/null || echo '{"error":"drive failed"}')
  results=$(echo "$results" | jq --argjson d "$drive" '. + [$d]')
fi

if [ "$mode" = "all" ] || [ "$mode" = "transit" ]; then
  transit=$("$SCRIPT_DIR/transit.sh" "$origin" "$destination" 2>/dev/null || echo '{"error":"transit failed"}')
  results=$(echo "$results" | jq --argjson t "$transit" '. + [$t]')
fi

echo "$results" | jq '.'
