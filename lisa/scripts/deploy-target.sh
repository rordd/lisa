#!/usr/bin/env bash
set -euo pipefail

# ─────────────────────────────────────────────
# Lisa -> webOS TV (ARM64) target deploy script
# Based on lisa/config/config.arm64.toml
# ─────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LISA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROFILE_DIR="$LISA_DIR/profiles/lisa"
CONFIG_FILE="$LISA_DIR/config/config.arm64.toml"
BINARY="$LISA_DIR/release/arm64/zeroclaw"
GOG_BINARY="$LISA_DIR/release/arm64/gog"
GOG_CONFIG_LOCAL="${HOME}/.config/gogcli"

# Target settings
TARGET_USER="root"
TARGET_DEPLOY_DIR="/home/root/lisa"
TARGET_ZEROCLAW_DIR="/home/root/.zeroclaw"
TARGET_WORKSPACE_DIR="$TARGET_ZEROCLAW_DIR/workspace"
TARGET_GOG_CONFIG="/home/root/.config/gogcli"
TOTAL=14

echo ""
echo "Lisa -> webOS TV target deploy"
echo "=============================="

# -- 1) Pre-flight file check --
echo ""
echo "[1/$TOTAL] Pre-flight file check..."
if [ ! -f "$CONFIG_FILE" ]; then
    echo "  FAIL - Target config not found: $CONFIG_FILE"
    exit 1
fi
if [ ! -f "$BINARY" ]; then
    echo "  FAIL - ARM64 binary not found: $BINARY"
    exit 1
fi
echo "  OK"

# -- 2) TARGET_IP check (arg > prompt) --
if [ -n "${1:-}" ]; then
    TARGET_IP="$1"
else
    read -rp "Enter target IP: " TARGET_IP
    if [ -z "$TARGET_IP" ]; then
        echo "  FAIL - TARGET_IP is required."
        exit 1
    fi
fi

TARGET="${TARGET_USER}@${TARGET_IP}"
echo "  Target: ${TARGET}"

# -- 3) SSH connection test --
echo ""
echo "[2/$TOTAL] Testing SSH connection..."
if ! ssh -o ConnectTimeout=5 -o BatchMode=yes "${TARGET}" "echo ok" >/dev/null 2>&1; then
    echo "  FAIL - Cannot connect to ${TARGET}"
    echo "         ssh-copy-id ${TARGET}"
    exit 1
fi
echo "  OK"

# -- 4) Create target directories --
echo ""
echo "[3/$TOTAL] Creating target directories..."
ssh "${TARGET}" "mkdir -p ${TARGET_DEPLOY_DIR} ${TARGET_ZEROCLAW_DIR} ${TARGET_WORKSPACE_DIR} ${TARGET_WORKSPACE_DIR}/skills"
echo "  OK"

# -- 5) Transfer binary --
echo ""
echo "[4/$TOTAL] Transferring binary..."
scp "${BINARY}" "${TARGET}:${TARGET_DEPLOY_DIR}/zeroclaw"
ssh "${TARGET}" "chmod +x ${TARGET_DEPLOY_DIR}/zeroclaw"
if [ -f "$GOG_BINARY" ]; then
    scp "${GOG_BINARY}" "${TARGET}:${TARGET_DEPLOY_DIR}/gog"
    ssh "${TARGET}" "chmod +x ${TARGET_DEPLOY_DIR}/gog"
    echo "  gog (calendar CLI)"
fi
echo "  OK"

# -- 6) Transfer config.toml --
echo ""
echo "[5/$TOTAL] Transferring config.toml..."

ssh "${TARGET}" bash <<REMOTE_BACKUP
if [ -f "${TARGET_ZEROCLAW_DIR}/config.toml" ]; then
    cp "${TARGET_ZEROCLAW_DIR}/config.toml" "${TARGET_ZEROCLAW_DIR}/config.toml.bak.\$(date +%s)"
    echo "  Existing config backed up"
fi
REMOTE_BACKUP

