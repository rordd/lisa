#!/usr/bin/env bash
set -euo pipefail

# ─────────────────────────────────────────────
# Lisa setup — install + configure + onboard
# ─────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LISA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_DIR="$(cd "$LISA_DIR/.." && pwd)"
CONFIG_TEMPLATE="$LISA_DIR/config/config.default.toml"
ENV_EXAMPLE="$LISA_DIR/profiles/.env.example"

# Defaults
MODE=""
TARGET=""
BINARY_PATH=""
PROFILE="lisa"
SKILLS_ONLY=false
TARGET_USER="root"
TARGET_DEPLOY_DIR="/home/root/lisa"
TARGET_ZEROCLAW_DIR="/home/root/.zeroclaw"
ZEROCLAW_DIR="${ZEROCLAW_CONFIG_DIR:-$HOME/.zeroclaw}"

usage() {
    cat << EOF
Usage: setup.sh <mode> [options]

Modes (required):
  --source              Build from source (cargo build)
  --release             Download from GitHub Releases
  --binary <path>       Use specified binary

Options:
  --target <IP>         Deploy to remote target via SSH
                        (without --target: setup on this host)
  --profile <name>      Profile to apply (default: lisa)
  --skills-only         Only update skills (skip everything else)

Examples:
  setup.sh --source                              # build + full setup locally
  setup.sh --release                             # download + full setup locally
  setup.sh --binary ./zeroclaw                   # use binary + full setup locally
  setup.sh --source --target 192.168.1.50        # cross-build + deploy to target
  setup.sh --release --target 192.168.1.50       # download + deploy to target
  setup.sh --source --skills-only                # rebuild + update skills only
EOF
    exit 1
}

# ── Parse args ──
while [[ $# -gt 0 ]]; do
    case "$1" in
        --source)      MODE="source"; shift ;;
        --release)     MODE="release"; shift ;;
        --binary)      MODE="binary"; BINARY_PATH="${2:-}"; shift 2 || usage ;;
        --target)      TARGET="${2:-}"; shift 2 || usage ;;
        --profile)     PROFILE="${2:-}"; shift 2 || usage ;;
        --skills-only) SKILLS_ONLY=true; shift ;;
        -h|--help)     usage ;;
        *)             echo "Unknown option: $1"; usage ;;
    esac
done

[[ -z "$MODE" ]] && { echo "ERROR: mode is required (--source, --release, or --binary)"; usage; }
[[ "$MODE" == "binary" && -z "$BINARY_PATH" ]] && { echo "ERROR: --binary requires a path"; usage; }

PROFILE_DIR="$LISA_DIR/profiles/$PROFILE"
if [[ ! -d "$PROFILE_DIR" ]]; then
    echo "ERROR: Profile not found: $PROFILE_DIR"
    exit 1
fi

# ── Load .env ──
ENV_FILE="$REPO_DIR/.env"
if [[ -f "$ENV_FILE" ]]; then
    # shellcheck disable=SC1090
    source "$ENV_FILE"
fi

# Minimum check
if [[ -z "${ZEROCLAW_API_KEY:-}" && "$SKILLS_ONLY" == false ]]; then
    echo "ERROR: ZEROCLAW_API_KEY not set"
    echo "  cp $ENV_EXAMPLE .env && edit .env"
    exit 1
fi

# ── Target setup ──
if [[ -n "$TARGET" ]]; then
    TARGET_HOST="${TARGET_USER}@${TARGET}"
    if ! ssh -o ConnectTimeout=5 -o BatchMode=yes "$TARGET_HOST" "echo ok" >/dev/null 2>&1; then
        echo "ERROR: Cannot SSH to $TARGET_HOST"
        exit 1
    fi
    WS="$TARGET_ZEROCLAW_DIR/workspace"
else
    WS="$ZEROCLAW_DIR/workspace"
fi

TOTAL=5
[[ "$SKILLS_ONLY" == true ]] && TOTAL=2
STEP=0

echo ""
echo "Lisa Setup"
echo "=========="
echo "  Mode:       $MODE"
echo "  Profile:    $PROFILE"
echo "  Target:     ${TARGET:-localhost}"
echo "  Skills only: $SKILLS_ONLY"
echo ""

# ── Helpers ──
ensure_dir() {
    if [[ -n "$TARGET" ]]; then
        ssh "$TARGET_HOST" "mkdir -p $1"
    else
        mkdir -p "$1"
    fi
}

