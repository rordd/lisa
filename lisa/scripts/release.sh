#!/usr/bin/env bash
set -euo pipefail

# ─────────────────────────────────────────────
# Lisa release — build + bundle + GitHub Release
# ─────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LISA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_DIR="$(cd "$LISA_DIR/.." && pwd)"

VERSION=""
SKIP_BUILD=false
DRY_RUN=false
TARGETS=("host")

usage() {
    cat << EOF
Usage: release.sh --version <tag> [options]

Options:
  --version <tag>       Release version tag (required, e.g. v0.2.0)
  --target <list>       Comma-separated: host, arm64, all (default: host)
  --skip-build          Skip build, use existing binaries
  --dry-run             Preview without uploading

Examples:
  release.sh --version v0.2.0                      # host only
  release.sh --version v0.2.0 --target all         # host + arm64
  release.sh --version v0.2.0 --dry-run            # preview
EOF
    exit 1
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --version)    VERSION="${2:-}"; shift 2 || usage ;;
        --target)     IFS=',' read -ra TARGETS <<< "${2:-}"; shift 2 || usage ;;
        --skip-build) SKIP_BUILD=true; shift ;;
        --dry-run)    DRY_RUN=true; shift ;;
        -h|--help)    usage ;;
        *)            echo "Unknown option: $1"; usage ;;
    esac
done

[[ -z "$VERSION" ]] && { echo "ERROR: --version required"; usage; }
[[ " ${TARGETS[*]} " =~ " all " ]] && TARGETS=("host" "arm64")

echo ""
echo "Lisa Release"
echo "============"
echo "  Version: $VERSION"
echo "  Targets: ${TARGETS[*]}"
echo ""

cd "$REPO_DIR"
STAGE_DIR="/tmp/lisa-release-$$"
mkdir -p "$STAGE_DIR"
BUNDLES=()

# ── Helper: platform name ──
host_platform() {
    case "$(uname -s)-$(uname -m)" in
        Darwin-arm64)  echo "aarch64-apple-darwin" ;;
        Darwin-x86_64) echo "x86_64-apple-darwin" ;;
        Linux-x86_64)  echo "x86_64-unknown-linux-gnu" ;;
        Linux-aarch64) echo "aarch64-unknown-linux-gnu" ;;
        *)             echo "unknown" ;;
    esac
}

# ── Helper: create bundle ──
create_bundle() {
    local platform="$1" binary="$2"
    local bundle_name="lisa-${VERSION}-${platform}"
    local bundle_dir="$STAGE_DIR/$bundle_name"

    mkdir -p "$bundle_dir/profiles"

    # Binary
    cp "$binary" "$bundle_dir/zeroclaw"
    chmod +x "$bundle_dir/zeroclaw"

    # Skill dependency binaries (gog, etc.)
    mkdir -p "$bundle_dir/bin"
    # gog — try local build first, then GitHub release
    local gog_bin=""
    case "$platform" in
        *linux*)
            gog_bin="$REPO_DIR/target/aarch64-unknown-linux-gnu/release/gog"
            [[ ! -f "$gog_bin" ]] && gog_bin=""
            ;;
        *apple*|*darwin*)
            gog_bin="$(command -v gog 2>/dev/null || true)"
            ;;
    esac
    if [[ -n "$gog_bin" && -f "$gog_bin" ]]; then
        cp "$gog_bin" "$bundle_dir/bin/gog"
        chmod +x "$bundle_dir/bin/gog"
        echo "  + gog binary included"
    else
        echo "  ! gog binary not found (skip)"
    fi

    # Scripts
    cp "$LISA_DIR/scripts/setup.sh" "$bundle_dir/"
    chmod +x "$bundle_dir/setup.sh"

    # Config
    mkdir -p "$bundle_dir/config"
    cp "$LISA_DIR/config/config.default.toml" "$bundle_dir/config/"

    # .env example
    cp "$LISA_DIR/profiles/.env.example" "$bundle_dir/"

    # Profiles
    cp -r "$LISA_DIR/profiles/lisa" "$bundle_dir/profiles/"

    # Tar
    local tarball="$STAGE_DIR/${bundle_name}.tar.gz"
    tar -czf "$tarball" -C "$STAGE_DIR" "$bundle_name"
    BUNDLES+=("$tarball")

    local size
    size=$(ls -lh "$tarball" | awk '{print $5}')
    echo "  Bundle: ${bundle_name}.tar.gz ($size)"
}

# ── Step 1: Build + Bundle ──
echo "[1/2] Building bundles..."

for target in "${TARGETS[@]}"; do
    case "$target" in
        host)
            PLATFORM=$(host_platform)
            if [[ "$SKIP_BUILD" == false ]]; then
                echo "  Building host..."
                cargo build --release
            fi
            BIN="$REPO_DIR/target/release/zeroclaw"
            [[ -f "$BIN" ]] || { echo "  ERROR: Host binary not found"; exit 1; }
            create_bundle "$PLATFORM" "$BIN"
            ;;
        arm64)
            PLATFORM="aarch64-unknown-linux-gnu"
            if [[ "$SKIP_BUILD" == false ]]; then
                echo "  Cross-compiling ARM64..."
                cross build --release --target aarch64-unknown-linux-gnu
            fi
            BIN="$REPO_DIR/target/aarch64-unknown-linux-gnu/release/zeroclaw"
            [[ -f "$BIN" ]] || { echo "  ERROR: ARM64 binary not found"; exit 1; }
            create_bundle "$PLATFORM" "$BIN"
            ;;
        *)
            echo "  WARNING: Unknown target '$target'"
            ;;
    esac
done
echo ""

# ── Step 2: Upload ──
echo "[2/2] Creating GitHub Release..."

NOTES="## Lisa $VERSION

### Bundles
$(for b in "${BUNDLES[@]}"; do echo "- $(basename "$b")"; done)

### Quick Start
\`\`\`bash
tar xzf lisa-${VERSION}-<platform>.tar.gz
cd lisa-${VERSION}-<platform>
cp .env.example .env && vi .env   # fill in API key + bot token
./setup.sh --binary ./zeroclaw
\`\`\`
"

if [[ "$DRY_RUN" == true ]]; then
    echo "  [dry-run] gh release create $VERSION"
    for b in "${BUNDLES[@]}"; do echo "  [dry-run]   $(basename "$b")"; done
else
    gh release create "$VERSION" \
        --repo rordd/lisa \
        --title "Lisa $VERSION" \
        --notes "$NOTES" \
        "${BUNDLES[@]}"
    echo "  https://github.com/rordd/lisa/releases/tag/$VERSION"
fi

# Cleanup
rm -rf "$STAGE_DIR"

echo ""
echo "Lisa release complete! ($VERSION, ${#BUNDLES[@]} bundle(s))"
