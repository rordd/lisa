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
SCOPE="full"     # full | binary | skills | config | clear
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
  --clear               Remove all installed files (binary + config + workspace + daemon)
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
  onboard.sh --clear                      # remove all installed files
  onboard.sh --clear --target 10.0.0.1    # remove all from target
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
        --clear)     SCOPE="clear"; shift ;;
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
for candidate in "$REPO_DIR/.env" "$BASE_DIR/.env" "$BASE_DIR/profiles/.env"; do
    [[ -f "$candidate" ]] && { ENV_FILE="$candidate"; break; }
done

if [[ -n "$ENV_FILE" ]]; then
    # shellcheck disable=SC1090
    set -a  # auto-export all sourced variables
    source "$ENV_FILE"
    set +a
fi

# API key check (skip for skills-only)
if [[ "$SCOPE" != "skills" && "$SCOPE" != "clear" && -z "${ZEROCLAW_API_KEY:-}" && -z "${AZURE_OPENAI_API_KEY:-}" ]]; then
    echo "ERROR: No API key found (ZEROCLAW_API_KEY or AZURE_OPENAI_API_KEY)"
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
    local dest="${!#}"  # last argument
    local srcs=("${@:1:$#-1}")  # all but last
    if [[ -n "$TARGET" ]]; then
        scp -qr "${srcs[@]}" "$TARGET_HOST:$dest"
    else
        cp -r "${srcs[@]}" "$dest"
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

    # Find binary: --build output > bundle > local build > PATH
    if [[ -n "$BINARY_PATH" ]]; then
        : # already set by --build
    elif [[ -f "$BASE_DIR/zeroclaw" ]]; then
        BINARY_PATH="$BASE_DIR/zeroclaw"  # bundle
    elif [[ -f "$REPO_DIR/target/release/zeroclaw" ]]; then
        BINARY_PATH="$REPO_DIR/target/release/zeroclaw"  # local build
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
# HOSTS — Azure private endpoint
# ══════════════════════════════════════════════
setup_hosts() {
    local private_ip="${AZURE_PRIVATE_ENDPOINT:-}"
    [[ -z "$private_ip" ]] && return 0

    # Extract hostname from ZEROCLAW_PROVIDER=custom:https://host.../path
    local provider="${ZEROCLAW_PROVIDER:-}"
    local hostname=""
    hostname=$(echo "$provider" | sed -n 's|.*custom:https\?://\([^/:]*\).*|\1|p')
    [[ -z "$hostname" ]] && return 0

    local hosts_entry="$private_ip $hostname"
    echo "[Hosts]"

    setup_hosts_on() {
        local run_cmd="$1"  # "" for local, "ssh $TARGET_HOST" for target
        local home_dir="$2"

        # Already configured?
        if $run_cmd grep -q "$hostname" /etc/hosts 2>/dev/null; then
            echo "  $hostname already in /etc/hosts ✓"
            return 0
        fi

        # 1. Writable /etc/hosts → direct append (root user)
        if $run_cmd test -w /etc/hosts 2>/dev/null; then
            $run_cmd sh -c "echo '$hosts_entry' >> /etc/hosts"
            echo "  Added $hosts_entry to /etc/hosts"
            return 0
        fi

        # 2. Need elevated privileges
        if [[ -z "$run_cmd" ]]; then
            # Local: use sudo (prompts for password if needed)
            echo "  sudo required to modify /etc/hosts"
            if sudo sh -c "echo '$hosts_entry' >> /etc/hosts" 2>/dev/null; then
                echo "  Added $hosts_entry to /etc/hosts (sudo)"
                return 0
            fi
            # /etc/hosts append failed (read-only filesystem) → bind mount
            local hosts_copy="$home_dir/.hosts"
            cp /etc/hosts "$hosts_copy"
            sh -c "echo '$hosts_entry' >> $hosts_copy"
            sudo mountpoint -q /etc/hosts 2>/dev/null && sudo umount /etc/hosts 2>/dev/null || true
            sudo mount --bind "$hosts_copy" /etc/hosts
            echo "  Added $hosts_entry via bind mount ($hosts_copy → /etc/hosts)"
        else
            # Target (SSH, typically root) → bind mount for read-only
            local hosts_copy="$home_dir/.hosts"
            $run_cmd cp /etc/hosts "$hosts_copy"
            $run_cmd sh -c "echo '$hosts_entry' >> $hosts_copy"
            $run_cmd mountpoint -q /etc/hosts 2>/dev/null && $run_cmd umount /etc/hosts 2>/dev/null || true
            $run_cmd mount --bind "$hosts_copy" /etc/hosts
            echo "  Added $hosts_entry via bind mount ($hosts_copy → /etc/hosts)"
        fi
    }

    if [[ -n "$TARGET" ]]; then
        setup_hosts_on "ssh $TARGET_HOST" "/home/$TARGET_USER"
    else
        setup_hosts_on "" "$HOME"
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
        # Local: copy config + .env to ~/.zeroclaw/ (independent of repo)
        cp "$CONFIG_TEMPLATE" "$CONFIG_PATH"
        chmod 600 "$CONFIG_PATH"
        if [[ -n "$ENV_FILE" ]]; then
            cp "$ENV_FILE" "$ZEROCLAW_DIR/.env"
            chmod 600 "$ZEROCLAW_DIR/.env"
            echo "  .env → $ZEROCLAW_DIR/.env"
        fi
    fi
    echo "  config.toml → $CONFIG_PATH"

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

        # Install gog if missing — use bundled bin/gog or download from lisa release
        if [[ -n "$TARGET" ]]; then
            local gog_installed
            gog_installed=$(ssh "$TARGET_HOST" "test -x $TARGET_DEPLOY_DIR/gog && echo yes || echo no")
            if [[ "$gog_installed" == "no" ]]; then
                echo "  Installing gog on target..."
                local gog_bin=""
                if [[ -f "$BASE_DIR/bin/gog" ]]; then
                    gog_bin="$BASE_DIR/bin/gog"
                else
                    # Download from lisa release (contains statically linked gog)
                    local target_arch
                    target_arch=$(ssh "$TARGET_HOST" "uname -m")
                    local lisa_platform="x86_64-unknown-linux-gnu"
                    [[ "$target_arch" == "aarch64" || "$target_arch" == "arm64" ]] && lisa_platform="aarch64-unknown-linux-gnu"
                    local lisa_tag
                    lisa_tag=$(gh release view --repo rordd/lisa --json tagName -q '.tagName' 2>/dev/null || echo "v0.2.0-lisa")
                    local gog_tmp="/tmp/gog-install-$$"
                    mkdir -p "$gog_tmp"
                    local lisa_tar="lisa-${lisa_tag}-${lisa_platform}.tar.gz"
                    gh release download "$lisa_tag" --repo rordd/lisa --pattern "$lisa_tar" --dir "$gog_tmp" 2>/dev/null \
                        || curl -sfL "https://github.com/rordd/lisa/releases/download/${lisa_tag}/${lisa_tar}" -o "$gog_tmp/$lisa_tar"
                    tar xzf "$gog_tmp/$lisa_tar" -C "$gog_tmp" --wildcards "*/bin/gog" --strip-components=2 2>/dev/null \
                        || tar xzf "$gog_tmp/$lisa_tar" -C "$gog_tmp" "$(tar tzf "$gog_tmp/$lisa_tar" | grep 'bin/gog$')" --strip-components=2
                    if [[ -f "$gog_tmp/gog" ]]; then
                        gog_bin="$gog_tmp/gog"
                    fi
                fi
                if [[ -n "$gog_bin" ]]; then
                    scp -q "$gog_bin" "$TARGET_HOST:$TARGET_DEPLOY_DIR/gog"
                    ssh "$TARGET_HOST" "chmod +x $TARGET_DEPLOY_DIR/gog"
                    echo "  gog installed → $TARGET_HOST:$TARGET_DEPLOY_DIR/gog"
                else
                    echo "  WARNING: gog binary not found in bundle or lisa release"
                fi
                [[ -d "/tmp/gog-install-$$" ]] && rm -rf "/tmp/gog-install-$$"
            else
                echo "  gog already installed on target ✓"
            fi
        elif ! command -v gog &>/dev/null; then
            echo "  Installing gog locally..."
            local gog_bin=""
            if [[ -f "$BASE_DIR/bin/gog" ]]; then
                gog_bin="$BASE_DIR/bin/gog"
            else
                # Download from lisa release
                local host_arch
                host_arch=$(uname -m)
                local host_os
                host_os=$(uname -s)
                local lisa_platform=""
                case "$host_os" in
                    Darwin) lisa_platform="aarch64-apple-darwin" ;;
                    Linux)
                        if [[ "$host_arch" == "aarch64" || "$host_arch" == "arm64" ]]; then
                            lisa_platform="aarch64-unknown-linux-gnu"
                        else
                            lisa_platform="x86_64-unknown-linux-gnu"
                        fi ;;
                esac
                if [[ -n "$lisa_platform" ]]; then
                    local lisa_tag
                    lisa_tag=$(gh release view --repo rordd/lisa --json tagName -q '.tagName' 2>/dev/null || echo "v0.2.0-lisa")
                    local gog_tmp="/tmp/gog-install-$$"
                    mkdir -p "$gog_tmp"
                    local lisa_tar="lisa-${lisa_tag}-${lisa_platform}.tar.gz"
                    gh release download "$lisa_tag" --repo rordd/lisa --pattern "$lisa_tar" --dir "$gog_tmp" 2>/dev/null \
                        || curl -sfL "https://github.com/rordd/lisa/releases/download/${lisa_tag}/${lisa_tar}" -o "$gog_tmp/$lisa_tar"
                    tar xzf "$gog_tmp/$lisa_tar" -C "$gog_tmp" --wildcards "*/bin/gog" --strip-components=2 2>/dev/null \
                        || tar xzf "$gog_tmp/$lisa_tar" -C "$gog_tmp" "$(tar tzf "$gog_tmp/$lisa_tar" | grep 'bin/gog$')" --strip-components=2
                    [[ -f "$gog_tmp/gog" ]] && gog_bin="$gog_tmp/gog"
                fi
            fi
            if [[ -n "$gog_bin" ]]; then
                mkdir -p "$HOME/.local/bin"
                cp "$gog_bin" "$HOME/.local/bin/gog"
                chmod +x "$HOME/.local/bin/gog"
                echo "  gog installed → ~/.local/bin/gog"
                if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
                    echo "  NOTE: Add ~/.local/bin to PATH if gog is not found"
                fi
            else
                echo "  WARNING: Could not install gog — download from lisa release manually"
            fi
            [[ -d "/tmp/gog-install-$$" ]] && rm -rf "/tmp/gog-install-$$"
        else
            echo "  gog already installed ✓"
        fi

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
# TESTS — full onboard only
# ══════════════════════════════════════════════
run_tests() {
    echo "[Test]"
    local pass=0 fail=0 skip=0
    local agent_ok=false

    # Helper: run command locally or on target (stdout only; stderr goes to terminal)
    run_cmd() {
        if [[ -n "$TARGET" ]]; then
            ssh "$TARGET_HOST" "cd $TARGET_DEPLOY_DIR && [ -f .env ] && . .env && export ZEROCLAW_CONFIG_DIR=$TARGET_ZEROCLAW_DIR && $1"
        else
            eval "$1"
        fi
    }

    # Helper: check if response indicates skill failure (not LLM error, but skill not working)
    has_skill_failure() {
        echo "$1" | grep -qiE '실패|접속.*없|연결.*없|연결.*않|연결.*안|인증.*필요|확인.*없|가져.*없|불러.*없|수가 없|할 수 없|failed|not connected|cannot access|unable to|couldn.t|authentication required'
    }

    # ── 1. Agent (LLM connection) ──
    echo "  [agent] Sending greeting..."
    local agent_out
    agent_out=$(run_cmd "zeroclaw agent -m '안녕~'") && EXIT=0 || EXIT=$?
    if [[ $EXIT -eq 0 ]]; then
        echo "  [agent] OK — $(echo "$agent_out" | tail -1 | head -c 80)"
        pass=$((pass + 1))
        agent_ok=true
    else
        echo "  [agent] FAIL (exit=$EXIT)"
        echo "$agent_out" | tail -3 | sed 's/^/    /'
        fail=$((fail + 1))
    fi

    # ── 2. Weather skill ──
    if [[ -d "$WS/skills/weather" ]]; then
        if [[ "$agent_ok" == true ]]; then
            echo "  [weather] Asking zeroclaw for weather..."
            local weather_out
            weather_out=$(run_cmd "zeroclaw agent -m 'Seoul weather now, one line'") && EXIT=0 || EXIT=$?
            if [[ $EXIT -eq 0 ]] && ! has_skill_failure "$weather_out"; then
                echo "  [weather] OK — $(echo "$weather_out" | tail -1 | head -c 80)"
                pass=$((pass + 1))
            else
                echo "  [weather] FAIL"
                echo "$weather_out" | tail -2 | sed 's/^/    /'
                fail=$((fail + 1))
            fi
        else
            echo "  [weather] SKIP — agent not connected"
            skip=$((skip + 1))
        fi
    fi

    # ── 3. Calendar skill ──
    if [[ -d "$WS/skills/calendar" ]]; then
        if ! command -v gog &>/dev/null; then
            echo "  [calendar] FAIL — gog not installed"
            echo "    Install: brew install steipete/tap/gogcli"
            fail=$((fail + 1))
        elif [[ "$agent_ok" == true ]]; then
            echo "  [calendar] Asking zeroclaw for schedule..."
            local cal_out
            cal_out=$(run_cmd "zeroclaw agent -m 'list today schedule, one line'") && EXIT=0 || EXIT=$?
            if [[ $EXIT -eq 0 ]] && ! has_skill_failure "$cal_out"; then
                echo "  [calendar] OK — $(echo "$cal_out" | tail -1 | head -c 80)"
                pass=$((pass + 1))
            else
                echo "  [calendar] FAIL"
                echo "$cal_out" | tail -2 | sed 's/^/    /'
                fail=$((fail + 1))
            fi
        else
            echo "  [calendar] SKIP — agent not connected"
            skip=$((skip + 1))
        fi
    fi

    # ── 4. TV Control skill ──
    if [[ -d "$WS/skills/tv-control" ]]; then
        if [[ "$agent_ok" == true ]]; then
            echo "  [tv-control] Asking zeroclaw for foreground app..."
            local tv_out
            tv_out=$(run_cmd "zeroclaw agent -m 'what app is running on TV now, one line'") && EXIT=0 || EXIT=$?
            if [[ $EXIT -eq 0 ]] && ! has_skill_failure "$tv_out"; then
                echo "  [tv-control] OK — $(echo "$tv_out" | tail -1 | head -c 80)"
                pass=$((pass + 1))
            else
                echo "  [tv-control] SKIP — luna-send unavailable (not on webOS)"
                skip=$((skip + 1))
            fi
        else
            echo "  [tv-control] SKIP — agent not connected"
            skip=$((skip + 1))
        fi
    fi

    echo ""
    echo "  Result: $pass passed, $fail failed, $skip skipped"
    if [[ $fail -gt 0 ]]; then
        echo "  WARNING: Some tests failed. Check the output above."
    fi
    echo ""
}

