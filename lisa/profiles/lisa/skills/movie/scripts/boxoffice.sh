#!/bin/sh
# Korean box office rankings via KOBIS API
# Usage: boxoffice.sh [daily|weekly] [YYYYMMDD]
set -e

MODE="${1:-daily}"
TARGET_DT="${2:-$(date -v-1d +%Y%m%d 2>/dev/null || date -d yesterday +%Y%m%d)}"

case "$MODE" in
  daily)
    URL="http://www.kobis.or.kr/kobisopenapi/webservice/rest/boxoffice/searchDailyBoxOfficeList.json?key=${KOBIS_API_KEY}&targetDt=${TARGET_DT}"
    ;;
  weekly)
    URL="http://www.kobis.or.kr/kobisopenapi/webservice/rest/boxoffice/searchWeeklyBoxOfficeList.json?key=${KOBIS_API_KEY}&targetDt=${TARGET_DT}&weekGb=0"
    ;;
  *)
    echo '{"error":"Invalid mode. Use: daily or weekly"}'; exit 1
    ;;
esac

RESP=$(curl -s "$URL")

# Check for error
ERR=$(printf '%s' "$RESP" | jq -r '.faultInfo.message // empty')
if [ -n "$ERR" ]; then
  printf '{"error":"%s"}\n' "$ERR"; exit 1
fi

if [ "$MODE" = "daily" ]; then
  printf '%s' "$RESP" | jq '{
    type: "daily",
    date: .boxOfficeResult.showRange,
    movies: [.boxOfficeResult.dailyBoxOfficeList[] | {
      rank: (.rank | tonumber),
      title: .movieNm,
      audience_today: (.audiCnt | tonumber),
      audience_total: (.audiAcc | tonumber),
      release_date: .openDt,
      new: (.rankOldAndNew == "NEW")
    }]
  }'
else
  printf '%s' "$RESP" | jq '{
    type: "weekly",
    date: .boxOfficeResult.showRange,
    movies: [.boxOfficeResult.weeklyBoxOfficeList[] | {
      rank: (.rank | tonumber),
      title: .movieNm,
      audience_week: (.audiCnt | tonumber),
      audience_total: (.audiAcc | tonumber),
      release_date: .openDt,
      new: (.rankOldAndNew == "NEW")
    }]
  }'
fi
