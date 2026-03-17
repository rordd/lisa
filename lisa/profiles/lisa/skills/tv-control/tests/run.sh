#!/bin/bash
# TV Control 스킬 E2E 벤치마크 테스트
# Usage: bash tests/run.sh [RESULT_DIR] [RUNS]
#
# 3개 시나리오 비교:
#   S1: SKILL.toml (native tool calling)
#   S2: SKILL.md always=true (system prompt injection)
#   S3: SKILL.md always=false (on-demand via read_skill)
#
# 성공 판정: mock-tv.sh 실제 호출 여부 (mock_calls > 0)
set +e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SKILL_DIR="$(dirname "$SCRIPT_DIR")"
REPO_DIR="$(cd "$SKILL_DIR/../../../../.." && pwd)"
ENV_FILE="$REPO_DIR/.env"
ZEROCLAW="${ZEROCLAW:-$(which zeroclaw 2>/dev/null || echo "$REPO_DIR/target/debug/zeroclaw")}"
RESULT_DIR="${1:-/tmp/tv-skill-benchmark}"
RUNS="${2:-10}"
PROMPTS=(
    "TV 볼륨을 8로 설정해줘"
    "TV 볼륨을 10으로 설정해줘"
)

mkdir -p "$RESULT_DIR"
echo "scenario,run,resp_chars,tool_exec,success,fail,mock_calls,blocked,h_tool,h_fake,h_result" > "$RESULT_DIR/summary.csv"

load_env() {
    cd "$REPO_DIR"
    export $(grep -v '^#' "$ENV_FILE" | xargs)
}

