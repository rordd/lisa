#!/bin/bash
# 관심종목 관리
# Usage:
#   watchlist.sh                  — 전체 조회 (시세 포함)
#   watchlist.sh add <코드|이름>  — 추가
#   watchlist.sh remove <코드|이름> — 삭제
#   watchlist.sh list             — 목록만 (시세 없이)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="$(dirname "$SCRIPT_DIR")/data"
WATCHLIST="$DATA_DIR/watchlist.json"

mkdir -p "$DATA_DIR"
[[ -f "$WATCHLIST" ]] || echo '[]' > "$WATCHLIST"

resolve_code() {
  local input="$1"
  if [[ "$input" =~ ^[0-9]{6}$ ]]; then
    echo "$input"
    return
  fi
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

resolve_name() {
  local input="$1"
  if [[ "$input" =~ ^[0-9]{6}$ ]]; then
    local result
    result=$(curl -s "https://m.stock.naver.com/api/stock/${input}/basic")
    echo "$result" | jq -r '.stockName // empty'
  else
    echo "$input"
  fi
}

cmd="${1:-}"

case "$cmd" in
  add)
    input="${2:?종목코드 또는 이름을 입력하세요}"
    code=$(resolve_code "$input") || exit 1
    name=$(resolve_name "$code")
    # 중복 체크
    exists=$(jq --arg c "$code" '[.[] | select(.code == $c)] | length' "$WATCHLIST")
    if [[ "$exists" -gt 0 ]]; then
      echo "{\"status\": \"already_exists\", \"code\": \"$code\", \"name\": \"$name\"}"
      exit 0
    fi
    jq --arg c "$code" --arg n "$name" '. + [{"code": $c, "name": $n}]' "$WATCHLIST" > "$WATCHLIST.tmp" && mv "$WATCHLIST.tmp" "$WATCHLIST"
    echo "{\"status\": \"added\", \"code\": \"$code\", \"name\": \"$name\"}"
    ;;
  remove|rm|del)
    input="${2:?종목코드 또는 이름을 입력하세요}"
    code=$(resolve_code "$input") || exit 1
    name=$(resolve_name "$code")
    jq --arg c "$code" '[.[] | select(.code != $c)]' "$WATCHLIST" > "$WATCHLIST.tmp" && mv "$WATCHLIST.tmp" "$WATCHLIST"
    echo "{\"status\": \"removed\", \"code\": \"$code\", \"name\": \"$name\"}"
    ;;
  list)
    cat "$WATCHLIST"
    ;;
  *)
    # 기본: 전체 관심종목 시세 조회
    codes=$(jq -r '.[].code' "$WATCHLIST")
    if [[ -z "$codes" ]]; then
      echo '{"status": "empty", "message": "관심종목이 없습니다. watchlist.sh add <종목> 으로 추가하세요."}'
      exit 0
    fi
    # quote.sh로 전체 조회
    # shellcheck disable=SC2086
    "$SCRIPT_DIR/quote.sh" $codes
    ;;
esac
