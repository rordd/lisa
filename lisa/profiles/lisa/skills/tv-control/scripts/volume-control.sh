#!/bin/sh
# Usage: volume-control.sh up|down|set [level]
action="$1"
case "$action" in
  up)
    luna-send -n 1 luna://com.webos.service.audio/master/volumeUp '{}'
    ;;
  down)
    luna-send -n 1 luna://com.webos.service.audio/master/volumeDown '{}'
    ;;
  set)
    level="$2"
    if [ -z "$level" ]; then
      echo "Error: level required for 'set'" >&2
      exit 1
    fi
    luna-send -n 1 luna://com.webos.service.audio/master/setVolume '{"volume":'"$level"'}'
    ;;
  *)
    echo "Usage: $0 up|down|set [level]" >&2
    exit 1
    ;;
esac
