#!/usr/bin/env bash
set -euo pipefail

# ─────────────────────────────────────────────
# Lisa onboard — install, configure, deploy
# ─────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Auto-detect: bundle (flat) vs repo (lisa/scripts/)
if [[ -d "$SCRIPT_DIR/config" && -d "$SCRIPT_DIR/profiles" ]]; then
    BASE_DIR="$SCRIPT_DIR"
    REPO_DIR="$SCRIPT_DIR"
elif [[ -d "$SCRIPT_DIR/../config" && -d "$SCRIPT_DIR/../profiles" ]]; then
    BASE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
    REPO_DIR="$(cd "$BASE_DIR/.." && pwd)"
else
    echo "ERROR: Cannot find config/ and profiles/ directories"
    exit 1
fi

CONFIG_TEMPLATE="$BASE_DIR/config/config.default.toml"
PROFILE="lisa"

# Defaults
DO_BUILD=false
SCOPE="full"     # full | binary | skills | config
TARGET=""
TARGET_USER="root"
TARGET_DEPLOY_DIR="/home/root/lisa"
TARGET_ZEROCLAW_DIR="/home/root/.zeroclaw"
ZEROCLAW_DIR="${ZEROCLAW_CONFIG_DIR:-$HOME/.zeroclaw}"

usage() {
    cat << 'EOF'
Usage: onboard.sh [options]

Options:
  --build               Build from source before onboarding
  --binary              Binary only (replace + restart)
  --skills              Skills only (replace + restart)
  --config              Config only (config.toml + .env + SOUL.md + AGENTS.md + restart)
  --target <IP>         Deploy to remote target via SSH
  --profile <name>      Profile to apply (default: lisa)
  -h, --help            Show this help

No options = full onboarding (binary + config + profile + skills + dependencies)

Examples:
  onboard.sh                              # full local onboard (bundle or repo)
  onboard.sh --build                      # build + full onboard
  onboard.sh --binary                     # binary only (quick swap)
  onboard.sh --build --binary             # build + binary only
  onboard.sh --skills                     # skills only
  onboard.sh --config                     # config + profile only
  onboard.sh --target 192.168.1.50        # full onboard to target
  onboard.sh --build --target 10.0.0.1    # cross-build + full deploy
  onboard.sh --target 10.0.0.1 --skills   # skills only to target
  onboard.sh --target 10.0.0.1 --config   # config only to target
EOF
    exit 0
}

# ── Parse args ──
while [[ $# -gt 0 ]]; do
    case "$1" in
        --build)     DO_BUILD=true; shift ;;
        --binary)    SCOPE="binary"; shift ;;
        --skills)    SCOPE="skills"; shift ;;
        --config)    SCOPE="config"; shift ;;
        --target)    TARGET="${2:-}"; shift 2 || { echo "ERROR: --target requires IP"; exit 1; } ;;
        --profile)   PROFILE="${2:-}"; shift 2 || { echo "ERROR: --profile requires name"; exit 1; } ;;
        -h|--help)   usage ;;
        *)           echo "Unknown option: $1"; usage ;;
    esac
done

PROFILE_DIR="$BASE_DIR/profiles/$PROFILE"
[[ -d "$PROFILE_DIR" ]] || { echo "ERROR: Profile not found: $PROFILE_DIR"; exit 1; }

# ── Load .env ──
ENV_FILE=""
for candidate in "$REPO_DIR/.env" "$BASE_DIR/.env"; do
    [[ -f "$candidate" ]] && { ENV_FILE="$candidate"; break; }
done

if [[ -n "$ENV_FILE" ]]; then
    # shellcheck disable=SC1090
    source "$ENV_FILE"
fi

# API key check (skip for skills-only)
if [[ "$SCOPE" != "skills" && -z "${ZEROCLAW_API_KEY:-}" ]]; then
    echo "ERROR: ZEROCLAW_API_KEY not set"
    echo "  cp $BASE_DIR/profiles/.env.example .env && edit .env"
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

echo ""
echo "Lisa Onboard"
echo "============"
echo "  Scope:    $SCOPE"
echo "  Build:    $DO_BUILD"
echo "  Target:   ${TARGET:-localhost}"
echo "  Profile:  $PROFILE"
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

restart_daemon() {
    echo ""
    echo "  Restarting daemon..."
    if [[ -n "$TARGET" ]]; then
        ssh "$TARGET_HOST" "pkill -9 -f zeroclaw 2>/dev/null; sleep 1; cd $TARGET_DEPLOY_DIR && source .env && nohup ./zeroclaw daemon > /tmp/zeroclaw.log 2>&1 &"
    else
        pkill -9 -f "zeroclaw daemon" 2>/dev/null || true
        sleep 1
    fi
    echo "  Done"
}

# ══════════════════════════════════════════════
# BUILD (optional)
# ══════════════════════════════════════════════
BINARY_PATH=""
if [[ "$DO_BUILD" == true ]]; then
    echo "[Build]"
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
    echo "  Build complete"
    echo ""
fi

