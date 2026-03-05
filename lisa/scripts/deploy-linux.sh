#!/usr/bin/env bash
set -euo pipefail

# ─────────────────────────────────────────────
# Lisa -> Linux (Ubuntu) deploy script
# Usage:
#   ./deploy-linux.sh              # local install
#   ./deploy-linux.sh <IP>         # remote install via SSH
# ─────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LISA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$LISA_DIR/.." && pwd)"
PROFILE_DIR="$LISA_DIR/profiles/lisa"
CONFIG_FILE="$LISA_DIR/config/config.linux.toml"
GOG_CONFIG_LOCAL="${HOME}/.config/gogcli"
LISA_ENV_FILE="$PROFILE_DIR/lisa.env"

REMOTE=false
TARGET_IP=""
TARGET=""
DEPLOY_DIR=""
ZEROCLAW_DIR=""
WORKSPACE_DIR=""
TOTAL=11
GOG_EMAIL=""
GOG_KR_PASS=""

echo ""
echo "Lisa -> Linux (Ubuntu) deploy"
echo "============================="

# ── Helpers ──
run_cmd() {
    if [ "$REMOTE" = true ]; then
        ssh "${TARGET}" "$1"
    else
        bash -c "$1"
    fi
}

copy_file() {
    if [ "$REMOTE" = true ]; then
        scp "$1" "${TARGET}:$2"
    else
        cp "$1" "$2"
    fi
}

# -- 1) Pre-flight check --
echo ""
echo "[1/$TOTAL] Pre-flight check..."

if [ ! -f "$CONFIG_FILE" ]; then
    echo "  FAIL - Config not found: $CONFIG_FILE"
    exit 1
fi

# Determine mode
if [ -n "${1:-}" ]; then
    TARGET_IP="$1"
    REMOTE=true
    TARGET="$(whoami)@${TARGET_IP}"
    echo "  Mode: remote (${TARGET})"
else
    echo "  Mode: local"
fi

# Detect target architecture (remote: ask target, local: uname)
if [ "$REMOTE" = true ]; then
    ARCH=$(ssh "${TARGET}" "uname -m")
else
    ARCH=$(uname -m)
fi
case "$ARCH" in
    x86_64|amd64) ARCH_DIR="x86_64" ;;
    aarch64|arm64) ARCH_DIR="arm64" ;;
    *) ARCH_DIR="" ;;
esac
echo "  Target arch: ${ARCH} (${ARCH_DIR:-unknown})"

# Find zeroclaw binary
BINARY=""

# 1) Pre-built release binary matching target arch
if [ -n "$ARCH_DIR" ] && [ -f "$LISA_DIR/release/${ARCH_DIR}/zeroclaw" ]; then
    BINARY="$LISA_DIR/release/${ARCH_DIR}/zeroclaw"
fi

# 2) cargo build output (local only, same arch)
LOCAL_ARCH=$(uname -m)
if [ -z "$BINARY" ] && [ "$ARCH" = "$LOCAL_ARCH" ] && [ -f "$PROJECT_ROOT/target/release/zeroclaw" ]; then
    BINARY="$PROJECT_ROOT/target/release/zeroclaw"
fi

# 3) Offer to build (local only, same arch)
if [ -z "$BINARY" ]; then
    if [ "$REMOTE" = false ] || [ "$ARCH" = "$LOCAL_ARCH" ]; then
        read -rp "  Binary not found for ${ARCH}. Build now? (cargo build --release) [Y/n]: " BUILD
        if [ "${BUILD}" != "n" ] && [ "${BUILD}" != "N" ]; then
            echo "  Building... (this may take a few minutes)"
            (cd "$PROJECT_ROOT" && cargo build --release)
            BINARY="$PROJECT_ROOT/target/release/zeroclaw"
        else
            echo "  FAIL - Binary required"
            exit 1
        fi
    else
        echo "  FAIL - No binary found for target arch: ${ARCH}"
        echo "  Place at: $LISA_DIR/release/${ARCH_DIR:-<arch>}/zeroclaw"
        exit 1
    fi
fi

