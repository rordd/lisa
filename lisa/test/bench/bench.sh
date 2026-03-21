#!/bin/bash
# Lisa Skill Benchmark Tool
# Usage: bench.sh [test_file]
# Default: bench.sh tests.json
#
# tests.json format:
# [
#   {"skill": "directions-kr", "query": "강남역에서 서울역 어떻게 가?"},
#   {"skill": "stock", "query": "삼성전자 주가"}
# ]
#
# Requires: websocat, jq
# RUST_LOG=debug for profiling logs

set -euo pipefail

WS_URL="${WS_URL:-ws://localhost:42617/ws/chat}"
LOG_FILE="${LOG_FILE:-/tmp/lisa-debug.log}"
TEST_FILE="${1:-$(dirname "$0")/tests.json}"
RESULTS_FILE="/tmp/bench-results.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

if ! command -v websocat &>/dev/null; then
  echo "websocat 필요: brew install websocat"
  exit 1
fi

if [[ ! -f "$TEST_FILE" ]]; then
  echo "테스트 파일 없음: $TEST_FILE"
  exit 1
fi

total=$(jq length "$TEST_FILE")
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}  Lisa Skill Benchmark  ($total tests)${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

results="[]"

for i in $(seq 0 $((total - 1))); do
  skill=$(jq -r ".[$i].skill" "$TEST_FILE")
  query=$(jq -r ".[$i].query" "$TEST_FILE")
  
  echo -e "${YELLOW}[$((i+1))/$total]${NC} ${skill}: ${query}"
  
  # Mark log position
  log_start=$(wc -l < "$LOG_FILE")
  
  # Send WS message and wait for done/error
  start_ms=$(python3 -c "import time; print(int(time.time()*1000))")
  
  ws_response=$(echo "{\"type\":\"message\",\"content\":\"$query\"}" | \
    websocat -t "$WS_URL" 2>/dev/null | \
    grep -m1 '"type":"done"\|"type":"error"' || echo '{"type":"error","message":"timeout"}')
  
  end_ms=$(python3 -c "import time; print(int(time.time()*1000))")
  wall_ms=$((end_ms - start_ms))
  
  # Parse response
  resp_type=$(echo "$ws_response" | jq -r '.type // "error"')
  
  # Parse profiling from log
  log_tail=$(tail -n +$((log_start + 1)) "$LOG_FILE")
  
  # Strip ANSI codes for reliable parsing
  clean_log=$(echo "$log_tail" | sed 's/\x1b\[[0-9;]*m//g')
  
  iterations=$(echo "$clean_log" | grep "agent turn complete" | grep -o "iterations=[0-9]*" | tail -1 | grep -o "[0-9]*" || echo "0")
  tool_calls=$(echo "$clean_log" | grep "agent turn complete" | grep -o "total_tool_calls=[0-9]*" | tail -1 | grep -o "[0-9]*" || echo "0")
  loop_ms=$(echo "$clean_log" | grep "agent turn complete" | grep -o "total_ms=[0-9]*" | tail -1 | grep -o "[0-9]*" || echo "0")
  
  # Per-iteration breakdown
  iter_details=$(echo "$clean_log" | grep "agent turn iteration" | \
    sed 's/.*iteration=\([0-9]*\).*tool_count=\([0-9]*\).*tools=\[\([^]]*\)\].*/  #\1: \2 calls [\3]/' || echo "")
  
  # Tool names used
  tools_used=$(echo "$clean_log" | grep "tool result" | \
    sed 's/.*tool=\([^ ]*\).*/\1/' | sort | uniq -c | sort -rn | \
    awk '{printf "%s×%s ", $2, $1}' || echo "")
  
  # Status
  if [[ "$resp_type" == "done" ]]; then
    status="${GREEN}✅${NC}"
  else
    status="${RED}❌${NC}"
    error_msg=$(echo "$ws_response" | jq -r '.message // "unknown"')
  fi
  
  echo -e "  $status wall=${wall_ms}ms loop=${loop_ms}ms iter=${iterations} tools=${tool_calls}"
  if [[ -n "$iter_details" ]]; then
    echo "$iter_details"
  fi
  if [[ -n "$tools_used" ]]; then
    echo -e "  🔧 $tools_used"
  fi
  echo ""
  
  # Ensure numeric defaults
  : "${loop_ms:=0}"
  : "${iterations:=0}"
  : "${tool_calls:=0}"
  [[ "$loop_ms" =~ ^[0-9]+$ ]] || loop_ms=0
  [[ "$iterations" =~ ^[0-9]+$ ]] || iterations=0
  [[ "$tool_calls" =~ ^[0-9]+$ ]] || tool_calls=0

  # Collect result
  results=$(echo "$results" | jq \
    --arg skill "$skill" \
    --arg query "$query" \
    --arg status "$resp_type" \
    --argjson wall_ms "$wall_ms" \
    --argjson loop_ms "$loop_ms" \
    --argjson iterations "$iterations" \
    --argjson tool_calls "$tool_calls" \
    --arg tools "$tools_used" \
    '. + [{skill: $skill, query: $query, status: $status, wall_ms: $wall_ms, loop_ms: $loop_ms, iterations: $iterations, tool_calls: $tool_calls, tools: $tools}]')
  
  # Reset session between tests
  sleep 1
done

# Summary
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}  Summary${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

echo "$results" | jq -r '.[] | "\(.skill // "-")\t\(.status // "-")\t\(.wall_ms // 0)ms\t\(.iterations // 0)iter\t\(.tool_calls // 0)calls"' | \
  column -t -s $'\t'

avg_wall=$(echo "$results" | jq '[.[].wall_ms // 0] | add / length | floor')
avg_tools=$(echo "$results" | jq '[.[].tool_calls // 0] | add / length * 10 | floor / 10')
total_ok=$(echo "$results" | jq '[.[] | select(.status == "done")] | length')

echo ""
echo -e "  Pass: ${GREEN}${total_ok}${NC}/${total}"
echo -e "  Avg wall: ${avg_wall}ms"
echo -e "  Avg tools: ${avg_tools}"

# Save results
echo "$results" | jq '.' > "$RESULTS_FILE"
echo -e "  Results: ${RESULTS_FILE}"
