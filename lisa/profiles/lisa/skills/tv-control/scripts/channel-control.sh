#!/bin/sh
# Usage: channel-control.sh up|down|goto [channel_number]
#
# Automatically switches to live TV (com.webos.app.livetv) if it is not
# already in the foreground before performing the channel action.

LIVETV_APP="com.webos.app.livetv"

ensure_livetv() {
    python3 "$(dirname "$0")/ensure_livetv.py"
}

action="$1"
case "$action" in
  up)
    ensure_livetv
    luna-send -n 1 -f luna://com.lge.inputgenerator/pushKeyEvent '{"eventtype":"key","keycodenum":402}'
    ;;
  down)
    ensure_livetv
    luna-send -n 1 -f luna://com.lge.inputgenerator/pushKeyEvent '{"eventtype":"key","keycodenum":403}'
    ;;
  goto)
    ch="$2"
    if [ -z "$ch" ]; then
      echo "Error: channel_number required for 'goto'" >&2
      exit 1
    fi
    ensure_livetv
    i=1
    while [ $i -le ${#ch} ]; do
      d=$(echo "$ch" | cut -c$i)
      luna-send -n 1 -f luna://com.webos.service.networkinput/sendSpecialKey '{"key":"'"$d"'","rcu":true}'
      sleep 0.3
      i=$((i+1))
    done
    sleep 0.3
    luna-send -n 1 -f luna://com.webos.service.networkinput/sendSpecialKey '{"key":"ENTER","rcu":true}'
    ;;
  *)
    echo "Usage: $0 up|down|goto [channel_number]" >&2
    exit 1
    ;;
esac
