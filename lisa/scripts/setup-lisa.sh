#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LISA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_DIR="$(cd "$LISA_DIR/.." && pwd)"
PROFILE_DIR="$LISA_DIR/profiles/lisa"
ZEROCLAW_DIR="${ZEROCLAW_CONFIG_DIR:-$HOME/.zeroclaw}"
WORKSPACE_DIR="$ZEROCLAW_DIR/workspace"
ENV_FILE="$REPO_DIR/.env"

echo "Lisa Profile Setup"
echo "===================="

# 1) Create directories
mkdir -p "$ZEROCLAW_DIR" "$WORKSPACE_DIR"

# 2) Check and load .env file
if [ ! -f "$ENV_FILE" ]; then
    echo ""
    echo "WARNING: .env file not found."
    echo "  Create it from the example template:"
    echo "    cp lisa/profiles/.env.example .env"
    echo "  Then fill in your API keys and settings."
    exit 1
fi
# shellcheck disable=SC1090
source "$ENV_FILE"

# 3) Verify required values
if [ -z "${ZEROCLAW_API_KEY:-}" ] && [ -z "${API_KEY:-}" ]; then
    echo "ERROR: ZEROCLAW_API_KEY is not set. Check your .env file."
    exit 1
fi

# 4) Generate config.toml (shared config + secret injection)
if [ -f "$ZEROCLAW_DIR/config.toml" ]; then
    BACKUP="$ZEROCLAW_DIR/config.toml.bak.$(date +%s)"
    cp "$ZEROCLAW_DIR/config.toml" "$BACKUP"
    echo "Existing config backed up: $BACKUP"
fi

cp "$LISA_DIR/config/config.default.toml" "$ZEROCLAW_DIR/config.toml"

# Inject Telegram settings
if [ -n "${TELEGRAM_BOT_TOKEN:-}" ]; then
    cat >> "$ZEROCLAW_DIR/config.toml" << EOF

[channels_config.telegram]
bot_token = "${TELEGRAM_BOT_TOKEN}"
allowed_users = [$(if [ -n "${TELEGRAM_ALLOWED_USERS:-}" ]; then echo "${TELEGRAM_ALLOWED_USERS}" | sed 's/,/", "/g; s/^/"/; s/$/"/'; fi)]
mention_only = ${TELEGRAM_MENTION_ONLY:-true}
EOF
    echo "OK: Telegram channel configured"
fi

# Inject Azure OpenAI profile
if [ -n "${AZURE_OPENAI_BASE_URL:-}" ]; then
    AZURE_KEY="${AZURE_OPENAI_API_KEY:-${ZEROCLAW_API_KEY:-}}"
    cat >> "$ZEROCLAW_DIR/config.toml" << EOF

[model_providers.azure]
name = "openai"
base_url = "${AZURE_OPENAI_BASE_URL}"
auth_header = "api-key"
api_key = "${AZURE_KEY}"
EOF
    echo "OK: Azure OpenAI profile configured"
fi

chmod 600 "$ZEROCLAW_DIR/config.toml"

# 5) Copy shared workspace files
for f in SOUL.md AGENTS.md; do
    if [ -f "$PROFILE_DIR/$f" ]; then
        cp "$PROFILE_DIR/$f" "$WORKSPACE_DIR/$f"
        echo "OK: $f -> workspace/"
    fi
done

# 6) Copy skills
if [ -d "$PROFILE_DIR/skills" ]; then
    mkdir -p "$WORKSPACE_DIR/skills"
    cp -r "$PROFILE_DIR/skills/"* "$WORKSPACE_DIR/skills/"
    SKILL_COUNT=$(ls -d "$PROFILE_DIR/skills/"*/ 2>/dev/null | wc -l | tr -d ' ')
    echo "OK: ${SKILL_COUNT} skill(s) -> workspace/skills/"
fi

# 7) Check USER.md
if [ ! -f "$WORKSPACE_DIR/USER.md" ]; then
    echo ""
    echo "NOTE: USER.md not found. Create it from the example template:"
    echo "  cp $PROFILE_DIR/USER.md.example $WORKSPACE_DIR/USER.md"
    echo "  Then edit it with your personal information."
fi

echo ""
echo "Lisa setup complete!"
echo "  Run:    source .env && zeroclaw daemon"
echo "  Status: zeroclaw status"
