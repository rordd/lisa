#!/usr/bin/env bash
set -euo pipefail

# ─────────────────────────────────────────────────────────────
# benchmark-skill-mode.sh
#
# Measures response time and LLM turn count of weather and
# tv-control skills in SKILL.md mode vs SKILL.toml mode on a
# remote target.
#
# Measurement method: POST /api/chat to the running daemon gateway
# (127.0.0.1:42617). This reflects actual user-facing response time
# (no binary startup overhead), matching the Telegram channel experience.
#
# LLM turn count: measured via runtime trace JSONL file on target.
# Runtime trace is temporarily enabled in config.toml for the duration
# of the benchmark and restored to original state afterwards.
#
# Usage:
#   benchmark-skill-mode.sh [--target <IP>] [--runs <N>]
#
# Prerequisites:
#   - zeroclaw binary already installed on target
#   - .env in ~/.zeroclaw/.env on target (all vars must be exported)
#   - SSH key auth to target
# ─────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ONBOARD="$SCRIPT_DIR/onboard.sh"

TARGET_IP="192.168.0.10"
TARGET_USER="root"
RUNS=10
RESULTS_FILE="/tmp/zeroclaw_benchmark_$$.txt"
GATEWAY_PORT=42617

# Queries designed to force a tool call on every run:
#   - weather: explicitly requests live API data (not inferable from context)
#   - tv-control: alternates between vol=8 and vol=10 to force tool call every run
WEATHER_QUERY="Open-Meteo API를 지금 직접 호출해서 서울 현재 기온과 날씨 상태를 한 줄로 알려줘"
TV_QUERY_A="TV 볼륨을 8로 설정해줘"
TV_QUERY_B="TV 볼륨을 10으로 설정해줘"

# ── Parse args ──
while [[ $# -gt 0 ]]; do
    case "$1" in
        --target) TARGET_IP="$2"; shift 2 ;;
        --runs)   RUNS="$2"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

TARGET_HOST="${TARGET_USER}@${TARGET_IP}"
DEPLOY_DIR="/home/${TARGET_USER}/lisa"
ZC_DIR="/home/${TARGET_USER}/.zeroclaw"
WS="${ZC_DIR}/workspace"
GATEWAY_URL="http://127.0.0.1:${GATEWAY_PORT}"
TRACE_FILE="${WS}/state/runtime-trace.jsonl"

# Global vars set by run_measurement()
_LAST_MS=0
_LAST_TURNS=0

# ─────────────────────────────────────────────────────────────
# Runtime Trace helpers (enable/disable on target config)
# ─────────────────────────────────────────────────────────────

enable_runtime_trace() {
    echo "  [trace] Enabling runtime trace on target config..."
    ssh "$TARGET_HOST" "
        cp ${ZC_DIR}/config.toml ${ZC_DIR}/config.toml.bak
        printf '\n[observability]\nbackend = \"none\"\nruntime_trace_mode = \"rolling\"\nruntime_trace_max_entries = 500\n' \
            >> ${ZC_DIR}/config.toml
        mkdir -p ${WS}/state
    "
    echo "  [trace] Enabled (backup saved as config.toml.bak)"
}

disable_runtime_trace() {
    echo "  [trace] Restoring original target config..."
    ssh "$TARGET_HOST" "
        [ -f ${ZC_DIR}/config.toml.bak ] \
            && mv ${ZC_DIR}/config.toml.bak ${ZC_DIR}/config.toml \
            || true
    "
    echo "  [trace] Config restored"
}

# ─────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────

# Clear skills and stop daemon (non-interactive)
clear_skills() {
    echo "  [clear] Stopping zeroclaw and clearing skills..."
    ssh "$TARGET_HOST" "
        pidof zeroclaw >/dev/null 2>&1 && kill -9 \$(pidof zeroclaw) 2>/dev/null || true
        sleep 1
        rm -rf ${WS}/skills
        mkdir -p ${WS}/skills
    "
    echo "  [clear] Done"
}

# Deploy skills to target (all files including SKILL.toml)
deploy_skills() {
    echo "  [deploy] Deploying skills..."
    bash "$ONBOARD" --skills --target "$TARGET_IP" 2>&1 | sed 's/^/    /'
}