scp "$CONFIG_FILE" "${TARGET}:${TARGET_ZEROCLAW_DIR}/config.toml"
ssh "${TARGET}" "chmod 600 ${TARGET_ZEROCLAW_DIR}/config.toml"
echo "  OK"

# -- 7) Transfer workspace files --
echo ""
echo "[6/$TOTAL] Transferring workspace files..."
for f in SOUL.md AGENTS.md USER.md; do
    if [ -f "$PROFILE_DIR/$f" ]; then
        scp "$PROFILE_DIR/$f" "${TARGET}:${TARGET_WORKSPACE_DIR}/$f"
        echo "  $f"
    fi
done

if [ -d "$PROFILE_DIR/skills" ]; then
    scp -r "$PROFILE_DIR/skills/"* "${TARGET}:${TARGET_WORKSPACE_DIR}/skills/"
    SKILL_COUNT=$(find "$PROFILE_DIR/skills" -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' ')
    echo "  $SKILL_COUNT skill(s)"
    ssh "${TARGET}" "find ${TARGET_WORKSPACE_DIR}/skills -name '*.sh' -exec chmod +x {} +"
fi
echo "  OK"

# -- 8) Configure Target TV for tv-control skill --
echo ""
echo "[7/$TOTAL] Configuring Target TV..."
TV_SKILL_FILE="${TARGET_WORKSPACE_DIR}/skills/tv-control/SKILL.md"
echo "  Select TV location for tv-control skill:"
echo "    1) N/A    — no target TV (skip tv-control)"
echo "    2) local  — commands run directly on this device"
echo "    3) remote — commands run via SSH to another TV"
read -rp "  Choice [1/2/3] (default: 1): " TV_CHOICE
TV_CHOICE="${TV_CHOICE:-1}"

case "$TV_CHOICE" in
    2)
        TV_LOCATION="local"
        TV_IP="N/A"
        ;;
    3)
        TV_LOCATION="remote"
        read -rp "  Remote TV IP address: " TV_IP
        if [ -z "$TV_IP" ]; then
            echo "  WARNING: no IP provided, setting to N/A"
            TV_LOCATION="N/A"
            TV_IP="N/A"
        fi
        ;;
    *)
        TV_LOCATION="N/A"
        TV_IP="N/A"
        ;;
esac

ssh "${TARGET}" bash <<REMOTE_TV
if [ -f "${TV_SKILL_FILE}" ]; then
    sed -i "s/^- \*\*Location\*\*:.*/- **Location**: ${TV_LOCATION}/" "${TV_SKILL_FILE}"
    if [ "${TV_LOCATION}" = "remote" ]; then
        sed -i "s/^- \*\*IP\*\*:.*/- **IP**: ${TV_IP}/" "${TV_SKILL_FILE}"
    else
        sed -i "s/^- \*\*IP\*\*:.*/- **IP**: N\/A/" "${TV_SKILL_FILE}"
    fi
    echo "  Location: ${TV_LOCATION}"
    echo "  IP: ${TV_IP}"
else
    echo "  SKIP — tv-control SKILL.md not found on target"
fi
REMOTE_TV
echo "  OK"

# -- 9) gog (calendar) setup & transfer --
echo ""
echo "[8/$TOTAL] Setting up gog (calendar)..."

# Load existing GOG_ACCOUNT from lisa.env if available
LISA_ENV_FILE="$PROFILE_DIR/lisa.env"
SAVED_GOG_ACCOUNT=""
if [ -f "$LISA_ENV_FILE" ]; then
    SAVED_GOG_ACCOUNT=$(grep -oP '^(export )?GOG_ACCOUNT=\K.+' "$LISA_ENV_FILE" 2>/dev/null || true)
fi

