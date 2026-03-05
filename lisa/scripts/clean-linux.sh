#!/usr/bin/env bash
set -uo pipefail

# ─────────────────────────────────────────────
# Lisa Linux clean script
# Removes all deployed files from local or remote Linux
# Usage:
#   ./clean-linux.sh              # clean local
#   ./clean-linux.sh <IP>         # clean remote via SSH
# ─────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LISA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
GOG_CONFIG_LOCAL="${HOME}/.config/gogcli"

REMOTE=false
TARGET_IP=""
TARGET=""
DEPLOY_DIR=""
ZEROCLAW_DIR=""
TARGET_GOG_DIR=""

echo ""
echo "Lisa Linux clean"
echo "================"
echo ""

# -- Determine mode --
if [ -n "${1:-}" ]; then
    TARGET_IP="$1"
    REMOTE=true
    TARGET="$(whoami)@${TARGET_IP}"
    echo "  Mode: remote (${TARGET})"
else
    echo "  Mode: local"
fi

# -- Resolve paths --
if [ "$REMOTE" = true ]; then
    REMOTE_HOME=$(ssh "${TARGET}" 'echo $HOME')
    DEPLOY_DIR="${REMOTE_HOME}/lisa"
    ZEROCLAW_DIR="${REMOTE_HOME}/.zeroclaw"
    TARGET_GOG_DIR="${REMOTE_HOME}/.config/gogcli"
else
    DEPLOY_DIR="${HOME}/lisa"
    ZEROCLAW_DIR="${HOME}/.zeroclaw"
    TARGET_GOG_DIR="${HOME}/.config/gogcli"
fi

# ── Helpers ──
run_cmd() {
    if [ "$REMOTE" = true ]; then
        ssh "${TARGET}" "$1"
    else
        bash -c "$1"
    fi
}

# -- Ask: clean deployed files? --
echo ""
echo "Deployed files to remove:"
echo "  ${DEPLOY_DIR}/"
echo "  ${ZEROCLAW_DIR}/"
echo "  ${TARGET_GOG_DIR}/"
echo "  /etc/hosts Lisa entries"
echo "  .bashrc PATH hook"
echo ""
CLEAN_DEPLOY=false
if [ "$REMOTE" = true ]; then
    read -rp "Clean deployed files on ${TARGET}? [y/N]: " CONFIRM
else
    read -rp "Clean local deployed files? [y/N]: " CONFIRM
fi
if [ "${CONFIRM}" = "y" ] || [ "${CONFIRM}" = "Y" ]; then
    CLEAN_DEPLOY=true
fi

# -- Ask: clean local gog tokens? (only when cleaning local or when local has tokens) --
CLEAN_LOCAL_TOKENS=false
if [ "$REMOTE" = true ]; then
    echo ""
    echo "Local files to remove:"
    echo "  ${GOG_CONFIG_LOCAL}/keyring/ (OAuth tokens)"
    echo ""
    read -rp "Remove local gog tokens? [y/N]: " CONFIRM
    if [ "${CONFIRM}" = "y" ] || [ "${CONFIRM}" = "Y" ]; then
        CLEAN_LOCAL_TOKENS=true
    fi
fi

# -- Nothing selected --
if [ "$CLEAN_DEPLOY" = false ] && [ "$CLEAN_LOCAL_TOKENS" = false ]; then
    echo ""
    echo "Nothing selected. Cancelled."
    exit 0
fi

# ── Deploy clean ──
if [ "$CLEAN_DEPLOY" = true ]; then

    # -- 1) Stop running processes --
    echo ""
    echo "[1/4] Stopping Lisa processes..."
    run_cmd 'pkill -f "zeroclaw" 2>/dev/null || true'
    echo "  OK"

    # -- 2) Clean .bashrc PATH hook --
    echo ""
    echo "[2/4] Cleaning .bashrc PATH hook..."
    run_cmd "sed -i '/# Lisa: add to PATH/d' ~/.bashrc 2>/dev/null; sed -i '\|${DEPLOY_DIR}|d' ~/.bashrc 2>/dev/null; echo '  .bashrc cleaned'"
    echo "  OK"

    # -- 3) Remove /etc/hosts Lisa entries --
    echo ""
    echo "[3/4] Removing /etc/hosts Lisa entries..."
    HOSTS_CHECK=$(run_cmd "grep -c 'Azure OpenAI endpoint for Lisa' /etc/hosts 2>/dev/null || echo 0")
    if [ "$HOSTS_CHECK" != "0" ]; then
        if [ "$REMOTE" = true ]; then
            ssh "${TARGET}" "sudo sed -i '/# Azure OpenAI endpoint for Lisa/d' /etc/hosts; sudo sed -i '/tvdevops.openai.azure.com/d' /etc/hosts"
        else
            sudo sed -i '/# Azure OpenAI endpoint for Lisa/d' /etc/hosts
            sudo sed -i '/tvdevops.openai.azure.com/d' /etc/hosts
        fi
        echo "  /etc/hosts entries removed"
    else
        echo "  no Lisa entries found"
    fi
    echo "  OK"

    # -- 4) Remove directories --
    echo ""
    echo "[4/4] Removing deployed files..."
    run_cmd "rm -rf '${DEPLOY_DIR}' && echo '  removed: ${DEPLOY_DIR}'"
    run_cmd "rm -rf '${ZEROCLAW_DIR}' && echo '  removed: ${ZEROCLAW_DIR}'"
    run_cmd "rm -rf '${TARGET_GOG_DIR}' && echo '  removed: ${TARGET_GOG_DIR}'"
    run_cmd "rm -f /tmp/lisa-daemon-test.log 2>/dev/null"
    echo "  OK"

fi

# ── Local token clean (remote mode only) ──
if [ "$CLEAN_LOCAL_TOKENS" = true ]; then

    echo ""
    echo "[*] Cleaning local gog tokens..."
    if [ -d "$GOG_CONFIG_LOCAL/keyring" ]; then
        rm -rf "$GOG_CONFIG_LOCAL/keyring"
        echo "  removed: ${GOG_CONFIG_LOCAL}/keyring/"
    else
        echo "  no keyring directory found"
    fi
    echo "  OK"

fi

# ── Summary ──
echo ""
echo "========================================"
echo "  Clean complete!"
echo "========================================"
echo ""
if [ "$CLEAN_DEPLOY" = true ]; then
    if [ "$REMOTE" = true ]; then
        echo "  Remote ${TARGET} has been reset."
    else
        echo "  Local deployment has been reset."
    fi
fi
if [ "$CLEAN_LOCAL_TOKENS" = true ]; then
    echo "  Local gog tokens removed."
fi
echo ""
if [ "$REMOTE" = true ]; then
    echo "  To redeploy: ./deploy-linux.sh ${TARGET_IP}"
else
    echo "  To redeploy: ./deploy-linux.sh"
fi
echo ""