# Remove SKILL.toml from deployed skills to activate SKILL.md mode
activate_skill_md_mode() {
    echo "  [mode] Removing SKILL.toml from target (SKILL.md mode)..."
    ssh "$TARGET_HOST" "rm -f ${WS}/skills/weather/SKILL.toml ${WS}/skills/tv-control/SKILL.toml"
    echo "  [mode] SKILL.md mode active"
}

# Start daemon on target
start_daemon() {
    echo "  [daemon] Starting zeroclaw daemon..."
    local hosts_copy="/home/${TARGET_USER}/.hosts"
    ssh "$TARGET_HOST" "
        [ -f $hosts_copy ] && ! grep -q '10.182.173.75' /etc/hosts 2>/dev/null \
            && mount --bind $hosts_copy /etc/hosts 2>/dev/null || true
        cd $DEPLOY_DIR
        [ -f $ZC_DIR/.env ] && . $ZC_DIR/.env
        export PATH=$DEPLOY_DIR:\$PATH
        export ZEROCLAW_CONFIG_DIR=$ZC_DIR
        nohup ./zeroclaw daemon > /tmp/zeroclaw.log 2>&1 &
        echo \$!
    " 2>/dev/null
    echo "  [daemon] Started"
}

# Wait until gateway /health responds (up to 15s)
wait_for_gateway() {
    echo "  [gateway] Waiting for gateway to be ready..."
    local attempts=0
    while [[ $attempts -lt 30 ]]; do
        if ssh "$TARGET_HOST" "curl -sf ${GATEWAY_URL}/health > /dev/null 2>&1"; then
            echo "  [gateway] Ready"
            return 0
        fi
        sleep 0.5
        attempts=$((attempts + 1))
    done
    echo "  [gateway] ERROR: Gateway did not become ready in 15s"
    exit 1
}

# Print skill files present on target
show_deployed_skills() {
    local mode="$1"
    echo "  [verify] Deployed skill files ($mode):"
    ssh "$TARGET_HOST" "ls ${WS}/skills/weather/ ${WS}/skills/tv-control/ 2>/dev/null" \
        | sed 's/^/    /'
}

