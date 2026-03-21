#!/bin/bash
# 한국 주식 시세 조회
# Usage: quote.sh <종목코드 또는 종목명> [종목코드2 ...]
# Source: 네이버 증권 API

set -euo pipefail

resolve_code() {
  local input="$1"
  # 숫자 6자리면 코드로 간주
  if [[ "$input" =~ ^[0-9]{6}$ ]]; then
    echo "$input"
    return
  fi
  # 종목명 → 코드 검색
  local result
  result=$(curl -s "https://ac.stock.naver.com/ac?q=$(python3 -c "import urllib.parse; print(urllib.parse.quote('$input'))")&target=stock&st=111&r_lt=111&q_enc=utf-8")
  local code
  code=$(echo "$result" | jq -r '.items[0].code // empty')
  if [[ -z "$code" ]]; then
    echo "ERROR: '$input' 종목을 찾을 수 없습니다" >&2
    return 1
  fi
  echo "$code"
}

fetch_quote() {
  local code="$1"
  local data
  data=$(curl -s "https://m.stock.naver.com/api/stock/${code}/basic")

  # jq로 필요한 필드만 추출
  echo "$data" | jq '{
    code: .itemCode,
    name: .stockName,
    market: .stockExchangeType.nameKor,
    price: .closePrice,
    change: .compareToPreviousClosePrice,
    changePercent: .fluctuationsRatio,
    direction: .compareToPreviousPrice.text,
    status: .marketStatus,
    tradedAt: .localTradedAt,
    charts: {
      day: .imageChartUrlInfo.line.day,
      month3: .imageChartUrlInfo.line.month3,
      year: .imageChartUrlInfo.line.year,
      candleDay: .imageChartUrlInfo.candle.day,
      candleWeek: .imageChartUrlInfo.candle.week
    }
  }'
}

if [[ $# -eq 0 ]]; then
  echo "Usage: quote.sh <종목코드|종목명> [종목코드|종목명 ...]" >&2
  exit 1
fi

results="[]"
for input in "$@"; do
  code=$(resolve_code "$input") || continue
  quote=$(fetch_quote "$code")
  results=$(echo "$results" | jq --argjson q "$quote" '. + [$q]')
done

# 단일 종목이면 배열 벗기기
if [[ $# -eq 1 ]]; then
  echo "$results" | jq '.[0]'
else
  echo "$results"
fi
