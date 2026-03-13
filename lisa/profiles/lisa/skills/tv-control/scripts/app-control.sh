#!/bin/sh
# Usage: app-control.sh list <keyword>|launch_id|launch_category|foreground [target]
action="$1"
target="$2"
case "$action" in
  list)
    if [ -z "$2" ]; then
      echo "Error: at least one keyword required for 'list'" >&2
      exit 1
    fi
    if [ ! -f apps.json ]; then
      luna-send -n 1 luna://com.webos.applicationManager/listApps '{}' \
        | python3 -c "import sys,json;apps=json.load(sys.stdin)['apps'];print(json.dumps([{k:a[k] for k in ('title','id','appCategory') if k in a} for a in apps]))" \
        > apps.json
    fi
    shift
    python3 -c "
import sys, json
keywords = [k.lower() for k in sys.argv[1:]]
with open('apps.json') as f:
    apps = json.load(f)
matched = [a for a in apps if any(kw in str(a.get(f, '')).lower() for kw in keywords for f in ('title', 'id', 'appCategory'))]
print(json.dumps(matched))
" "$@"
    ;;
  launch_id)
    if [ -z "$target" ]; then
      echo "Error: app_id required for 'launch_id'" >&2
      exit 1
    fi
    luna-send -n 1 luna://com.webos.applicationManager/launch '{"id":"'"$target"'"}'
    ;;
  launch_category)
    if [ -z "$target" ]; then
      echo "Error: category required for 'launch_category'" >&2
      exit 1
    fi
    luna-send -n 1 luna://com.webos.applicationManager/launchDefaultApp '{"category":"'"$target"'"}'
    ;;
  foreground)
    luna-send -n 1 luna://com.webos.applicationManager/getForegroundAppInfo '{}'
    ;;
  *)
    echo "Usage: $0 list|launch_id|launch_category|foreground [target]" >&2
    exit 1
    ;;
esac