# ══════════════════════════════════════════════
# BINARY
# ══════════════════════════════════════════════
install_binary() {
    echo "[Binary]"

    # Find binary
    if [[ -n "$BINARY_PATH" ]]; then
        : # already set by --build
    elif [[ -f "$BASE_DIR/zeroclaw" ]]; then
        BINARY_PATH="$BASE_DIR/zeroclaw"  # bundle
    elif command -v zeroclaw &>/dev/null; then
        BINARY_PATH="$(command -v zeroclaw)"
    else
        echo "  ERROR: No binary found. Use --build or place zeroclaw in bundle."
        exit 1
    fi

    if [[ -n "$TARGET" ]]; then
        ensure_dir "$TARGET_DEPLOY_DIR"
        scp -q "$BINARY_PATH" "$TARGET_HOST:$TARGET_DEPLOY_DIR/zeroclaw"
        ssh "$TARGET_HOST" "chmod +x $TARGET_DEPLOY_DIR/zeroclaw"
        echo "  zeroclaw → $TARGET_HOST:$TARGET_DEPLOY_DIR/"
        # Dependency binaries (bin/)
        if [[ -d "$BASE_DIR/bin" ]]; then
            for dep in "$BASE_DIR/bin"/*; do
                [[ -f "$dep" ]] || continue
                scp -q "$dep" "$TARGET_HOST:$TARGET_DEPLOY_DIR/$(basename "$dep")"
                ssh "$TARGET_HOST" "chmod +x $TARGET_DEPLOY_DIR/$(basename "$dep")"
                echo "  $(basename "$dep") → $TARGET_HOST:$TARGET_DEPLOY_DIR/"
            done
        fi
    else
        # Install: copy to existing location or ~/.local/bin
        local existing
        existing="$(command -v zeroclaw 2>/dev/null || true)"
        if [[ -n "$existing" ]]; then
            cp "$BINARY_PATH" "$existing"
            chmod +x "$existing"
            echo "  Updated: $existing"
        else
            mkdir -p "$HOME/.local/bin"
            cp "$BINARY_PATH" "$HOME/.local/bin/zeroclaw"
            chmod +x "$HOME/.local/bin/zeroclaw"
            echo "  Installed to ~/.local/bin/"
        fi
        if [[ -d "$BASE_DIR/bin" ]]; then
            for dep in "$BASE_DIR/bin"/*; do
                [[ -f "$dep" ]] || continue
                cp "$dep" "$HOME/.local/bin/$(basename "$dep")"
                chmod +x "$HOME/.local/bin/$(basename "$dep")"
                echo "  $(basename "$dep") installed"
            done
        fi
        echo "  Installed"
    fi
    echo ""
}

# ══════════════════════════════════════════════
# CONFIG
# ══════════════════════════════════════════════
install_config() {
    echo "[Config]"

    if [[ -n "$TARGET" ]]; then
        ensure_dir "$TARGET_ZEROCLAW_DIR"
        ensure_dir "$WS"
        CONFIG_PATH="/tmp/lisa-config-$$.toml"
    else
        ensure_dir "$ZEROCLAW_DIR"
        ensure_dir "$WS"
        CONFIG_PATH="$ZEROCLAW_DIR/config.toml"
        [[ -f "$CONFIG_PATH" ]] && cp "$CONFIG_PATH" "$CONFIG_PATH.bak.$(date +%s)"
    fi

    if [[ -n "$TARGET" ]]; then
        # Target: copy config + .env
        scp -q "$CONFIG_TEMPLATE" "$TARGET_HOST:$TARGET_ZEROCLAW_DIR/config.toml"
        ssh "$TARGET_HOST" "chmod 600 $TARGET_ZEROCLAW_DIR/config.toml"
        if [[ -n "$ENV_FILE" ]]; then
            scp -q "$ENV_FILE" "$TARGET_HOST:$TARGET_DEPLOY_DIR/.env"
            ssh "$TARGET_HOST" "chmod 600 $TARGET_DEPLOY_DIR/.env"
            echo "  .env → $TARGET_HOST:$TARGET_DEPLOY_DIR/"
        fi
    else
        # Local: symlink config.toml → config.default.toml
        ln -sf "$CONFIG_TEMPLATE" "$CONFIG_PATH"
        # Local: symlink .env → repo .env
        if [[ -n "$ENV_FILE" ]]; then
            ln -sf "$ENV_FILE" "$ZEROCLAW_DIR/.env"
            echo "  .env → $(basename "$ENV_FILE")"
        fi
    fi
    echo "  config.toml → $(basename "$CONFIG_TEMPLATE")"

    # Profile files (SOUL.md, AGENTS.md)
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
        echo "  USER.md (exists, kept)"
    fi

    echo "  OK"
    echo ""
}

# ══════════════════════════════════════════════
# SKILLS
# ══════════════════════════════════════════════
install_skills() {
    echo "[Skills]"
    ensure_dir "$WS/skills"

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
}

# ══════════════════════════════════════════════
# DEPENDENCIES (gog, etc.) — full onboard only
# ══════════════════════════════════════════════
install_deps() {
    echo "[Dependencies]"

    # ── gog (Google Calendar) ──
    if [[ -d "$PROFILE_DIR/skills/calendar" ]]; then
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
        fi
    fi
    echo ""
}

# ══════════════════════════════════════════════
# CONNECTION TEST — full onboard only
# ══════════════════════════════════════════════
test_connection() {
    echo "[Test]"
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
    echo ""
}

# ══════════════════════════════════════════════
# EXECUTE by scope
# ══════════════════════════════════════════════
case "$SCOPE" in
    full)
        install_binary
        install_config
        install_skills
        install_deps
        test_connection
        ;;
    binary)
        install_binary
        restart_daemon
        ;;
    skills)
        install_skills
        restart_daemon
        ;;
    config)
        install_config
        restart_daemon
        ;;
esac

echo "════════════════════════════════"
echo "Lisa onboard complete!"
echo "  Scope:   $SCOPE"
echo "  Profile: $PROFILE"
if [[ -n "$TARGET" ]]; then
    echo "  Target:  $TARGET_HOST"
    echo "  Run:     ssh $TARGET_HOST 'cd $TARGET_DEPLOY_DIR && source .env && ./zeroclaw daemon'"
else
    echo "  Run:     source .env && zeroclaw daemon"
fi
echo "════════════════════════════════"
