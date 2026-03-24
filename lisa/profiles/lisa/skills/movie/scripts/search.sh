#!/bin/sh
# Search movies by title via KOBIS API
# Usage: search.sh "영화제목"
set -e

QUERY="$1"
if [ -z "$QUERY" ]; then
  echo '{"error":"Usage: search.sh <movie_title>"}'; exit 1
fi

ENCODED=$(printf '%s' "$QUERY" | jq -Rr @uri)

URL="http://www.kobis.or.kr/kobisopenapi/webservice/rest/movie/searchMovieList.json?key=${KOBIS_API_KEY}&movieNm=${ENCODED}&itemPerPage=10"

RESP=$(curl -s "$URL")

ERR=$(printf '%s' "$RESP" | jq -r '.faultInfo.message // empty')
if [ -n "$ERR" ]; then
  printf '{"error":"%s"}\n' "$ERR"; exit 1
fi

printf '%s' "$RESP" | jq '{
  query: "'"$QUERY"'",
  total: (.movieListResult.totCnt | tonumber),
  movies: [.movieListResult.movieList[:10][] | {
    code: .movieCd,
    title: .movieNm,
    title_en: .movieNmEn,
    release_date: .openDt,
    genre: .genreAlt,
    director: (.directors | map(.peopleNm) | join(", ")),
    nation: .nationAlt,
    type: .typeNm
  }]
}'
