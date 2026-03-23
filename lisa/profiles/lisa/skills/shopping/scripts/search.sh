#!/bin/sh
# 네이버 쇼핑 검색
# Usage: search.sh <검색어> [count] [sort]
# sort: sim(관련도), date(날짜), asc(가격낮은순), dsc(가격높은순)
# Source: Naver Shopping API

set -eu

if [ -z "${NAVER_CLIENT_ID:-}" ] || [ -z "${NAVER_CLIENT_SECRET:-}" ]; then
  echo '{"error": "NAVER_CLIENT_ID/SECRET not set"}' >&2
  exit 1
fi

query="$1"
count="${2:-5}"
sort="${3:-sim}"

# Validate sort
case "$sort" in
  sim|date|asc|dsc) ;;
  *) echo '{"error": "invalid sort. use: sim, date, asc, dsc"}'; exit 1 ;;
esac

encoded=$(printf '%s' "$query" | jq -Rr @uri)

curl -s "https://openapi.naver.com/v1/search/shop.json?query=${encoded}&display=${count}&sort=${sort}" \
  -H "X-Naver-Client-Id: ${NAVER_CLIENT_ID}" \
  -H "X-Naver-Client-Secret: ${NAVER_CLIENT_SECRET}" | jq '{
  total: .total,
  items: [.items[] | {
    name: (.title | gsub("<[^>]*>"; "")),
    price: .lprice,
    mall: .mallName,
    brand: .brand,
    category: ((.category1 // "") + " > " + (.category2 // "") + " > " + (.category3 // "")),
    url: .link,
    image: .image
  }]
}'
