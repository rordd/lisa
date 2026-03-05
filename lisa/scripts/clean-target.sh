#!/usr/bin/env bash
set -uo pipefail

# ─────────────────────────────────────────────
# Lisa target clean script
# Removes all deployed files from target and local gog tokens
# ─────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LISA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
GOG_CONFIG_LOCAL="${HOME}/.config/gogcli"

# Target settings
TARGET_USER="root"
TARGET_DEPLOY_DIR="/home/root/lisa"
TARGET_ZEROCLAW_DIR="/home/root/.zeroclaw"
TARGET_GOG_CONFIG="/home/root/.config/gogcli"

echo ""
echo "Lisa target clean"
echo "================="
echo ""

# -- Target IP --
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

# -- Ask: clean target? --
echo ""
echo "Target files to remove:"
echo "  ${TARGET_DEPLOY_DIR}/"
echo "  ${TARGET_ZEROCLAW_DIR}/"
echo "  ${TARGET_GOG_CONFIG}/"
echo "  /etc/hosts bind mount"
echo "  .profile hooks"
echo ""
CLEAN_TARGET=false
read -rp "Clean target ${TARGET}? [y/N]: " CONFIRM
if [ "${CONFIRM}" = "y" ] || [ "${CONFIRM}" = "Y" ]; then
    CLEAN_TARGET=true
fi

# -- Ask: clean local tokens? --
echo ""
echo "Local files to remove:"
echo "  ${GOG_CONFIG_LOCAL}/keyring/ (OAuth tokens)"
echo ""
CLEAN_LOCAL=false
read -rp "Remove local gog tokens? [y/N]: " CONFIRM
if [ "${CONFIRM}" = "y" ] || [ "${CONFIRM}" = "Y" ]; then
    CLEAN_LOCAL=true
fi

# -- Nothing selected --
if [ "$CLEAN_TARGET" = false ] && [ "$CLEAN_LOCAL" = false ]; then
    echo ""
    echo "Nothing selected. Cancelled."
    exit 0
fi

# ── Target clean ──
if [ "$CLEAN_TARGET" = true ]; then

    # -- 1) Stop running processes --
    echo ""
    echo "[1/5] Stopping Lisa processes on target..."
    ssh "${TARGET}" 'pkill -f "zeroclaw" 2>/dev/null || true'
    echo "  OK"

    # -- 2) Remove /etc/hosts bind mount --
    echo ""
    echo "[2/5] Removing /etc/hosts bind mount..."
    ssh "${TARGET}" bash <<'REMOTE_HOSTS'
if mount | grep -q "/etc/hosts"; then
    umount /etc/hosts 2>/dev/null && echo "  bind mount removed" || echo "  WARNING: umount failed"
else
    echo "  no bind mount active"
fi
# Remove hosts file (optional, keep original /etc/hosts intact)
rm -f /home/root/hosts 2>/dev/null
REMOTE_HOSTS
    echo "  OK"

    # -- 3) Clean .profile hooks --
    echo ""
    echo "[3/5] Cleaning .profile hooks..."
    ssh "${TARGET}" bash <<'REMOTE_PROFILE'
PROFILE_FILE="/home/root/.profile"
if [ -f "$PROFILE_FILE" ]; then
    # Remove Lisa-related lines
    sed -i '/# Lisa:/d' "$PROFILE_FILE"
    sed -i '/bind-hosts\.sh/d' "$PROFILE_FILE"
    sed -i '/\/home\/root\/lisa/d' "$PROFILE_FILE"
    # Remove empty trailing lines
    sed -i -e :a -e '/^\n*$/{$d;N;ba' -e '}' "$PROFILE_FILE"
    echo "  .profile cleaned"
else
    echo "  .profile not found"
fi
REMOTE_PROFILE
    echo "  OK"

    # -- 4) Remove target directories --
    echo ""
    echo "[4/5] Removing target files..."
    ssh "${TARGET}" bash <<REMOTE_CLEAN
rm -rf "${TARGET_DEPLOY_DIR}" && echo "  removed: ${TARGET_DEPLOY_DIR}"
rm -rf "${TARGET_ZEROCLAW_DIR}" && echo "  removed: ${TARGET_ZEROCLAW_DIR}"
rm -rf "${TARGET_GOG_CONFIG}" && echo "  removed: ${TARGET_GOG_CONFIG}"
rm -f /tmp/lisa-daemon-test.log 2>/dev/null
REMOTE_CLEAN
    echo "  OK"

fi

# ── Local clean ──
if [ "$CLEAN_LOCAL" = true ]; then

    echo ""
    echo "[5/5] Cleaning local gog tokens..."
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
if [ "$CLEAN_TARGET" = true ]; then
    echo "  Target ${TARGET} has been reset."
fi
if [ "$CLEAN_LOCAL" = true ]; then
    echo "  Local gog tokens removed."
fi
echo ""
echo "  To redeploy: ./deploy-target.sh ${TARGET_IP}"
echo ""
