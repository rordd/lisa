#!/bin/bash
# 카카오 주소→좌표 변환
# Usage: geocode.sh <주소 또는 장소명>
# Env: KAKAO_REST_API_KEY
# Output: x,y (경도,위도)

set -euo pipefail

if [[ -z "${KAKAO_REST_API_KEY:-}" ]]; then
  echo '{"error": "KAKAO_REST_API_KEY not set"}' >&2
  exit 1
fi

query=$(python3 -c "import urllib.parse; print(urllib.parse.quote('$1'))")

curl -s "https://dapi.kakao.com/v2/local/search/keyword.json?query=${query}&size=1" \
  -H "Authorization: KakaoAK ${KAKAO_REST_API_KEY}" | jq -r '.documents[0] | "\(.x),\(.y)"'
