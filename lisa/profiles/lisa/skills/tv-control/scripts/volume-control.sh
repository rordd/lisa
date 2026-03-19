#!/bin/sh
# Usage: volume-control.sh up|down|set [level]
# Outputs only luna-send JSON result to stdout.
action="$1"
case "$action" in
  up)
    luna-send -n 1 luna://com.webos.service.audio/master/volumeUp '{}' || echo '{"returnValue":false}'
    ;;
  down)
    luna-send -n 1 luna://com.webos.service.audio/master/volumeDown '{}' || echo '{"returnValue":false}'
    ;;
  set)
    level="$2"
    if [ -z "$level" ]; then
      echo '{"returnValue":false,"errorText":"level required for set"}'
      exit 0
    fi
    luna-send -n 1 luna://com.webos.service.audio/master/setVolume '{"volume":'"$level"'}' || echo '{"returnValue":false}'
    ;;
  *)
    echo '{"returnValue":false,"errorText":"unknown action: '"$action"'. Use up|down|set"}'
    exit 0
    ;;
esac