copy_file() {
    local src="$1" dest="$2"
    if [[ -n "$TARGET" ]]; then
        scp -q "$src" "$TARGET_HOST:$dest"
    else
        cp "$src" "$dest"
    fi
}

copy_dir() {
    local src="$1" dest="$2"
    if [[ -n "$TARGET" ]]; then
        scp -qr "$src" "$TARGET_HOST:$dest"
    else
        cp -r "$src" "$dest"
    fi
}

# ════════════════════════════════════════════
# Step 1: Binary
# ════════════════════════════════════════════
STEP=$((STEP + 1))
echo "[$STEP/$TOTAL] Preparing binary..."

case "$MODE" in
    source)
        cd "$REPO_DIR"
        if [[ -n "$TARGET" ]]; then
            echo "  Cross-compiling for ARM64..."
            cross build --release --target aarch64-unknown-linux-gnu
            BINARY_PATH="$REPO_DIR/target/aarch64-unknown-linux-gnu/release/zeroclaw"
        else
            echo "  Building from source..."
            cargo build --release
            BINARY_PATH="$REPO_DIR/target/release/zeroclaw"
        fi
        ;;
    release)
        if [[ -n "$TARGET" ]]; then
            ASSET="zeroclaw-aarch64-unknown-linux-gnu"
        else
            case "$(uname -s)-$(uname -m)" in
                Darwin-arm64)  ASSET="zeroclaw-aarch64-apple-darwin" ;;
                Darwin-x86_64) ASSET="zeroclaw-x86_64-apple-darwin" ;;
                Linux-x86_64)  ASSET="zeroclaw-x86_64-unknown-linux-gnu" ;;
                Linux-aarch64) ASSET="zeroclaw-aarch64-unknown-linux-gnu" ;;
                *) echo "  ERROR: Unsupported platform"; exit 1 ;;
            esac
        fi
        BINARY_PATH="/tmp/zeroclaw-download"
        gh release download --repo rordd/lisa --pattern "$ASSET" --output "$BINARY_PATH" --clobber
        chmod +x "$BINARY_PATH"
        ;;
    binary)
        [[ ! -f "$BINARY_PATH" ]] && { echo "  ERROR: Binary not found: $BINARY_PATH"; exit 1; }
        ;;
esac

# Install binary
if [[ -n "$TARGET" ]]; then
    ensure_dir "$TARGET_DEPLOY_DIR"
    scp -q "$BINARY_PATH" "$TARGET_HOST:$TARGET_DEPLOY_DIR/zeroclaw"
    ssh "$TARGET_HOST" "chmod +x $TARGET_DEPLOY_DIR/zeroclaw"
    echo "  Installed to $TARGET_HOST:$TARGET_DEPLOY_DIR/zeroclaw"
else
    cargo install --path "$REPO_DIR" --force 2>/dev/null || {
        mkdir -p "$HOME/.local/bin"
        cp "$BINARY_PATH" "$HOME/.local/bin/zeroclaw"
        chmod +x "$HOME/.local/bin/zeroclaw"
    }
    echo "  Installed"
fi
echo ""

if [[ "$SKILLS_ONLY" == true ]]; then
    # Skip to skills
    STEP=$((STEP + 1))
    echo "[$STEP/$TOTAL] Updating skills..."
    ensure_dir "$WS/skills"
    goto_skills=true
else
    goto_skills=false

    # ════════════════════════════════════════════
    # Step 2: Config
    # ════════════════════════════════════════════
    STEP=$((STEP + 1))
    echo "[$STEP/$TOTAL] Generating config..."

    if [[ -n "$TARGET" ]]; then
        ensure_dir "$TARGET_ZEROCLAW_DIR"
        ensure_dir "$WS"
        CONFIG_DEST="/tmp/lisa-config-$$"
        mkdir -p "$CONFIG_DEST"
        CONFIG_PATH="$CONFIG_DEST/config.toml"
    else
        ensure_dir "$ZEROCLAW_DIR"
        ensure_dir "$WS"
        CONFIG_PATH="$ZEROCLAW_DIR/config.toml"
        # Backup existing
        [[ -f "$CONFIG_PATH" ]] && cp "$CONFIG_PATH" "$CONFIG_PATH.bak.$(date +%s)"
    fi

    cp "$CONFIG_TEMPLATE" "$CONFIG_PATH"

    # Inject Telegram
    if [[ -n "${TELEGRAM_BOT_TOKEN:-}" ]]; then
        cat >> "$CONFIG_PATH" << EOF

