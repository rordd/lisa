#!/bin/sh
# Movie detail info via KOBIS API
# Usage: detail.sh "영화코드"
set -e

MOVIE_CD="$1"
if [ -z "$MOVIE_CD" ]; then
  echo '{"error":"Usage: detail.sh <movie_code>"}'; exit 1
fi

URL="http://www.kobis.or.kr/kobisopenapi/webservice/rest/movie/searchMovieInfo.json?key=${KOBIS_API_KEY}&movieCd=${MOVIE_CD}"

RESP=$(curl -s "$URL")

ERR=$(printf '%s' "$RESP" | jq -r '.faultInfo.message // empty')
if [ -n "$ERR" ]; then
  printf '{"error":"%s"}\n' "$ERR"; exit 1
fi

printf '%s' "$RESP" | jq '.movieInfoResult.movieInfo | {
  code: .movieCd,
  title: .movieNm,
  title_en: .movieNmEn,
  release_date: (.openDt // ""),
  runtime: (.showTm + "분"),
  genre: ([.genres[]?.genreNm] | join(", ")),
  nation: ([.nations[]?.nationNm] | join(", ")),
  directors: [.directors[]? | {name: .peopleNm, name_en: .peopleNmEn}],
  actors: [.actors[]? | {name: .peopleNm, role: .cast}][:10],
  rating: ([.audits[]?.watchGradeNm] | join(", ")),
  type: .typeNm,
  companies: [.companys[]? | {name: .companyNm, role: .companyPartNm}][:5]
}'
