#!/bin/sh
# Google Places 장소검색 + 지도
# Usage: search.sh <키워드> [결과수] [nomap]
# Env: GOOGLE_MAPS_API_KEY

set -eu

if [ -z "${GOOGLE_MAPS_API_KEY:-}" ]; then
  echo '{"error": "GOOGLE_MAPS_API_KEY not set"}' >&2
  exit 1
fi

query="$1"
count="${2:-5}"
mode="${3:-}"

# 지도만 모드: 장소 geocode → 지도 URL
if [ "$mode" = "maponly" ]; then
  loc=$(curl -s "https://places.googleapis.com/v1/places:searchText" \
    -H "Content-Type: application/json" \
    -H "X-Goog-Api-Key: ${GOOGLE_MAPS_API_KEY}" \
    -H "X-Goog-FieldMask: places.location,places.displayName" \
    -d "$(jq -cn --arg q "$query" '{textQuery: $q, maxResultCount: 1, languageCode: "ko"}')" | jq '.places[0]')
  lat=$(echo "$loc" | jq -r '.location.latitude')
  lng=$(echo "$loc" | jq -r '.location.longitude')
  name=$(echo "$loc" | jq -r '.displayName.text')
  map_url="https://maps.googleapis.com/maps/api/staticmap?center=${lat},${lng}&zoom=15&size=600x400&maptype=roadmap&markers=color:red|${lat},${lng}&key=${GOOGLE_MAPS_API_KEY}"
  jq -n --arg name "$name" --arg url "$map_url" --argjson lat "$lat" --argjson lng "$lng" '{name: $name, lat: $lat, lng: $lng, map_url: $url}'
  exit 0
fi

results=$(curl -s "https://places.googleapis.com/v1/places:searchText" \
  -H "Content-Type: application/json" \
  -H "X-Goog-Api-Key: ${GOOGLE_MAPS_API_KEY}" \
  -H "X-Goog-FieldMask: places.displayName,places.formattedAddress,places.googleMapsUri,places.location,places.internationalPhoneNumber,places.primaryType,places.rating,places.userRatingCount" \
  -d "$(jq -cn --arg q "$query" --argjson c "$count" '{textQuery: $q, maxResultCount: $c, languageCode: "ko"}')" | jq '[.places // [] | .[] | {
  name: .displayName.text,
  type: .primaryType,
  address: .formattedAddress,
  phone: .internationalPhoneNumber,
  rating: .rating,
  reviews: .userRatingCount,
  url: .googleMapsUri,
  lat: .location.latitude,
  lng: .location.longitude
}]')

# 지도 URL 생성 (nomap이 아닌 경우)
if [ "$mode" != "nomap" ]; then
  markers=$(echo "$results" | jq -r '.[] | "\(.lat),\(.lng)"' | head -5 | \
    awk '{printf "&markers=color:red|%s", $0}')
  center=$(echo "$results" | jq -r '.[0] | "\(.lat),\(.lng)"')
  map_url="https://maps.googleapis.com/maps/api/staticmap?center=${center}&zoom=14&size=600x400&maptype=roadmap${markers}&key=${GOOGLE_MAPS_API_KEY}"
  echo "$results" | jq --arg map "$map_url" '{places: ., map_url: $map}'
else
  echo "$results"
fi