# gog binary (optional, matching target arch)
GOG_BINARY=""
if [ -n "$ARCH_DIR" ] && [ -f "$LISA_DIR/release/${ARCH_DIR}/gog" ]; then
    GOG_BINARY="$LISA_DIR/release/${ARCH_DIR}/gog"
fi

echo "  Binary: $BINARY"
echo "  OK"

# -- 2) Connection / target setup --
echo ""
if [ "$REMOTE" = true ]; then
    echo "[2/$TOTAL] Testing SSH connection..."
    if ! ssh -o ConnectTimeout=5 -o BatchMode=yes "${TARGET}" "echo ok" >/dev/null 2>&1; then
        echo "  FAIL - Cannot connect to ${TARGET}"
        echo "         ssh-copy-id ${TARGET}"
        exit 1
    fi
    REMOTE_HOME=$(ssh "${TARGET}" 'echo $HOME')
    DEPLOY_DIR="${REMOTE_HOME}/lisa"
    ZEROCLAW_DIR="${REMOTE_HOME}/.zeroclaw"
else
    echo "[2/$TOTAL] Local environment check..."
    DEPLOY_DIR="${HOME}/lisa"
    ZEROCLAW_DIR="${HOME}/.zeroclaw"
fi
WORKSPACE_DIR="${ZEROCLAW_DIR}/workspace"
echo "  Deploy: ${DEPLOY_DIR}"
echo "  OK"

# -- 3) Create directories --
echo ""
echo "[3/$TOTAL] Creating directories..."
run_cmd "mkdir -p '${DEPLOY_DIR}' '${ZEROCLAW_DIR}' '${WORKSPACE_DIR}' '${WORKSPACE_DIR}/skills'"
echo "  OK"

# -- 4) Install binary --
echo ""
echo "[4/$TOTAL] Installing binary..."
copy_file "$BINARY" "${DEPLOY_DIR}/zeroclaw"
run_cmd "chmod +x '${DEPLOY_DIR}/zeroclaw'"

if [ -n "$GOG_BINARY" ]; then
    copy_file "$GOG_BINARY" "${DEPLOY_DIR}/gog"
    run_cmd "chmod +x '${DEPLOY_DIR}/gog'"
    echo "  gog (calendar CLI)"
fi
echo "  OK"

# -- 5) Install config --
echo ""
echo "[5/$TOTAL] Installing config.toml..."
run_cmd "[ -f '${ZEROCLAW_DIR}/config.toml' ] && cp '${ZEROCLAW_DIR}/config.toml' '${ZEROCLAW_DIR}/config.toml.bak.\$(date +%s)' && echo '  Existing config backed up' || true"
copy_file "$CONFIG_FILE" "${ZEROCLAW_DIR}/config.toml"
run_cmd "chmod 600 '${ZEROCLAW_DIR}/config.toml'"
echo "  OK"

# -- 6) Install workspace files --
echo ""
echo "[6/$TOTAL] Installing workspace files..."
for f in SOUL.md AGENTS.md USER.md; do
    if [ -f "$PROFILE_DIR/$f" ]; then
        copy_file "$PROFILE_DIR/$f" "${WORKSPACE_DIR}/$f"
        echo "  $f"
    fi
done

