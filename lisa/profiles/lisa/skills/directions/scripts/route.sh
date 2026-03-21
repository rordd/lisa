#!/bin/sh
# Google Directions 글로벌 길찾기
# Usage: route.sh <출발지> <도착지> [mode]
# mode: drive (기본), transit, walk, all
# Env: GOOGLE_MAPS_API_KEY

set -eu

if [ -z "${GOOGLE_MAPS_API_KEY:-}" ]; then
  echo '{"error": "GOOGLE_MAPS_API_KEY not set"}' >&2
  exit 1
fi

origin_raw="$1"
destination_raw="$2"
mode="${3:-drive}"

# Validate mode
case "$mode" in
  drive|transit|walk|all) ;;
  *) echo '{"error":"invalid mode. use: drive, transit, walk, all"}'; exit 1 ;;
esac

origin=$(printf '%s' "$origin_raw" | jq -Rr @uri)
destination=$(printf '%s' "$destination_raw" | jq -Rr @uri)

google_mode() {
  case "$1" in
    drive) echo "driving" ;;
    transit) echo "transit" ;;
    walk) echo "walking" ;;
    *) echo "$1" ;;
  esac
}

fetch_route() {
  gmode=$(google_mode "$1")
  response=$(curl -s "https://maps.googleapis.com/maps/api/directions/json?origin=${origin}&destination=${destination}&mode=${gmode}&language=ko&departure_time=now&key=${GOOGLE_MAPS_API_KEY}")

  status=$(echo "$response" | jq -r '.status')
  if [ "$status" != "OK" ]; then
    echo "{\"mode\": \"$1\", \"error\": \"$status\"}"
    return
  fi

  if [ "$gmode" = "transit" ]; then
    echo "$response" | jq '{
      mode: "transit",
      duration: .routes[0].legs[0].duration.text,
      distance: .routes[0].legs[0].distance.text,
      departure: .routes[0].legs[0].departure_time.text,
      arrival: .routes[0].legs[0].arrival_time.text,
      steps: [.routes[0].legs[0].steps[] | select(.travel_mode == "TRANSIT") | {
        line: .transit_details.line.short_name,
        name: .transit_details.line.name,
        vehicle: .transit_details.line.vehicle.name,
        from: .transit_details.departure_stop.name,
        to: .transit_details.arrival_stop.name,
        depart: .transit_details.departure_time.text,
        arrive: .transit_details.arrival_time.text,
        stops: .transit_details.num_stops,
        duration: .duration.text
      }]
    }'
  else
    echo "$response" | jq --arg m "$1" '{
      mode: $m,
      duration: .routes[0].legs[0].duration.text,
      duration_in_traffic: .routes[0].legs[0].duration_in_traffic.text,
      distance: .routes[0].legs[0].distance.text,
      steps: [.routes[0].legs[0].steps[:5][] | {
        instruction: .html_instructions,
        distance: .distance.text,
        duration: .duration.text
      }]
    }'
  fi
}

if [ "$mode" = "all" ]; then
  drive_result=$(fetch_route "drive")
  transit_result=$(fetch_route "transit")
  echo "[$drive_result, $transit_result]" | jq '.'
else
  fetch_route "$mode"
fi
