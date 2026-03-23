#!/bin/sh
# 네이버 뉴스 검색
# Usage: search.sh <검색어> [count] [sort]
# sort: sim(관련도), date(최신)
# Source: Naver News Search API

set -eu

if [ -z "${NAVER_CLIENT_ID:-}" ] || [ -z "${NAVER_CLIENT_SECRET:-}" ]; then
  echo '{"error": "NAVER_CLIENT_ID/SECRET not set"}' >&2
  exit 1
fi

query="$1"
count="${2:-5}"
sort="${3:-date}"

case "$sort" in
  sim|date) ;;
  *) echo '{"error": "invalid sort. use: sim, date"}'; exit 1 ;;
esac

encoded=$(printf '%s' "$query" | jq -Rr @uri)

curl -s "https://openapi.naver.com/v1/search/news.json?query=${encoded}&display=${count}&sort=${sort}" \
  -H "X-Naver-Client-Id: ${NAVER_CLIENT_ID}" \
  -H "X-Naver-Client-Secret: ${NAVER_CLIENT_SECRET}" | jq '{
  total: .total,
  items: [.items[] | {
    title: (.title | gsub("<[^>]*>"; "")),
    description: (.description | gsub("<[^>]*>"; "")),
    url: .originallink,
    source: (.link | split("/")[2] // ""),
    date: .pubDate
  }]
}'