if [ -d "$PROFILE_DIR/skills" ]; then
    for skill_dir in "$PROFILE_DIR/skills"/*/; do
        [ -d "$skill_dir" ] || continue
        skill_name=$(basename "$skill_dir")
        run_cmd "mkdir -p '${WORKSPACE_DIR}/skills/${skill_name}'"
        if [ "$REMOTE" = true ]; then
            scp -r "${skill_dir}"* "${TARGET}:${WORKSPACE_DIR}/skills/${skill_name}/" 2>/dev/null || true
        else
            cp -r "${skill_dir}"* "${WORKSPACE_DIR}/skills/${skill_name}/" 2>/dev/null || true
        fi
    done
    SKILL_COUNT=$(find "$PROFILE_DIR/skills" -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' ')
    echo "  $SKILL_COUNT skill(s)"
    run_cmd "find '${WORKSPACE_DIR}/skills' -name '*.sh' -exec chmod +x {} +"
fi
echo "  OK"

# -- 7) Configure Target TV --
echo ""
echo "[7/$TOTAL] Configuring Target TV..."
TV_SKILL_FILE="${WORKSPACE_DIR}/skills/tv-control/SKILL.md"
echo "  Select TV location for tv-control skill:"
echo "    1) N/A    — no target TV (skip tv-control)"
echo "    2) local  — commands run directly on this device"
echo "    3) remote — commands run via SSH to another TV"
read -rp "  Choice [1/2/3] (default: 1): " TV_CHOICE
TV_CHOICE="${TV_CHOICE:-1}"

case "$TV_CHOICE" in
    2) TV_LOCATION="local"; TV_IP="N/A" ;;
    3)
        TV_LOCATION="remote"
        read -rp "  Remote TV IP address: " TV_IP
        if [ -z "$TV_IP" ]; then
            echo "  WARNING: no IP provided, setting to N/A"
            TV_LOCATION="N/A"; TV_IP="N/A"
        fi
        ;;
    *) TV_LOCATION="N/A"; TV_IP="N/A" ;;
esac

run_cmd "if [ -f '${TV_SKILL_FILE}' ]; then sed -i \"s/^- \*\*Location\*\*:.*/- **Location**: ${TV_LOCATION}/\" '${TV_SKILL_FILE}'; if [ '${TV_LOCATION}' = 'remote' ]; then sed -i \"s/^- \*\*IP\*\*:.*/- **IP**: ${TV_IP}/\" '${TV_SKILL_FILE}'; else sed -i 's/^- \*\*IP\*\*:.*/- **IP**: N\/A/' '${TV_SKILL_FILE}'; fi; echo '  Location: ${TV_LOCATION}'; echo '  IP: ${TV_IP}'; else echo '  SKIP — SKILL.md not found'; fi"
echo "  OK"

# -- 8) gog (calendar) setup --
echo ""
echo "[8/$TOTAL] Setting up gog (calendar)..."

# Load existing GOG_ACCOUNT from lisa.env
SAVED_GOG_ACCOUNT=""
[ -f "$LISA_ENV_FILE" ] && SAVED_GOG_ACCOUNT=$(grep -oP '^(export )?GOG_ACCOUNT=\K.+' "$LISA_ENV_FILE" 2>/dev/null || true)

# Check for local gog OAuth tokens
GOG_KEYRING_LOCAL="$GOG_CONFIG_LOCAL/keyring"
if [ ! -d "$GOG_KEYRING_LOCAL" ] || [ -z "$(ls -A "$GOG_KEYRING_LOCAL" 2>/dev/null)" ]; then
    echo "  No gog OAuth tokens found locally."

    # Install gog if not available
    if ! command -v gog >/dev/null 2>&1; then
        echo "  gog CLI not found."
        read -rp "  Install gog now? [Y/n]: " INSTALL_GOG
        if [ "${INSTALL_GOG}" != "n" ] && [ "${INSTALL_GOG}" != "N" ]; then
            if command -v brew >/dev/null 2>&1; then
                echo "  Installing via Homebrew..."
                brew install steipete/tap/gogcli || echo "  WARNING: install failed"
            elif command -v go >/dev/null 2>&1; then
                echo "  Installing via go install..."
                go install github.com/steipete/gogcli/cmd/gog@latest || echo "  WARNING: install failed"
                GOBIN="$(go env GOPATH)/bin"
                if [ -d "$GOBIN" ] && ! echo "$PATH" | grep -q "$GOBIN"; then
                    export PATH="$GOBIN:$PATH"
                fi
            else
                echo "  Neither brew nor go found. Install one first."
            fi
        fi
    fi

    # OAuth setup
    if command -v gog >/dev/null 2>&1; then
        read -rp "  Set up Google Calendar (gog) now? [Y/n]: " SETUP_GOG
        if [ "${SETUP_GOG}" != "n" ] && [ "${SETUP_GOG}" != "N" ]; then
            if [ ! -f "$GOG_CONFIG_LOCAL/credentials.json" ]; then
                read -rp "  Path to client_secret.json (from Google Cloud Console): " CLIENT_SECRET_PATH
                if [ -n "${CLIENT_SECRET_PATH}" ] && [ -f "${CLIENT_SECRET_PATH}" ]; then
                    gog auth credentials "${CLIENT_SECRET_PATH}" || echo "  WARNING: credentials setup failed"
                else
                    echo "  Skipped — file not found"
                fi
            fi

            echo "  Setting keyring backend to file..."
            gog auth keyring file || echo "  WARNING: keyring setup failed"

            if [ -f "$GOG_CONFIG_LOCAL/credentials.json" ]; then
                if [ -n "$SAVED_GOG_ACCOUNT" ]; then
                    read -rp "  Google account email [$SAVED_GOG_ACCOUNT]: " GOG_EMAIL
                    GOG_EMAIL="${GOG_EMAIL:-$SAVED_GOG_ACCOUNT}"
                else
                    read -rp "  Google account email: " GOG_EMAIL
                fi
                if [ -n "${GOG_EMAIL}" ]; then
                    read -rsp "  Keyring password (for encrypting OAuth tokens): " GOG_KR_PASS
                    echo ""
                    export GOG_KEYRING_PASSWORD="$GOG_KR_PASS"
                    echo "  OAuth URL will be printed below."
                    echo "  Open it in any browser, authorize, then paste the redirect URL back here."
                    gog auth add "${GOG_EMAIL}" --services calendar --manual || echo "  WARNING: OAuth failed"
                fi
            fi
        fi
    fi
