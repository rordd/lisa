#!/bin/sh
# 한국 시장 지수 조회 (코스피/코스닥)
# Usage: market.sh [KOSPI|KOSDAQ|both]
# Source: 네이버 증권 API

set -eu

fetch_index() {
  index="$1"
  data=""
  data=$(curl -s "https://m.stock.naver.com/api/index/${index}/basic")
  echo "$data" | jq '{
    index: .stockName,
    price: .closePrice,
    change: .compareToPreviousClosePrice,
    changePercent: .fluctuationsRatio,
    direction: .compareToPreviousPrice.text,
    status: .marketStatus,
    tradedAt: .localTradedAt
  }'
}

target="${1:-both}"

case "$target" in
  KOSPI|kospi)
    fetch_index "KOSPI"
    ;;
  KOSDAQ|kosdaq)
    fetch_index "KOSDAQ"
    ;;
  both|*)
    echo '['
    fetch_index "KOSPI"
    echo ','
    fetch_index "KOSDAQ"
    echo ']'
    ;;
esac