# Auto-setup: gog OAuth if no local tokens
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
                # go install puts binary in GOPATH/bin — add to PATH
                GOBIN="$(go env GOPATH)/bin"
                if [ -d "$GOBIN" ] && ! echo "$PATH" | grep -q "$GOBIN"; then
                    export PATH="$GOBIN:$PATH"
                fi
            else
                echo "  Neither brew nor go found. Install one first:"
                echo "    brew: https://brew.sh"
                echo "    go:   https://go.dev/dl"
            fi
            if command -v gog >/dev/null 2>&1; then
                echo "  Installed: $(which gog)"
            fi
        fi
    fi

    # Auth setup if gog is available
    if command -v gog >/dev/null 2>&1; then
        read -rp "  Set up Google Calendar (gog) now? [Y/n]: " SETUP_GOG
        if [ "${SETUP_GOG}" != "n" ] && [ "${SETUP_GOG}" != "N" ]; then
            # Step A: register OAuth client credentials (requires client_secret.json from Google Cloud Console)
            if [ ! -f "$GOG_CONFIG_LOCAL/credentials.json" ]; then
                read -rp "  Path to client_secret.json (from Google Cloud Console): " CLIENT_SECRET_PATH
                if [ -n "${CLIENT_SECRET_PATH}" ] && [ -f "${CLIENT_SECRET_PATH}" ]; then
                    gog auth credentials "${CLIENT_SECRET_PATH}" || echo "  WARNING: credentials setup failed"
                else
                    echo "  Skipped — file not found"
                fi
            fi

            # Step B: set keyring backend to file (for headless target)
            echo "  Setting keyring backend to file..."
            gog auth keyring file || echo "  WARNING: keyring setup failed"

            # Step C: OAuth authentication
            if [ -f "$GOG_CONFIG_LOCAL/credentials.json" ]; then
                if [ -n "$SAVED_GOG_ACCOUNT" ]; then
                    read -rp "  Google account email [$SAVED_GOG_ACCOUNT]: " GOG_EMAIL
                    GOG_EMAIL="${GOG_EMAIL:-$SAVED_GOG_ACCOUNT}"
                else
                    read -rp "  Google account email: " GOG_EMAIL
                fi
                if [ -n "${GOG_EMAIL}" ]; then
                    # Set keyring password as env var so gog uses it without prompting
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

# Transfer gog credentials to target
if [ -d "$GOG_CONFIG_LOCAL" ] && [ -n "$(ls -A "$GOG_CONFIG_LOCAL" 2>/dev/null)" ]; then
    ssh "${TARGET}" "mkdir -p ${TARGET_GOG_CONFIG}"
    scp -r "$GOG_CONFIG_LOCAL/"* "${TARGET}:${TARGET_GOG_CONFIG}/"
    ssh "${TARGET}" "chmod -R 600 ${TARGET_GOG_CONFIG}; chmod 700 ${TARGET_GOG_CONFIG} ${TARGET_GOG_CONFIG}/keyring 2>/dev/null"
    echo "  $(ls "$GOG_CONFIG_LOCAL" | wc -l | tr -d ' ') file(s) transferred"
else
    echo "  SKIP — no gog credentials to transfer"
fi

# Auto-setup: lisa.env — create or update
DEFAULT_ACCOUNT="${GOG_EMAIL:-$SAVED_GOG_ACCOUNT}"
DEFAULT_PASSWORD="${GOG_KR_PASS:-}"
SAVED_GOG_PASSWORD=""
if [ -f "$LISA_ENV_FILE" ]; then
    SAVED_GOG_PASSWORD=$(grep -oP '^(export )?GOG_KEYRING_PASSWORD=\K.+' "$LISA_ENV_FILE" 2>/dev/null || true)
fi

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
# Lisa target environment variables
# Generated by deploy-target.sh on $(date +%Y-%m-%d)

# --- Google Calendar (gog) ---
export GOG_ACCOUNT=${INPUT_ACCOUNT}
export GOG_KEYRING_PASSWORD=${INPUT_PASSWORD}
export GOG_KEYRING_BACKEND=file
ENVEOF
        echo "  lisa.env created"
    fi