fi

# Transfer gog credentials (remote only — local already has them)
if [ "$REMOTE" = true ]; then
    TARGET_GOG_DIR="${REMOTE_HOME}/.config/gogcli"
    if [ -d "$GOG_CONFIG_LOCAL" ] && [ -n "$(ls -A "$GOG_CONFIG_LOCAL" 2>/dev/null)" ]; then
        ssh "${TARGET}" "mkdir -p '${TARGET_GOG_DIR}'"
        scp -r "$GOG_CONFIG_LOCAL/"* "${TARGET}:${TARGET_GOG_DIR}/"
        ssh "${TARGET}" "chmod -R 600 '${TARGET_GOG_DIR}'; chmod 700 '${TARGET_GOG_DIR}' '${TARGET_GOG_DIR}/keyring' 2>/dev/null || true"
        echo "  $(ls "$GOG_CONFIG_LOCAL" | wc -l | tr -d ' ') file(s) transferred"
    else
        echo "  SKIP — no gog credentials to transfer"
    fi
fi

# lisa.env setup
DEFAULT_ACCOUNT="${GOG_EMAIL:-$SAVED_GOG_ACCOUNT}"
DEFAULT_PASSWORD="${GOG_KR_PASS:-}"
SAVED_GOG_PASSWORD=""
[ -f "$LISA_ENV_FILE" ] && SAVED_GOG_PASSWORD=$(grep -oP '^(export )?GOG_KEYRING_PASSWORD=\K.+' "$LISA_ENV_FILE" 2>/dev/null || true)

if [ ! -f "$LISA_ENV_FILE" ]; then
    read -rp "  Create lisa.env? [Y/n]: " CREATE_ENV
    if [ "${CREATE_ENV}" != "n" ] && [ "${CREATE_ENV}" != "N" ]; then
        if [ -n "$DEFAULT_ACCOUNT" ]; then
            read -rp "  GOG_ACCOUNT (email) [$DEFAULT_ACCOUNT]: " INPUT_ACCOUNT
            INPUT_ACCOUNT="${INPUT_ACCOUNT:-$DEFAULT_ACCOUNT}"
        else
            read -rp "  GOG_ACCOUNT (email): " INPUT_ACCOUNT
        fi
        if [ -n "$DEFAULT_PASSWORD" ]; then
            INPUT_PASSWORD="$DEFAULT_PASSWORD"
            echo "  GOG_KEYRING_PASSWORD: (auto-filled from keyring setup)"
        else
            read -rsp "  GOG_KEYRING_PASSWORD: " INPUT_PASSWORD
            echo ""
        fi
        cat > "$LISA_ENV_FILE" << ENVEOF
# Lisa environment variables
# Generated by deploy-linux.sh on $(date +%Y-%m-%d)

# --- Google Calendar (gog) ---
export GOG_ACCOUNT=${INPUT_ACCOUNT}
export GOG_KEYRING_PASSWORD=${INPUT_PASSWORD}
export GOG_KEYRING_BACKEND=file
ENVEOF
        echo "  lisa.env created"
    fi
