#!/bin/sh
# Usage: app-control.sh launch|foreground [target]
# Outputs luna-send JSON result to stdout.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
action="$1"
target="$2"

# Ensure apps.json cache exists
ensure_apps_cache() {
    if [ ! -f apps.json ] || [ ! -s apps.json ]; then
        rm -f apps.json
        luna-send -n 1 luna://com.webos.applicationManager/listApps '{}' \
            | python3 "$SCRIPT_DIR/app-parse-list.py" \
            > apps.json.tmp 2>/dev/null
        if [ -s apps.json.tmp ]; then
            mv apps.json.tmp apps.json
        else
            rm -f apps.json.tmp
        fi
    fi
}

case "$action" in
  list)
    if [ -z "$2" ]; then
        echo '{"returnValue":false,"errorText":"at least one keyword required for list"}'
        exit 0
    fi
    ensure_apps_cache
    shift
    python3 "$SCRIPT_DIR/app-search.py" "$@"
    ;;
  launch)
    if [ -z "$target" ]; then
        echo '{"returnValue":false,"errorText":"keyword or app name required for launch"}'
        exit 0
    fi
    ensure_apps_cache
    python3 "$SCRIPT_DIR/app-launch.py" "$target"
    ;;
  foreground)
    luna-send -n 1 luna://com.webos.applicationManager/getForegroundAppInfo '{}' || echo '{"returnValue":false}'
    ;;
  *)
    echo '{"returnValue":false,"errorText":"unknown action: '"$action"'. Use launch|foreground"}'
    exit 0
    ;;
esac