else
    # lisa.env exists — update missing or changed values
    NEED_UPDATE=false

    # GOG_KEYRING_PASSWORD: prompt if empty and no default from this session
    if [ -z "$SAVED_GOG_PASSWORD" ] && [ -z "$DEFAULT_PASSWORD" ]; then
        read -rsp "  GOG_KEYRING_PASSWORD is empty in lisa.env. Enter password: " DEFAULT_PASSWORD
        echo ""
    fi
    if [ -n "$DEFAULT_PASSWORD" ] && [ "$DEFAULT_PASSWORD" != "$SAVED_GOG_PASSWORD" ]; then
        sed -i "s/^.*GOG_KEYRING_PASSWORD=.*/export GOG_KEYRING_PASSWORD=${DEFAULT_PASSWORD}/" "$LISA_ENV_FILE"
        echo "  lisa.env updated (GOG_KEYRING_PASSWORD)"
        NEED_UPDATE=true
    fi

    # GOG_ACCOUNT: update if changed
    if [ -n "$DEFAULT_ACCOUNT" ] && [ "$DEFAULT_ACCOUNT" != "$SAVED_GOG_ACCOUNT" ]; then
        sed -i "s/^.*GOG_ACCOUNT=.*/export GOG_ACCOUNT=${DEFAULT_ACCOUNT}/" "$LISA_ENV_FILE"
        echo "  lisa.env updated (GOG_ACCOUNT)"
        NEED_UPDATE=true
    fi
fi

# Transfer lisa.env
if [ -f "$LISA_ENV_FILE" ]; then
    scp "$LISA_ENV_FILE" "${TARGET}:${TARGET_DEPLOY_DIR}/lisa.env"
    ssh "${TARGET}" "chmod 600 ${TARGET_DEPLOY_DIR}/lisa.env"
    echo "  lisa.env"
fi
echo "  OK"

# -- 9) /etc/hosts setup (Read-Only bypass) --
echo ""
echo "[9/$TOTAL] Setting up /etc/hosts..."
ssh "${TARGET}" bash <<'REMOTE_HOSTS'
HOSTS_RW="/home/root/hosts"
HOSTS_ENTRY="10.182.173.75 tvdevops.openai.azure.com"

if [ ! -f "$HOSTS_RW" ]; then
    cp /etc/hosts "$HOSTS_RW"
fi

if ! grep -qF "tvdevops.openai.azure.com" "$HOSTS_RW"; then
    printf '\n# Azure OpenAI endpoint for Lisa\n%s\n' "$HOSTS_ENTRY" >> "$HOSTS_RW"
    echo "  added: $HOSTS_ENTRY"
else
    echo "  already exists"
fi

if ! mount | grep -q "/etc/hosts"; then
    mount --bind "$HOSTS_RW" /etc/hosts
    echo "  bind mount applied"
else
    echo "  bind mount already active"
fi
REMOTE_HOSTS
echo "  OK"

# -- 10) Auto bind mount on boot/SSH login --
echo ""
echo "[10/$TOTAL] Setting up auto bind mount..."
ssh "${TARGET}" bash <<'REMOTE_BOOT'
BOOT_SCRIPT="/home/root/lisa/bind-hosts.sh"
PROFILE_FILE="/home/root/.profile"

cat > "$BOOT_SCRIPT" << 'BINDEOF'
#!/bin/sh
HOSTS_RW="/home/root/hosts"
if [ -f "$HOSTS_RW" ] && ! mount | grep -q "/etc/hosts"; then
    mount --bind "$HOSTS_RW" /etc/hosts
fi
BINDEOF
chmod +x "$BOOT_SCRIPT"

if [ ! -f "$PROFILE_FILE" ]; then
    touch "$PROFILE_FILE"
fi
if ! grep -qF "bind-hosts.sh" "$PROFILE_FILE"; then
    printf '\n# Lisa: auto bind mount /etc/hosts from RW area\n[ -x /home/root/lisa/bind-hosts.sh ] && /home/root/lisa/bind-hosts.sh 2>/dev/null\n' >> "$PROFILE_FILE"
