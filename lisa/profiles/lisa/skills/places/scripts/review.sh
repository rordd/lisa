#!/bin/sh
# 장소 리뷰 조회
# Usage: review.sh <place_id> [count]
# Source: Google Places API v1

set -eu

if [ -z "${GOOGLE_MAPS_API_KEY:-}" ]; then
  echo '{"error": "GOOGLE_MAPS_API_KEY not set"}' >&2
  exit 1
fi

place_id="$1"
count="${2:-5}"

raw=$(curl -s "https://places.googleapis.com/v1/places/${place_id}" \
  -H "Content-Type: application/json" \
  -H "X-Goog-Api-Key: ${GOOGLE_MAPS_API_KEY}" \
  -H "X-Goog-FieldMask: displayName,rating,userRatingCount,reviews" \
  -H "Accept-Language: ko")

printf '%s' "$raw" | tr -d '\000-\037' | jq --argjson c "$count" '{
  name: .displayName.text,
  rating: .rating,
  totalReviews: .userRatingCount,
  reviews: [.reviews[:$c][] | {
    author: .authorAttribution.displayName,
    rating: .rating,
    text: .text.text,
    time: .relativePublishTimeDescription
  }]
}'
