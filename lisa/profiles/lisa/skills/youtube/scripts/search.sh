#!/bin/sh
# 유튜브 영상 검색
# Usage: search.sh <검색어> [count] [order]
# order: relevance(관련도), date(최신), viewCount(조회수)
# Source: YouTube Data API v3

set -eu

if [ -z "${GOOGLE_MAPS_API_KEY:-}" ]; then
  echo '{"error": "GOOGLE_MAPS_API_KEY not set"}' >&2
  exit 1
fi

query="$1"
count="${2:-5}"
order="${3:-relevance}"

case "$order" in
  relevance|date|viewCount) ;;
  *) echo '{"error": "invalid order. use: relevance, date, viewCount"}'; exit 1 ;;
esac

encoded=$(printf '%s' "$query" | jq -Rr @uri)

curl -s "https://www.googleapis.com/youtube/v3/search?part=snippet&q=${encoded}&type=video&maxResults=${count}&order=${order}&key=${GOOGLE_MAPS_API_KEY}" | jq '{
  items: [.items[] | {
    title: .snippet.title,
    channel: .snippet.channelTitle,
    description: (.snippet.description | if length > 100 then .[:100] + "..." else . end),
    date: .snippet.publishedAt,
    url: ("https://youtu.be/" + .id.videoId),
    thumbnail: .snippet.thumbnails.medium.url
  }]
}'
