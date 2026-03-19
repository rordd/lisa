#!/bin/sh
# Usage: app-control.sh list|launch|foreground [target]
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
action="$1"
target="$2"

# Ensure apps.json cache exists
ensure_apps_cache() {
    if [ ! -f apps.json ]; then
        luna-send -n 1 luna://com.webos.applicationManager/listApps '{}' \
            | python3 "$SCRIPT_DIR/app-parse-list.py" \
            > apps.json
    fi
}

case "$action" in
  list)
    if [ -z "$2" ]; then
        echo "Error: at least one keyword required for 'list'" >&2
        exit 1
    fi
    ensure_apps_cache
    shift
    python3 "$SCRIPT_DIR/app-search.py" "$@"
    ;;
  launch)
    if [ -z "$target" ]; then
        echo "Error: keyword or app name required for 'launch' (e.g. 'home', 'netflix')" >&2
        exit 1
    fi
    ensure_apps_cache
    python3 "$SCRIPT_DIR/app-launch.py" "$target"
    ;;
  foreground)
    luna-send -n 1 luna://com.webos.applicationManager/getForegroundAppInfo '{}'
    ;;
  *)
    echo "Usage: $0 list|launch|foreground [target]" >&2
    exit 1
    ;;
esac