fi
if ! grep -qF '/home/root/lisa' "$PROFILE_FILE" || ! grep -qF 'PATH=' "$PROFILE_FILE"; then
    printf '\n# Lisa: add /home/root/lisa to PATH\nexport PATH="/home/root/lisa:$PATH"\n' >> "$PROFILE_FILE"
fi
echo "  .profile hook registered"
REMOTE_BOOT
echo "  OK"

# -- 11) Create start scripts (daemon + agent) --
echo ""
echo "[11/$TOTAL] Creating start scripts..."
ssh "${TARGET}" bash << 'REMOTE_START'
# daemon mode - background service (gateway + channels + scheduler)
cat > /home/root/lisa/start-lisa.sh << 'STARTEOF'
#!/bin/sh
cd /home/root/lisa
[ -x /home/root/lisa/bind-hosts.sh ] && /home/root/lisa/bind-hosts.sh 2>/dev/null
export ZEROCLAW_CONFIG_DIR="/home/root/.zeroclaw"
export PATH="/home/root/lisa:$PATH"
[ -f /home/root/lisa/lisa.env ] && . /home/root/lisa/lisa.env
exec /home/root/lisa/zeroclaw daemon
STARTEOF
chmod +x /home/root/lisa/start-lisa.sh

# agent mode - interactive chat / single message
# Note: agent CLI --temperature default(0.7) ignores config, so -t 1.0 is explicit
cat > /home/root/lisa/lisa-agent.sh << 'AGENTEOF'
#!/bin/sh
cd /home/root/lisa
[ -x /home/root/lisa/bind-hosts.sh ] && /home/root/lisa/bind-hosts.sh 2>/dev/null
export ZEROCLAW_CONFIG_DIR="/home/root/.zeroclaw"
export PATH="/home/root/lisa:$PATH"
[ -f /home/root/lisa/lisa.env ] && . /home/root/lisa/lisa.env
if [ -n "$1" ]; then
    exec /home/root/lisa/zeroclaw agent -t 1.0 -m "$*"
else
    exec /home/root/lisa/zeroclaw agent -t 1.0
fi
AGENTEOF
chmod +x /home/root/lisa/lisa-agent.sh
REMOTE_START
echo "  start-lisa.sh (daemon)"
echo "  lisa-agent.sh (agent)"
echo "  OK"

# -- 12) Verify deployment --
echo ""
echo "[12/$TOTAL] Verifying deployment..."
ssh "${TARGET}" bash << 'VERIFY'
echo "  Binary:  $(ls -lh /home/root/lisa/zeroclaw 2>/dev/null | awk '{print $5}')"
echo "  Config:  $(ls -lh /home/root/.zeroclaw/config.toml 2>/dev/null | awk '{print $5}')"
echo "  Hosts:   $(grep -c 'tvdevops.openai.azure.com' /etc/hosts 2>/dev/null || echo 0) entries"
echo "  Mount:   $(mount | grep -c '/etc/hosts' || echo 0) bind(s)"
VERIFY
echo "  OK"

# -- 13) Post-deploy functional tests --
echo ""
echo "[13-14/$TOTAL] Running post-deploy tests..."
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

# Agent mode: single message
echo ""
echo "  [agent mode]"
AGENT_RESULT=$(ssh "${TARGET}" '/home/root/lisa/lisa-agent.sh hi' 2>&1 | head -20) || true
AGENT_EXIT=$?
run_test "agent: single message" "$AGENT_RESULT" "$AGENT_EXIT"

# Skill: device-control - foreground app
echo ""
echo "  [device-control skill]"
DC_RESULT=$(ssh "${TARGET}" "luna-send -n 1 luna://com.webos.applicationManager/getForegroundAppInfo '{}'" 2>&1) || true
DC_EXIT=$?
run_test "device-control: getForegroundAppInfo" "$DC_RESULT" "$DC_EXIT"

# Skill: device-control - volume
VOL_RESULT=$(ssh "${TARGET}" "luna-send -n 1 luna://com.webos.service.audio/master/getVolume '{}'" 2>&1) || true
VOL_EXIT=$?
run_test "device-control: getVolume" "$VOL_RESULT" "$VOL_EXIT"