# ══════════════════════════════════════════════
# CLEAR — remove all installed files
# ══════════════════════════════════════════════
clear_all() {
    echo ""
    echo "Lisa Clear"
    echo "=========="

    if [[ -n "$TARGET" ]]; then
        echo "  Target: $TARGET_HOST"
        echo ""
        echo "  Will remove:"
        echo "    $TARGET_DEPLOY_DIR/          (binary + deps + .env)"
        echo "    $TARGET_ZEROCLAW_DIR/        (config + workspace)"
        echo "    zeroclaw daemon process"
    else
        echo "  Target: localhost"
        echo ""
        echo "  Will remove:"
        local bin_path
        bin_path="$(command -v zeroclaw 2>/dev/null || echo "$HOME/.local/bin/zeroclaw")"
        echo "    $bin_path"
        if [[ -d "$BASE_DIR/bin" ]]; then
            for dep in "$BASE_DIR/bin"/*; do
                [[ -f "$dep" ]] || continue
                echo "    $HOME/.local/bin/$(basename "$dep")"
            done
        fi
        echo "    $ZEROCLAW_DIR/               (config + workspace)"
        echo "    zeroclaw daemon process"
    fi

    echo ""
    if [[ -t 0 ]]; then
        read -rp "  Proceed? [y/N] " confirm
        [[ "$confirm" =~ ^[Yy]$ ]] || { echo "  Cancelled"; exit 0; }
    else
        echo "  ERROR: Non-interactive mode — cannot confirm. Abort."
        exit 1
    fi

    echo ""

    # 1. Stop daemon
    echo "[Daemon]"
    if [[ -n "$TARGET" ]]; then
        ssh "$TARGET_HOST" "pkill -9 -f zeroclaw 2>/dev/null || true"
    else
        pkill -9 -f "zeroclaw daemon" 2>/dev/null || true
    fi
    echo "  Stopped"

    # 2. Remove binary
    echo "[Binary]"
    if [[ -n "$TARGET" ]]; then
        ssh "$TARGET_HOST" "rm -rf $TARGET_DEPLOY_DIR"
        echo "  Removed $TARGET_DEPLOY_DIR/"
    else
        local bin_path
        bin_path="$(command -v zeroclaw 2>/dev/null || true)"
        if [[ -n "$bin_path" ]]; then
            rm -f "$bin_path"
            echo "  Removed $bin_path"
        else
            echo "  Not found (skipped)"
        fi
        # Remove dep binaries
        if [[ -d "$BASE_DIR/bin" ]]; then
            for dep in "$BASE_DIR/bin"/*; do
                [[ -f "$dep" ]] || continue
                local dep_name
                dep_name="$(basename "$dep")"
                rm -f "$HOME/.local/bin/$dep_name"
                echo "  Removed ~/.local/bin/$dep_name"
            done
        fi
    fi

    # 3. Remove config + workspace
    echo "[Config + Workspace]"
    if [[ -n "$TARGET" ]]; then
        ssh "$TARGET_HOST" "rm -rf $TARGET_ZEROCLAW_DIR"
        echo "  Removed $TARGET_ZEROCLAW_DIR/"
    else
        rm -rf "$ZEROCLAW_DIR"
        echo "  Removed $ZEROCLAW_DIR/"
    fi

    # 4. Remove hosts entry
    echo "[Hosts]"
    local provider="${ZEROCLAW_PROVIDER:-}"
    local hostname=""
    hostname=$(echo "$provider" | sed -n 's|.*custom:https\?://\([^/:]*\).*|\1|p')
    if [[ -n "$hostname" ]]; then
        if [[ -n "$TARGET" ]]; then
            if ssh "$TARGET_HOST" "grep -q '$hostname' /etc/hosts 2>/dev/null"; then
                # Unmount bind if active
                ssh "$TARGET_HOST" "mountpoint -q /etc/hosts 2>/dev/null && umount /etc/hosts || true"
                ssh "$TARGET_HOST" "rm -f /home/$TARGET_USER/.hosts"
                echo "  Removed $hostname from /etc/hosts (bind unmounted)"
            else
                echo "  No hosts entry found (skipped)"
            fi
        else
            if grep -q "$hostname" /etc/hosts 2>/dev/null; then
                if mountpoint -q /etc/hosts 2>/dev/null; then
                    # Bind mounted → unmount to restore original
                    sudo umount /etc/hosts 2>/dev/null || umount /etc/hosts 2>/dev/null || true
                    rm -f "$HOME/.hosts"
                    echo "  Removed $hostname from /etc/hosts (bind unmounted)"
                else
                    # Directly written → remove the line
                    sudo sed -i "/$hostname/d" /etc/hosts 2>/dev/null \
                        || sed -i "/$hostname/d" /etc/hosts 2>/dev/null || true
                    echo "  Removed $hostname from /etc/hosts"
                fi
            else
                echo "  No hosts entry found (skipped)"
            fi
        fi
    else
        echo "  No custom provider hostname (skipped)"
    fi

    echo ""
}

# ══════════════════════════════════════════════
# EXECUTE by scope
# ══════════════════════════════════════════════
case "$SCOPE" in
    full)
        install_binary
        setup_hosts
        install_config
        install_skills
        install_deps
        run_tests
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
        setup_hosts
        install_config
        restart_daemon
        ;;
    clear)
        clear_all
        echo "════════════════════════════════"
        echo "Lisa clear complete!"
        if [[ -n "$TARGET" ]]; then
            echo "  Target:  $TARGET_HOST"
        fi
        echo "════════════════════════════════"
        exit 0
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
    echo "  Run:     source ~/.zeroclaw/.env && zeroclaw daemon"
fi
echo "════════════════════════════════"