else
    # lisa.env exists — update missing or changed values
    if [ -z "$SAVED_GOG_PASSWORD" ] && [ -z "$DEFAULT_PASSWORD" ]; then
        read -rsp "  GOG_KEYRING_PASSWORD is empty in lisa.env. Enter password: " DEFAULT_PASSWORD
        echo ""
    fi
    if [ -n "$DEFAULT_PASSWORD" ] && [ "$DEFAULT_PASSWORD" != "$SAVED_GOG_PASSWORD" ]; then
        sed -i "s/^.*GOG_KEYRING_PASSWORD=.*/export GOG_KEYRING_PASSWORD=${DEFAULT_PASSWORD}/" "$LISA_ENV_FILE"
        echo "  lisa.env updated (GOG_KEYRING_PASSWORD)"
    fi
    if [ -n "$DEFAULT_ACCOUNT" ] && [ "$DEFAULT_ACCOUNT" != "$SAVED_GOG_ACCOUNT" ]; then
        sed -i "s/^.*GOG_ACCOUNT=.*/export GOG_ACCOUNT=${DEFAULT_ACCOUNT}/" "$LISA_ENV_FILE"
        echo "  lisa.env updated (GOG_ACCOUNT)"
    fi
fi

# Transfer lisa.env
if [ -f "$LISA_ENV_FILE" ]; then
    copy_file "$LISA_ENV_FILE" "${DEPLOY_DIR}/lisa.env"
    run_cmd "chmod 600 '${DEPLOY_DIR}/lisa.env'"
    echo "  lisa.env"
fi
echo "  OK"

# -- 9) /etc/hosts (Azure private endpoint, if needed) --
echo ""
echo "[9/$TOTAL] Checking /etc/hosts..."
HOSTS_ENTRY="10.182.173.75 tvdevops.openai.azure.com"

if grep -q "tvdevops.openai.azure.com" "$CONFIG_FILE" 2>/dev/null; then
    HOSTS_EXISTS=$(run_cmd "grep -c 'tvdevops.openai.azure.com' /etc/hosts 2>/dev/null || echo 0")
    if [ "$HOSTS_EXISTS" = "0" ]; then
        echo "  Azure private endpoint not in /etc/hosts"
        read -rp "  Add '${HOSTS_ENTRY}'? (requires sudo) [Y/n]: " ADD_HOST
        if [ "${ADD_HOST}" != "n" ] && [ "${ADD_HOST}" != "N" ]; then
            if [ "$REMOTE" = true ]; then
                ssh "${TARGET}" "echo '# Azure OpenAI endpoint for Lisa' | sudo tee -a /etc/hosts >/dev/null; echo '${HOSTS_ENTRY}' | sudo tee -a /etc/hosts >/dev/null"
            else
                echo "# Azure OpenAI endpoint for Lisa" | sudo tee -a /etc/hosts >/dev/null
                echo "${HOSTS_ENTRY}" | sudo tee -a /etc/hosts >/dev/null
            fi
            echo "  Added"
        fi
    else
        echo "  Already configured"
    fi
else
    echo "  SKIP — no Azure private endpoint in config"
fi
echo "  OK"

# -- 10) Create start scripts + PATH --
echo ""
echo "[10/$TOTAL] Creating start scripts..."

# Generate start-lisa.sh (daemon)
cat << EOF > /tmp/lisa-start-linux.sh
#!/bin/sh
cd ${DEPLOY_DIR}
export ZEROCLAW_CONFIG_DIR="${ZEROCLAW_DIR}"
export PATH="${DEPLOY_DIR}:\$PATH"
[ -f ${DEPLOY_DIR}/lisa.env ] && . ${DEPLOY_DIR}/lisa.env
exec ${DEPLOY_DIR}/zeroclaw daemon
EOF