# Skill: weather - wttr.in (requires internet)
echo ""
echo "  [weather skill]"
ssh "${TARGET}" "curl -s --max-time 5 -o /dev/null 'http://wttr.in'" >/dev/null 2>&1 && HAS_INET=true || HAS_INET=false
if [ "$HAS_INET" = "true" ]; then
    WEATHER_RESULT=$(ssh "${TARGET}" "curl -s --max-time 10 'wttr.in/Seoul?format=%c+%t'" 2>&1) || true
    WEATHER_EXIT=$?
    run_test "weather: wttr.in query" "$WEATHER_RESULT" "$WEATHER_EXIT"
else
    echo "  [SKIP] weather: target has no internet access"
fi

# Skill: calendar - gog CLI check
echo ""
echo "  [calendar skill]"
CAL_RESULT=$(ssh "${TARGET}" "test -x /home/root/lisa/gog && echo 'ok' || echo 'gog not installed (skip)'" 2>&1)
if echo "$CAL_RESULT" | grep -q "not installed"; then
    echo "  [SKIP] calendar: gog not installed"
else
    # Load lisa.env for GOG_KEYRING_PASSWORD + GOG_ACCOUNT
    CAL_ENV=". /home/root/lisa/lisa.env 2>/dev/null; export PATH=/home/root/lisa:\$PATH"

    # Test 1: calendar list
    CAL_LIST=$(ssh "${TARGET}" "${CAL_ENV}; gog calendar calendars 2>&1 | head -5") || true
    CAL_LIST_EXIT=$?
    run_test "calendar: gog calendar calendars" "$CAL_LIST" "$CAL_LIST_EXIT"

    # Test 2: today's events
    CAL_EVENTS=$(ssh "${TARGET}" "${CAL_ENV}; gog calendar events primary --from \$(date +%Y-%m-%dT00:00:00) --to \$(date +%Y-%m-%dT23:59:59) --json 2>&1 | head -10") || true
    CAL_EVENTS_EXIT=$?
    run_test "calendar: today's events" "$CAL_EVENTS" "$CAL_EVENTS_EXIT"
fi

# Daemon mode: start -> status check
echo ""
echo "  [daemon mode]"
ssh "${TARGET}" bash <<'DAEMON_START'
cd /home/root/lisa
[ -x bind-hosts.sh ] && ./bind-hosts.sh 2>/dev/null
export ZEROCLAW_CONFIG_DIR="/home/root/.zeroclaw"
nohup ./zeroclaw daemon > /tmp/lisa-daemon-test.log 2>&1 &
DAEMON_START
sleep 3
DAEMON_STATUS=$(ssh "${TARGET}" '/home/root/lisa/zeroclaw status' 2>&1) || true
DAEMON_EXIT=$?
run_test "daemon: zeroclaw status" "$DAEMON_STATUS" "$DAEMON_EXIT"

# Gateway /health (no auth required)
GW_PORT=$(ssh "${TARGET}" "grep '^port' ${TARGET_ZEROCLAW_DIR}/config.toml 2>/dev/null | head -1 | sed 's/[^0-9]//g'" 2>/dev/null)
GW_PORT="${GW_PORT:-42617}"
HEALTH_RESULT=$(ssh "${TARGET}" "curl -s --max-time 5 'http://127.0.0.1:${GW_PORT}/health'" 2>&1) || true
HEALTH_EXIT=$?
if echo "$HEALTH_RESULT" | grep -q '"status"'; then
    run_test "gateway: /health (port ${GW_PORT})" "$HEALTH_RESULT" "$HEALTH_EXIT"
else
    run_test "gateway: /health (port ${GW_PORT})" "$HEALTH_RESULT" "1"
fi