[channels_config.telegram]
bot_token = "${TELEGRAM_BOT_TOKEN}"
allowed_users = [$(if [ -n "${TELEGRAM_ALLOWED_USERS:-}" ]; then echo "${TELEGRAM_ALLOWED_USERS}" | sed 's/,/", "/g; s/^/"/; s/$/"/'; fi)]
mention_only = ${TELEGRAM_MENTION_ONLY:-true}
EOF
        echo "  Telegram configured"
    fi

    # Inject Azure OpenAI
    if [[ -n "${AZURE_OPENAI_BASE_URL:-}" ]]; then
        local_key="${AZURE_OPENAI_API_KEY:-${ZEROCLAW_API_KEY:-}}"
        local_auth="${AZURE_OPENAI_AUTH_HEADER:-api-key}"
        cat >> "$CONFIG_PATH" << EOF

[model_providers.azure]
name = "openai"
base_url = "${AZURE_OPENAI_BASE_URL}"
auth_header = "${local_auth}"
api_key = "${local_key}"
EOF
        echo "  Azure OpenAI configured"
    fi

    chmod 600 "$CONFIG_PATH"

    if [[ -n "$TARGET" ]]; then
        scp -q "$CONFIG_PATH" "$TARGET_HOST:$TARGET_ZEROCLAW_DIR/config.toml"
        ssh "$TARGET_HOST" "chmod 600 $TARGET_ZEROCLAW_DIR/config.toml"
        # Transfer .env
        if [[ -f "$ENV_FILE" ]]; then
            scp -q "$ENV_FILE" "$TARGET_HOST:$TARGET_DEPLOY_DIR/.env"
            ssh "$TARGET_HOST" "chmod 600 $TARGET_DEPLOY_DIR/.env"
        fi
        rm -rf "$CONFIG_DEST"
    fi
    echo "  OK"
    echo ""

    # ════════════════════════════════════════════
    # Step 3: Profile
    # ════════════════════════════════════════════
    STEP=$((STEP + 1))
    echo "[$STEP/$TOTAL] Applying profile ($PROFILE)..."

    for f in SOUL.md AGENTS.md; do
        [[ -f "$PROFILE_DIR/$f" ]] && copy_file "$PROFILE_DIR/$f" "$WS/$f" && echo "  $f"
    done

    # USER.md — don't overwrite if exists
    if [[ -n "$TARGET" ]]; then
        HAS_USER=$(ssh "$TARGET_HOST" "test -f $WS/USER.md && echo yes || echo no")
    else
        HAS_USER=$([[ -f "$WS/USER.md" ]] && echo yes || echo no)
    fi

    if [[ "$HAS_USER" == "no" && -f "$PROFILE_DIR/USER.md.example" ]]; then
        copy_file "$PROFILE_DIR/USER.md.example" "$WS/USER.md"
        echo "  USER.md (from example — edit with your info)"
    else
        echo "  USER.md (exists, skipped)"
    fi
    echo "  OK"
    echo ""
fi

# ════════════════════════════════════════════
# Step 4 (or 2): Skills
# ════════════════════════════════════════════
if [[ "$goto_skills" == false ]]; then
    STEP=$((STEP + 1))
    echo "[$STEP/$TOTAL] Installing skills..."
    ensure_dir "$WS/skills"
fi

