#!/bin/bash
# 종목 뉴스 조회
# Usage: news.sh <종목코드> [개수]
# Source: 네이버 증권 API

set -euo pipefail

code="$1"
count="${2:-5}"

curl -s "https://m.stock.naver.com/api/news/stock/${code}?page=1&pageSize=${count}" | jq '[.[].items[] | {
  title: (.title | gsub("&quot;"; "\"")),
  source: .officeName,
  date: .datetime,
  url: ("https://n.news.naver.com/mnews/article/" + .officeId + "/" + .articleId)
}]'
