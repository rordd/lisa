#!/bin/sh
# Google Routes 대중교통 길찾기 (한국)
# Usage: transit.sh <출발지> <도착지>
# Env: GOOGLE_MAPS_API_KEY

set -eu

if [ -z "${GOOGLE_MAPS_API_KEY:-}" ]; then
  echo '{"error": "GOOGLE_MAPS_API_KEY not set"}' >&2
  exit 1
fi

origin="$1"
destination="$2"

payload=$(jq -n --arg o "$origin" --arg d "$destination" '{
  origin: {address: $o},
  destination: {address: $d},
  travelMode: "TRANSIT",
  languageCode: "ko"
}')

curl -s "https://routes.googleapis.com/directions/v2:computeRoutes" \
  -H "Content-Type: application/json" \
  -H "X-Goog-Api-Key: ${GOOGLE_MAPS_API_KEY}" \
  -H "X-Goog-FieldMask: routes.duration,routes.distanceMeters,routes.legs.steps.transitDetails,routes.legs.steps.travelMode,routes.legs.steps.staticDuration,routes.legs.steps.distanceMeters,routes.legs.steps.navigationInstruction" \
  -d "$payload" | jq '{
  mode: "transit",
  duration: .routes[0].duration,
  distance_m: .routes[0].distanceMeters,
  steps: [.routes[0].legs[0].steps[] | select(.transitDetails != null) | {
    line: .transitDetails.transitLine.nameShort,
    name: .transitDetails.transitLine.name,
    color: .transitDetails.transitLine.color,
    from: .transitDetails.stopDetails.departureStop.name,
    to: .transitDetails.stopDetails.arrivalStop.name,
    depart: .transitDetails.localizedValues.departureTime.time.text,
    arrive: .transitDetails.localizedValues.arrivalTime.time.text,
    stops: .transitDetails.stopCount,
    distance_m: .distanceMeters
  }]
}'
