#!/bin/bash
# Google Places 장소검색
# Usage: search.sh <키워드> [결과수]
# Env: GOOGLE_MAPS_API_KEY

set -euo pipefail

if [[ -z "${GOOGLE_MAPS_API_KEY:-}" ]]; then
  echo '{"error": "GOOGLE_MAPS_API_KEY not set"}' >&2
  exit 1
fi

query="$1"
count="${2:-5}"

curl -s "https://places.googleapis.com/v1/places:searchText" \
  -H "Content-Type: application/json" \
  -H "X-Goog-Api-Key: ${GOOGLE_MAPS_API_KEY}" \
  -H "X-Goog-FieldMask: places.displayName,places.formattedAddress,places.googleMapsUri,places.location,places.internationalPhoneNumber,places.primaryType,places.rating,places.userRatingCount" \
  -d "{\"textQuery\": \"${query}\", \"maxResultCount\": ${count}, \"languageCode\": \"ko\"}" | jq '[.places // [] | .[] | {
  name: .displayName.text,
  type: .primaryType,
  address: .formattedAddress,
  phone: .internationalPhoneNumber,
  rating: .rating,
  reviews: .userRatingCount,
  url: .googleMapsUri,
  lat: .location.latitude,
  lng: .location.longitude
}]'