# Gateway /pair + /api/chat (pairing -> chat test)
PAIR_CODE=$(ssh "${TARGET}" "sed -n 's/.*X-Pairing-Code: *\([0-9]*\).*/\1/p' /tmp/lisa-daemon-test.log | head -1" 2>/dev/null) || true
if [ -n "$PAIR_CODE" ]; then
    PAIR_RESULT=$(ssh "${TARGET}" "curl -s -X POST http://127.0.0.1:${GW_PORT}/pair -H 'X-Pairing-Code: ${PAIR_CODE}'" 2>&1) || true
    if echo "$PAIR_RESULT" | grep -q '"paired":true'; then
        run_test "gateway: /pair" "paired (code ${PAIR_CODE})" "0"
        GW_TOKEN=$(echo "$PAIR_RESULT" | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')
        if [ -n "$GW_TOKEN" ]; then
            CHAT_RESULT=$(ssh "${TARGET}" "curl -s --max-time 30 -X POST http://127.0.0.1:${GW_PORT}/api/chat \
              -H 'Content-Type: application/json' \
              -H 'Authorization: Bearer ${GW_TOKEN}' \
              -d '{\"message\":\"hi\"}'" 2>&1) || true
            CHAT_EXIT=$?
            if echo "$CHAT_RESULT" | grep -q '"reply"'; then
                run_test "gateway: /api/chat" "$(echo "$CHAT_RESULT" | sed -n 's/.*"reply":"\([^"]*\)".*/\1/p' | head -c 60)" "$CHAT_EXIT"
            else
                run_test "gateway: /api/chat" "$CHAT_RESULT" "1"
            fi
        fi
    else
        run_test "gateway: /pair" "$PAIR_RESULT" "1"
    fi
else
    echo "  [SKIP] gateway: failed to extract pairing code"
fi

ssh "${TARGET}" 'pkill -f "zeroclaw daemon"; rm -f /tmp/lisa-daemon-test.log' 2>/dev/null || true

# Telegram Bot API check
echo ""
echo "  [telegram channel]"
TG_TOKEN=$(ssh "${TARGET}" "grep 'bot_token' ${TARGET_ZEROCLAW_DIR}/config.toml 2>/dev/null | sed 's/.*= *\"//;s/\".*//' | head -1") || true
if [ -z "$TG_TOKEN" ] || [ "$TG_TOKEN" = "YOUR_BOT_TOKEN" ]; then
    echo "  [SKIP] telegram: bot_token not configured"
elif [ "$HAS_INET" != "true" ]; then
    echo "  [SKIP] telegram: target has no internet access"
else
    TG_RESULT=$(ssh "${TARGET}" "curl -s --max-time 10 'https://api.telegram.org/bot${TG_TOKEN}/getMe'" 2>&1) || true
    TG_EXIT=$?
    if echo "$TG_RESULT" | grep -q '"ok":true'; then
        TG_NAME=$(echo "$TG_RESULT" | python3 -c "import sys,json;print(json.load(sys.stdin)['result']['first_name'])" 2>/dev/null || echo "unknown")
        run_test "telegram: Bot API getMe (${TG_NAME})" "$TG_RESULT" "$TG_EXIT"
    else
        run_test "telegram: Bot API getMe" "$TG_RESULT" "1"
    fi
fi

# Test summary
echo ""
echo "  ------------------------"
echo "  Results: ${TEST_PASS} passed / ${TEST_FAIL} failed"
if [ "$TEST_FAIL" -gt 0 ]; then
    echo "  WARNING: Some tests failed. Check logs above."
fi

echo ""
echo "========================================================"
echo "  Lisa target deploy complete!"
echo "========================================================"
echo ""
echo "  Target:  ${TARGET}"
echo "  Binary:  ${TARGET_DEPLOY_DIR}/zeroclaw"
echo "  Config:  ${TARGET_ZEROCLAW_DIR}/config.toml"
echo ""
echo "To use:"
echo "  ssh ${TARGET} '/home/root/lisa/start-lisa.sh'      # daemon"
echo "  ssh ${TARGET} '/home/root/lisa/lisa-agent.sh'      # agent"
echo "  ssh ${TARGET} '/home/root/lisa/lisa-agent.sh hi!'  # message"
echo "  ssh ${TARGET} '/home/root/lisa/zeroclaw status'    # status"