if [[ -d "$PROFILE_DIR/skills" ]]; then
    SKILL_COUNT=0
    for skill_dir in "$PROFILE_DIR/skills"/*/; do
        [[ -d "$skill_dir" ]] || continue
        skill_name=$(basename "$skill_dir")
        ensure_dir "$WS/skills/$skill_name"
        copy_dir "$skill_dir"* "$WS/skills/$skill_name/"
        echo "  $skill_name"
        SKILL_COUNT=$((SKILL_COUNT + 1))
    done
    echo "  $SKILL_COUNT skill(s) installed"
fi
echo ""

# ════════════════════════════════════════════
# Step 5: Skill dependencies + test
# ════════════════════════════════════════════
if [[ "$SKILLS_ONLY" == false ]]; then
    STEP=$((STEP + 1))
    echo "[$STEP/$TOTAL] Skill dependencies & test..."

    # ── gog (Google Calendar) ──
    if [[ -d "$PROFILE_DIR/skills/calendar" ]]; then
        echo ""
        echo "  [gog] Google Calendar"

        GOG_ACCOUNT="${GOG_ACCOUNT:-}"
        GOG_KEYRING_PASSWORD="${GOG_KEYRING_PASSWORD:-}"
        GOG_KEYRING_BACKEND="${GOG_KEYRING_BACKEND:-file}"

        if [[ -z "$GOG_ACCOUNT" ]]; then
            if [[ -t 0 ]]; then
                read -rp "  Google account (e.g. you@gmail.com): " GOG_ACCOUNT
            else
                echo "  Skipped (no GOG_ACCOUNT in .env, non-interactive)"
                GOG_ACCOUNT=""
            fi
        else
            echo "  Account: $GOG_ACCOUNT (from .env)"
        fi

        if [[ -n "$GOG_ACCOUNT" && -z "$GOG_KEYRING_PASSWORD" ]]; then
            if [[ -t 0 ]]; then
                read -rsp "  Keyring password: " GOG_KEYRING_PASSWORD
                echo ""
            else
                echo "  Skipped (no GOG_KEYRING_PASSWORD in .env)"
            fi
        elif [[ -n "$GOG_ACCOUNT" ]]; then
            echo "  Keyring password: (from .env)"
        fi

        if [[ -n "$GOG_ACCOUNT" ]]; then
            if [[ -n "$TARGET" ]]; then
                GOG_CONFIG_LOCAL="${HOME}/.config/gogcli"
                if [[ -d "$GOG_CONFIG_LOCAL" ]]; then
                    copy_dir "$GOG_CONFIG_LOCAL" "$TARGET_ZEROCLAW_DIR/.config/gogcli"
                    echo "  gog config transferred"
                else
                    echo "  WARNING: No local gog config. Run on target:"
                    echo "    gog auth add $GOG_ACCOUNT --services calendar --manual"
                fi
            else
                if command -v gog &>/dev/null && GOG_KEYRING_PASSWORD="$GOG_KEYRING_PASSWORD" GOG_KEYRING_BACKEND="$GOG_KEYRING_BACKEND" gog auth list 2>/dev/null | grep -q "$GOG_ACCOUNT"; then
                    echo "  Already authenticated ✓"
                else
                    echo "  Run: GOG_KEYRING_BACKEND=$GOG_KEYRING_BACKEND GOG_KEYRING_PASSWORD=<pw> gog auth add $GOG_ACCOUNT --services calendar --manual"
                fi
            fi

            # Save to .env if not there
            if [[ -f "$ENV_FILE" ]] && ! grep -q "^GOG_ACCOUNT=" "$ENV_FILE" 2>/dev/null; then
                cat >> "$ENV_FILE" << EOF
GOG_ACCOUNT=$GOG_ACCOUNT
GOG_KEYRING_PASSWORD=$GOG_KEYRING_PASSWORD
GOG_KEYRING_BACKEND=$GOG_KEYRING_BACKEND
EOF
                echo "  Saved to .env"
            fi
        fi
    fi

    # ── Connection test ──
    echo ""
    echo "  Testing connection..."
    if [[ -n "$TARGET" ]]; then
        GREETING=$(ssh "$TARGET_HOST" "cd $TARGET_DEPLOY_DIR && [ -f .env ] && . .env && export ZEROCLAW_CONFIG_DIR=$TARGET_ZEROCLAW_DIR && ./zeroclaw agent -m 'hi!'" 2>/dev/null | tail -1) || true
    else
        GREETING=$(zeroclaw agent -m 'hi!' 2>/dev/null | tail -1) || true
    fi

    if [[ -n "$GREETING" ]]; then
        echo "  Response: $(echo "$GREETING" | head -c 80)"
    else
        echo "  WARNING: No response (check API key)"
    fi
fi

echo ""
echo "════════════════════════════════"
echo "Lisa setup complete!"
echo "  Profile: $PROFILE"
if [[ -n "$TARGET" ]]; then
    echo "  Target:  $TARGET_HOST"
    echo "  Run:     ssh $TARGET_HOST 'cd $TARGET_DEPLOY_DIR && source .env && ./zeroclaw daemon'"
else
    echo "  Run:     cd $REPO_DIR && source .env && zeroclaw daemon"
fi
echo "════════════════════════════════"