# Generate lisa-agent.sh (agent)
# Note: agent CLI --temperature default(0.7) ignores config, so -t 1.0 is explicit
cat << EOF > /tmp/lisa-agent-linux.sh
#!/bin/sh
cd ${DEPLOY_DIR}
export ZEROCLAW_CONFIG_DIR="${ZEROCLAW_DIR}"
export PATH="${DEPLOY_DIR}:\$PATH"
[ -f ${DEPLOY_DIR}/lisa.env ] && . ${DEPLOY_DIR}/lisa.env
if [ -n "\$1" ]; then
    exec ${DEPLOY_DIR}/zeroclaw agent -t 1.0 -m "\$*"
else
    exec ${DEPLOY_DIR}/zeroclaw agent -t 1.0
fi
EOF

copy_file /tmp/lisa-start-linux.sh "${DEPLOY_DIR}/start-lisa.sh"
copy_file /tmp/lisa-agent-linux.sh "${DEPLOY_DIR}/lisa-agent.sh"
run_cmd "chmod +x '${DEPLOY_DIR}/start-lisa.sh' '${DEPLOY_DIR}/lisa-agent.sh'"
rm -f /tmp/lisa-start-linux.sh /tmp/lisa-agent-linux.sh

echo "  start-lisa.sh (daemon)"
echo "  lisa-agent.sh (agent)"

# Add to PATH via .bashrc
BASHRC_CHECK=$(run_cmd "grep -c '${DEPLOY_DIR}' ~/.bashrc 2>/dev/null || echo 0")
if [ "$BASHRC_CHECK" = "0" ]; then
    run_cmd "printf '\n# Lisa: add to PATH\nexport PATH=\"${DEPLOY_DIR}:\$PATH\"\n' >> ~/.bashrc"
    echo "  PATH added to .bashrc"
fi
echo "  OK"

# -- 11) Post-deploy tests --
echo ""
echo "[11/$TOTAL] Running post-deploy tests..."
TEST_PASS=0
TEST_FAIL=0

run_test() {
    local name="$1"
    local result="$2"
    local exit_code="$3"
    if [ "$exit_code" -eq 0 ] && [ -n "$result" ]; then
        echo "  [PASS] $name"
        TEST_PASS=$((TEST_PASS + 1))
    else
        echo "  [FAIL] $name"
        [ -n "$result" ] && echo "     $result"
        TEST_FAIL=$((TEST_FAIL + 1))
    fi
}

# Agent test
echo ""
echo "  [agent mode]"
if [ "$REMOTE" = true ]; then
    AGENT_RESULT=$(ssh "${TARGET}" "'${DEPLOY_DIR}/lisa-agent.sh' hi" 2>&1 | head -20) || true
else
    AGENT_RESULT=$("${DEPLOY_DIR}/lisa-agent.sh" hi 2>&1 | head -20) || true
fi
AGENT_EXIT=$?
run_test "agent: single message" "$AGENT_RESULT" "$AGENT_EXIT"

# Weather test
echo ""
echo "  [weather skill]"
run_cmd "curl -s --max-time 5 -o /dev/null 'http://wttr.in'" >/dev/null 2>&1 && HAS_INET=true || HAS_INET=false
if [ "$HAS_INET" = "true" ]; then
    WEATHER_RESULT=$(run_cmd "curl -s --max-time 10 'wttr.in/Seoul?format=%c+%t'" 2>&1) || true
    WEATHER_EXIT=$?
    run_test "weather: wttr.in" "$WEATHER_RESULT" "$WEATHER_EXIT"
else
    echo "  [SKIP] no internet access"
fi

# Calendar test
echo ""
echo "  [calendar skill]"
GOG_CHECK=$(run_cmd "command -v gog >/dev/null 2>&1 && echo yes || (test -x '${DEPLOY_DIR}/gog' && echo yes || echo no)" 2>/dev/null) || GOG_CHECK="no"
if [ "$GOG_CHECK" = "yes" ]; then
    CAL_ENV=". '${DEPLOY_DIR}/lisa.env' 2>/dev/null; export PATH='${DEPLOY_DIR}':\$PATH"
    CAL_LIST=$(run_cmd "${CAL_ENV}; gog calendar calendars 2>&1 | head -5") || true
    CAL_EXIT=$?
    run_test "calendar: gog calendar calendars" "$CAL_LIST" "$CAL_EXIT"
else
    echo "  [SKIP] gog not installed"
fi

