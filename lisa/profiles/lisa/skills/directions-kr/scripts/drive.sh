#!/bin/sh
# 카카오 모빌리티 자동차 길찾기
# Usage: drive.sh <출발지> <도착지>
# 입력: 주소/장소명 또는 좌표(lon,lat)
# Env: KAKAO_REST_API_KEY

set -eu

if [ -z "${KAKAO_REST_API_KEY:-}" ]; then
  echo '{"error": "KAKAO_REST_API_KEY not set"}' >&2
  exit 1
fi

geocode() {
  input="$1"
  # 이미 좌표면 그대로
  if echo "$input" | grep -qE '^[0-9]+\.[0-9]+,[0-9]+\.[0-9]+$'; then
    echo "$input"
    return
  fi
  query=$(printf '%s' "$input" | jq -Rr @uri)
  result=$(curl -s "https://dapi.kakao.com/v2/local/search/keyword.json?query=${query}&size=1" \
    -H "Authorization: KakaoAK ${KAKAO_REST_API_KEY}" | jq -r '.documents[0] | "\(.x),\(.y)"')
  if [ "$result" = "null,null" ] || [ -z "$result" ]; then
    echo '{"error": "geocode failed"}' >&2
    return 1
  fi
  echo "$result"
}

origin=$(geocode "$1")
destination=$(geocode "$2")

curl -s "https://apis-navi.kakaomobility.com/v1/directions?origin=${origin}&destination=${destination}&priority=RECOMMEND" \
  -H "Authorization: KakaoAK ${KAKAO_REST_API_KEY}" | jq '{
  mode: "drive",
  result: .routes[0].result_msg,
  distance_m: .routes[0].summary.distance,
  duration_s: .routes[0].summary.duration,
  duration_min: (.routes[0].summary.duration / 60 | floor),
  taxi_fare: .routes[0].summary.fare.taxi,
  toll: .routes[0].summary.fare.toll,
  traffic_summary: (
    [.routes[0].sections[].roads[] | {s: .traffic_state, d: .distance}] |
    group_by(.s) | map({
      state: (if .[0].s == 0 then "정보없음" elif .[0].s == 1 then "막힘" elif .[0].s == 2 then "지체" elif .[0].s == 3 then "서행" elif .[0].s == 4 then "원활" elif .[0].s == 6 then "졸음주의" else "기타" end),
      distance_m: (map(.d) | add)
    }) | sort_by(-.distance_m)
  ),
  traffic_roads: [.routes[0].sections[].roads[] | select(.traffic_state <= 2) | {
    name, distance_m: .distance, speed_kmh: .traffic_speed,
    state: (if .traffic_state == 1 then "막힘" elif .traffic_state == 2 then "지체" else "정보없음" end)
  }],
  guides: [.routes[0].sections[].guides[] | select(.type != 0) | {
    name: .name,
    guidance: .guidance,
    distance_m: .distance,
    duration_s: .duration
  }]
}'