reset_memory() {
    rm -f ~/.zeroclaw/workspace/memory/*.md ~/.zeroclaw/workspace/MEMORY.md ~/.zeroclaw/brain.db 2>/dev/null
}

run_test() {
    local scenario="$1"
    local run="$2"
    local outfile="$RESULT_DIR/${scenario}_run${run}.txt"

    # mock 로그만 매 Run 리셋 (메모리는 시나리오 시작 시 1회만)
    > /tmp/mock-tv-calls.log

    # 프롬프트 교차 (홀수=8, 짝수=10)
    local idx=$(( (run - 1) % ${#PROMPTS[@]} ))
    local msg="${PROMPTS[$idx]}"

    load_env
    RUST_LOG=debug "$ZEROCLAW" agent -m "$msg" > "$outfile" 2>&1

    sleep 2

    local toml_loaded; toml_loaded=$(grep -c "TOML skill loaded.*tv-control" "$outfile") || toml_loaded=0
    local native_reg; native_reg=$(grep -c "Registered skill tool.*tv" "$outfile") || native_reg=0
    local tool_exec; tool_exec=$(grep -c "Tool call executing" "$outfile") || tool_exec=0
    local clean_file; clean_file=$(sed 's/\x1b\[[0-9;]*m//g' "$outfile")
    local success_count; success_count=$(echo "$clean_file" | grep -c "success=true") || success_count=0
    local fail_count; fail_count=$(echo "$clean_file" | grep -c "success=false") || fail_count=0
    local mock_calls; mock_calls=$(wc -l < /tmp/mock-tv-calls.log 2>/dev/null | tr -d ' ') || mock_calls=0
    local blocked; blocked=$(echo "$clean_file" | grep -c "blocked by security\|Blocked by security") || blocked=0
    local resp_chars; resp_chars=$(wc -c < "$outfile" | tr -d ' ')

    # Hallucination 감지
    # 1. 존재하지 않는 tool 호출 (tv_ 로 시작하지만 등록된 8개가 아닌 것)
    local halluc_tool; halluc_tool=$(echo "$clean_file" | grep "Tool call executing" | grep -v "tv_launch_app\|tv_get_foreground\|tv_channel_up\|tv_channel_down\|tv_go_to_channel\|tv_volume_up\|tv_volume_down\|tv_set_volume\|shell\|read_skill\|pwd\|memory" | wc -l | tr -d ' ') || halluc_tool=0
    # 2. tool 안 부르고 성공한 척 (exec=0인데 응답에 "설정" "완료" 등)
    local fake_success=0
    if [ "$tool_exec" -eq 0 ] && [ "$mock_calls" -eq 0 ]; then
        fake_success=$(echo "$clean_file" | grep -ci "설정했\|완료\|변경했\|맞춰\|볼륨.*[0-9]") || fake_success=0
        [ "$fake_success" -gt 0 ] && fake_success=1
    fi
    # 3. mock 안 불렸는데 성공 주장
    local fake_result=0
    if [ "$mock_calls" -eq 0 ] && [ "$tool_exec" -gt 0 ]; then
        fake_result=1
    fi

    printf "  Run %d: exec=%d ok=%d fail=%d mock=%d h_tool=%d h_fake=%d h_result=%d\n" \
        "$run" "$tool_exec" "$success_count" "$fail_count" "$mock_calls" "$halluc_tool" "$fake_success" "$fake_result"
    echo "$scenario,$run,$resp_chars,$tool_exec,$success_count,$fail_count,$mock_calls,$blocked,$halluc_tool,$fake_success,$fake_result" >> "$RESULT_DIR/summary.csv"

    sleep 2
}

echo "========================================"
echo " TV Control 스킬 E2E 벤치마크"
load_env
echo " 모델: ${ZEROCLAW_MODEL:-unknown}"
echo " 각 시나리오 ${RUNS}회"
echo "========================================"
echo ""

# Setup: 백업 + trap으로 kill 시에도 원복 보장
cp "$SKILL_DIR/SKILL.toml" "$SKILL_DIR/SKILL.toml.orig" 2>/dev/null || true
cp "$SKILL_DIR/SKILL.md" "$SKILL_DIR/SKILL.md.orig" 2>/dev/null || true

cleanup() {
    echo "▶ 원복 및 정리..."
    [ -f "$SKILL_DIR/SKILL.toml.orig" ] && cp "$SKILL_DIR/SKILL.toml.orig" "$SKILL_DIR/SKILL.toml" && rm -f "$SKILL_DIR/SKILL.toml.orig"
    [ -f "$SKILL_DIR/SKILL.md.orig" ] && cp "$SKILL_DIR/SKILL.md.orig" "$SKILL_DIR/SKILL.md" && rm -f "$SKILL_DIR/SKILL.md.orig"
    rm -f /tmp/mock-tv-calls.log
}
trap cleanup EXIT INT TERM

# ========== S1: SKILL.toml (mock) ==========
echo "▶ S1: SKILL.toml (native tool call)"
cp "$SCRIPT_DIR/SKILL.toml.mock" "$SKILL_DIR/SKILL.toml"
rm -f "$SKILL_DIR/SKILL.md"
reset_memory
for i in $(seq 1 $RUNS); do run_test "s1_toml" "$i"; done
echo ""

# ========== S2: SKILL.md always=true (mock) ==========
echo "▶ S2: SKILL.md always=true (system prompt)"
rm -f "$SKILL_DIR/SKILL.toml"
cp "$SCRIPT_DIR/SKILL.md.mock" "$SKILL_DIR/SKILL.md"
sed -i '' 's/always: false/always: true/' "$SKILL_DIR/SKILL.md" 2>/dev/null
grep "always" "$SKILL_DIR/SKILL.md" | head -1
reset_memory
for i in $(seq 1 $RUNS); do run_test "s2_md_always" "$i"; done
echo ""

# ========== S3: SKILL.md always=false (mock) ==========
echo "▶ S3: SKILL.md always=false (on-demand)"
rm -f "$SKILL_DIR/SKILL.toml"
cp "$SCRIPT_DIR/SKILL.md.mock" "$SKILL_DIR/SKILL.md"
sed -i '' 's/always: true/always: false/' "$SKILL_DIR/SKILL.md" 2>/dev/null
grep "always" "$SKILL_DIR/SKILL.md" | head -1
reset_memory
for i in $(seq 1 $RUNS); do run_test "s3_md_ondemand" "$i"; done
echo ""

# Cleanup은 trap에서 자동 실행

echo ""
echo "========================================"
echo " 결과 요약 (mock>0 = 실제 성공)"
echo "========================================"
column -t -s',' "$RESULT_DIR/summary.csv"

echo ""
echo "--- 시나리오별 성공률 ---"
for s in s1_toml s2_md_always s3_md_ondemand; do
    total=$(grep "^${s}," "$RESULT_DIR/summary.csv" | wc -l | tr -d ' ')
    mock_ok=$(grep "^${s}," "$RESULT_DIR/summary.csv" | awk -F',' '$7>0' | wc -l | tr -d ' ')
    h_tool=$(grep "^${s}," "$RESULT_DIR/summary.csv" | awk -F',' '{s+=$9} END{print s+0}')
    h_fake=$(grep "^${s}," "$RESULT_DIR/summary.csv" | awk -F',' '{s+=$10} END{print s+0}')
    h_result=$(grep "^${s}," "$RESULT_DIR/summary.csv" | awk -F',' '{s+=$11} END{print s+0}')
    echo "  $s: mock=$mock_ok/$total | halluc: tool=$h_tool fake=$h_fake result=$h_result"
done

echo ""
echo "상세: $RESULT_DIR/"
