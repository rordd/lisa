#!/bin/sh
# Usage: channel-control.sh up|down|goto [channel_number]
#
# Automatically switches to live TV (com.webos.app.livetv) if it is not
# already in the foreground before performing the channel action.
# Outputs only luna-send JSON result to stdout.

LIVETV_APP="com.webos.app.livetv"

ensure_livetv() {
    python3 "$(dirname "$0")/ensure_livetv.py"
}

action="$1"
case "$action" in
  up)
    ensure_livetv
    luna-send -n 1 -f luna://com.lge.inputgenerator/pushKeyEvent '{"eventtype":"key","keycodenum":402}' || echo '{"returnValue":false}'
    ;;
  down)
    ensure_livetv
    luna-send -n 1 -f luna://com.lge.inputgenerator/pushKeyEvent '{"eventtype":"key","keycodenum":403}' || echo '{"returnValue":false}'
    ;;
  goto)
    ch="$2"
    if [ -z "$ch" ]; then
      echo '{"returnValue":false,"errorText":"channel_number required for goto"}'
      exit 0
    fi
    ensure_livetv
    i=1
    while [ $i -le ${#ch} ]; do
      d=$(echo "$ch" | cut -c$i)
      luna-send -n 1 -f luna://com.webos.service.networkinput/sendSpecialKey '{"key":"'"$d"'","rcu":true}' > /dev/null 2>&1
      sleep 0.3
      i=$((i+1))
    done
    sleep 0.3
    luna-send -n 1 -f luna://com.webos.service.networkinput/sendSpecialKey '{"key":"ENTER","rcu":true}' || echo '{"returnValue":false}'
    ;;
  *)
    echo '{"returnValue":false,"errorText":"unknown action: '"$action"'. Use up|down|goto"}'
    exit 0
    ;;
esac