# Send query to daemon gateway.
# Sets _LAST_MS (elapsed ms) and _LAST_TURNS (LLM turn count).
run_measurement() {
    local query="$1"
    # Escape for JSON embedding (backslash and double-quote)
    local json_query
    json_query=$(printf '%s' "$query" | sed 's/\\/\\\\/g; s/"/\\"/g')

    # Clear trace file before request to isolate this request's events
    ssh "$TARGET_HOST" "> ${TRACE_FILE} 2>/dev/null || true"

    # Send request, measure elapsed time on target
    local elapsed_s
    elapsed_s=$(ssh "$TARGET_HOST" "
        curl -s -o /dev/null \
             -w '%{time_total}' \
             -X POST ${GATEWAY_URL}/api/chat \
             -H 'Content-Type: application/json' \
             -d '{\"message\":\"${json_query}\"}' \
        2>/dev/null
    ")

    # Count llm_response events written during this request
    local turns
    turns=$(ssh "$TARGET_HOST" "
        grep -c '\"llm_response\"' ${TRACE_FILE} 2>/dev/null || echo 0
    ")

    _LAST_MS=$(printf '%.0f' "$(echo "${elapsed_s//[[:space:]]/} * 1000" | bc)")
    _LAST_TURNS="${turns//[[:space:]]/}"
    _LAST_TURNS="${_LAST_TURNS:-0}"
}

# ─────────────────────────────────────────────────────────────
# Benchmark runner
# ─────────────────────────────────────────────────────────────

declare -a WEATHER_TIMES
declare -a TV_TIMES
declare -a WEATHER_TURNS
declare -a TV_TURNS

benchmark_phase() {
    local label="$1"
    WEATHER_TIMES=()
    TV_TIMES=()
    WEATHER_TURNS=()
    TV_TURNS=()
    local total_w=0 total_tv=0 total_tw=0 total_ttv=0

    echo ""
    echo "─────────────────────────────────────"
    echo "  Benchmark: $label"
    echo "─────────────────────────────────────"

    # Warm-up: one ignored run
    echo "  [warm-up] Sending warm-up query..."
    run_measurement "$WEATHER_QUERY" || true

    # Weather
    echo ""
    echo "  [weather] $RUNS runs..."
    for i in $(seq 1 "$RUNS"); do
        run_measurement "$WEATHER_QUERY"
        WEATHER_TIMES+=("$_LAST_MS")
        WEATHER_TURNS+=("$_LAST_TURNS")
        total_w=$((total_w + _LAST_MS))
        total_tw=$((total_tw + _LAST_TURNS))
        printf "    #%2d : %5d ms  (%d turns)\n" "$i" "$_LAST_MS" "$_LAST_TURNS"
    done
    local avg_w=$((total_w / RUNS))
    local avg_tw_x10=$(( (total_tw * 10) / RUNS ))

    # TV-control
    echo ""
    echo "  [tv-control] $RUNS runs... (alternating vol=8 / vol=10)"
    for i in $(seq 1 "$RUNS"); do
        if (( i % 2 == 1 )); then
            run_measurement "$TV_QUERY_A"
        else
            run_measurement "$TV_QUERY_B"
        fi
        TV_TIMES+=("$_LAST_MS")
        TV_TURNS+=("$_LAST_TURNS")
        total_tv=$((total_tv + _LAST_MS))
        total_ttv=$((total_ttv + _LAST_TURNS))
        printf "    #%2d : %5d ms  (%d turns)\n" "$i" "$_LAST_MS" "$_LAST_TURNS"
    done
    local avg_tv=$((total_tv / RUNS))
    local avg_ttv_x10=$(( (total_ttv * 10) / RUNS ))

    # Save results: label|skill|times_csv|avg_ms|turns_csv|avg_turns_x10
    echo "${label}|weather|$(IFS=','; echo "${WEATHER_TIMES[*]}")|${avg_w}|$(IFS=','; echo "${WEATHER_TURNS[*]}")|${avg_tw_x10}" >> "$RESULTS_FILE"
    echo "${label}|tv-control|$(IFS=','; echo "${TV_TIMES[*]}")|${avg_tv}|$(IFS=','; echo "${TV_TURNS[*]}")|${avg_ttv_x10}" >> "$RESULTS_FILE"

    echo ""
    printf "  Averages  → weather: %d ms (%d.%d turns)  |  tv-control: %d ms (%d.%d turns)\n" \
        "$avg_w"  "$((avg_tw_x10  / 10))" "$((avg_tw_x10  % 10))" \
        "$avg_tv" "$((avg_ttv_x10 / 10))" "$((avg_ttv_x10 % 10))"
}

# ─────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════════"
echo "  ZeroClaw Skill Mode Benchmark (daemon mode)"
echo "  Target  : $TARGET_HOST"
echo "  Gateway : $GATEWAY_URL/api/chat"
echo "  Runs    : $RUNS per skill per mode"
echo "══════════════════════════════════════════════"

# Verify SSH connectivity
if ! ssh -o ConnectTimeout=5 -o BatchMode=yes "$TARGET_HOST" "echo ok" >/dev/null 2>&1; then
    echo "ERROR: Cannot SSH to $TARGET_HOST"
    exit 1
fi

# Verify bc is available locally (for ms conversion)
if ! command -v bc &>/dev/null; then
    echo "ERROR: 'bc' is required for time conversion. Install it and retry."
    exit 1
fi

rm -f "$RESULTS_FILE"

# Enable runtime trace on target; restore on exit (normal or error)
enable_runtime_trace
trap 'disable_runtime_trace' EXIT

# ──────────────────────────────
# Phase 1: SKILL.md mode
# ──────────────────────────────
echo ""
echo "[Phase 1] SKILL.md mode"
clear_skills
deploy_skills
activate_skill_md_mode
show_deployed_skills "SKILL.md"
start_daemon
wait_for_gateway
benchmark_phase "SKILL.md"

# ──────────────────────────────
# Phase 2: SKILL.toml mode
# ──────────────────────────────
echo ""
echo "[Phase 2] SKILL.toml mode"
clear_skills
deploy_skills
show_deployed_skills "SKILL.toml"
start_daemon
wait_for_gateway
benchmark_phase "SKILL.toml"

# ─────────────────────────────────────────────────────────────
# Final Report
# ─────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════════"
echo "  Final Benchmark Report"
echo "══════════════════════════════════════════════"

# Per-run detail table for each skill × mode
declare -A avg_map
declare -A turns_map
declare -A times_arr
declare -A turns_arr

while IFS='|' read -r label skill times avg_ms turns avg_turns_x10; do
    avg_map["${skill}_${label}"]="$avg_ms"
    turns_map["${skill}_${label}"]="$avg_turns_x10"
    times_arr["${skill}_${label}"]="$times"
    turns_arr["${skill}_${label}"]="$turns"
done < "$RESULTS_FILE"

for skill in weather tv-control; do
    echo ""
    echo "  ── $skill ──────────────────────────────────────────────────"
    printf "  %4s  %12s  %7s  %12s  %7s\n" \
        "Run" "SKILL.md" "turns" "SKILL.toml" "turns"
    echo "  ──────────────────────────────────────────────────────────"

    IFS=',' read -ra md_times   <<< "${times_arr[${skill}_SKILL.md]:-}"
    IFS=',' read -ra toml_times <<< "${times_arr[${skill}_SKILL.toml]:-}"
    IFS=',' read -ra md_turns   <<< "${turns_arr[${skill}_SKILL.md]:-}"
    IFS=',' read -ra toml_turns <<< "${turns_arr[${skill}_SKILL.toml]:-}"

    local_runs=${#md_times[@]}
    for (( i=0; i<local_runs; i++ )); do
        printf "  %4d  %8d ms  %7s  %8d ms  %7s\n" \
            "$((i+1))" \
            "${md_times[$i]}"   "${md_turns[$i]} turns" \
            "${toml_times[$i]}" "${toml_turns[$i]} turns"
    done

    md_avg="${avg_map[${skill}_SKILL.md]:-0}"
    toml_avg="${avg_map[${skill}_SKILL.toml]:-0}"
    md_tx="${turns_map[${skill}_SKILL.md]:-0}"
    toml_tx="${turns_map[${skill}_SKILL.toml]:-0}"
    echo "  ──────────────────────────────────────────────────────────"
    printf "  %4s  %8d ms  %d.%d turns  %8d ms  %d.%d turns\n" \
        "avg" \
        "$md_avg"   "$((md_tx   / 10))" "$((md_tx   % 10))" \
        "$toml_avg" "$((toml_tx / 10))" "$((toml_tx % 10))"
done

# Summary comparison
echo ""
echo "  ── Comparison ──────────────────────────────────────────────────"
printf "  %-14s  %11s  %9s  %11s  %9s  %s\n" \
    "Skill" "SKILL.md" "turns" "SKILL.toml" "turns" "Diff (ms)"
echo "  ────────────────────────────────────────────────────────────────"

for skill in weather tv-control; do
    md_avg="${avg_map[${skill}_SKILL.md]:-0}"
    toml_avg="${avg_map[${skill}_SKILL.toml]:-0}"
    md_tx="${turns_map[${skill}_SKILL.md]:-0}"
    toml_tx="${turns_map[${skill}_SKILL.toml]:-0}"
    diff=$((md_avg - toml_avg))
    if [[ $diff -gt 0 ]]; then
        diff_str="SKILL.toml faster by ${diff}ms"
    elif [[ $diff -lt 0 ]]; then
        diff_str="SKILL.md faster by ${diff#-}ms"
    else
        diff_str="equal"
    fi
    printf "  %-14s  %8d ms  %d.%d turns  %8d ms  %d.%d turns  %s\n" \
        "$skill" \
        "$md_avg"   "$((md_tx   / 10))" "$((md_tx   % 10))" \
        "$toml_avg" "$((toml_tx / 10))" "$((toml_tx % 10))" \
        "$diff_str"
done

echo ""
echo "══════════════════════════════════════════════"
echo "  Benchmark complete"
echo "══════════════════════════════════════════════"
echo ""

rm -f "$RESULTS_FILE"