# Daemon test
echo ""
echo "  [daemon mode]"
run_cmd "cd '${DEPLOY_DIR}' && export ZEROCLAW_CONFIG_DIR='${ZEROCLAW_DIR}' && export PATH='${DEPLOY_DIR}':\$PATH && [ -f '${DEPLOY_DIR}/lisa.env' ] && . '${DEPLOY_DIR}/lisa.env'; nohup '${DEPLOY_DIR}/zeroclaw' daemon > /tmp/lisa-daemon-test.log 2>&1 &" || true
sleep 3
DAEMON_STATUS=$(run_cmd "'${DEPLOY_DIR}/zeroclaw' status" 2>&1) || true
DAEMON_EXIT=$?
run_test "daemon: zeroclaw status" "$DAEMON_STATUS" "$DAEMON_EXIT"

# Gateway /health
GW_PORT=$(run_cmd "grep '^port' '${ZEROCLAW_DIR}/config.toml' 2>/dev/null | head -1 | sed 's/[^0-9]//g'" 2>/dev/null) || true
GW_PORT="${GW_PORT:-42617}"
HEALTH_RESULT=$(run_cmd "curl -s --max-time 5 'http://127.0.0.1:${GW_PORT}/health'" 2>&1) || true
if echo "$HEALTH_RESULT" | grep -q '"status"'; then
    run_test "gateway: /health (port ${GW_PORT})" "$HEALTH_RESULT" "0"
else
    run_test "gateway: /health (port ${GW_PORT})" "$HEALTH_RESULT" "1"
fi

# Telegram test
TG_TOKEN=$(run_cmd "grep 'bot_token' '${ZEROCLAW_DIR}/config.toml' 2>/dev/null | sed 's/.*= *\"//;s/\".*//' | head -1" 2>/dev/null) || true
if [ -n "$TG_TOKEN" ] && [ "$TG_TOKEN" != "YOUR_BOT_TOKEN" ] && [ "$HAS_INET" = "true" ]; then
    echo ""
    echo "  [telegram channel]"
    TG_RESULT=$(run_cmd "curl -s --max-time 10 'https://api.telegram.org/bot${TG_TOKEN}/getMe'" 2>&1) || true
    if echo "$TG_RESULT" | grep -q '"ok":true'; then
        TG_NAME=$(echo "$TG_RESULT" | python3 -c "import sys,json;print(json.load(sys.stdin)['result']['first_name'])" 2>/dev/null || echo "unknown")
        run_test "telegram: Bot API getMe (${TG_NAME})" "$TG_RESULT" "0"
    else
        run_test "telegram: Bot API getMe" "$TG_RESULT" "1"
    fi
fi

# Stop daemon
run_cmd "pkill -f 'zeroclaw daemon'; rm -f /tmp/lisa-daemon-test.log" 2>/dev/null || true

# Test summary
echo ""
echo "  ------------------------"
echo "  Results: ${TEST_PASS} passed / ${TEST_FAIL} failed"
[ "$TEST_FAIL" -gt 0 ] && echo "  WARNING: Some tests failed. Check logs above."

echo ""
echo "========================================================"
echo "  Lisa deploy complete!"
echo "========================================================"
echo ""
echo "  Deploy: ${DEPLOY_DIR}"
echo "  Config: ${ZEROCLAW_DIR}/config.toml"
echo ""
echo "To use:"
if [ "$REMOTE" = true ]; then
    echo "  ssh ${TARGET} '${DEPLOY_DIR}/start-lisa.sh'      # daemon"
    echo "  ssh ${TARGET} '${DEPLOY_DIR}/lisa-agent.sh'      # agent"
    echo "  ssh ${TARGET} '${DEPLOY_DIR}/lisa-agent.sh hi!'  # message"
    echo "  ssh ${TARGET} '${DEPLOY_DIR}/zeroclaw status'    # status"
else
    echo "  ${DEPLOY_DIR}/start-lisa.sh      # daemon"
    echo "  ${DEPLOY_DIR}/lisa-agent.sh      # agent"
    echo "  ${DEPLOY_DIR}/lisa-agent.sh hi!  # message"
    echo "  ${DEPLOY_DIR}/zeroclaw status    # status"
fi
echo ""
