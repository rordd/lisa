#!/bin/bash
# Mock TV control script for E2E testing
# Simulates webOS luna-send responses

LOG_FILE="/tmp/mock-tv-calls.log"
echo "$(date '+%Y-%m-%d %H:%M:%S') CALLED: $0 $*" >> "$LOG_FILE"

ACTION="$1"
shift

case "$ACTION" in
    launch)
        APP_ID="${1:-unknown}"
        echo "{\"returnValue\":true,\"id\":\"$APP_ID\"}"
        ;;
    foreground)
        echo "{\"returnValue\":true,\"appId\":\"com.webos.app.livetv\",\"processId\":\"1001\",\"windowId\":\"_Web_Window_0\"}"
        ;;
    channel_up)
        echo "{\"returnValue\":true,\"action\":\"channel_up\",\"channel\":\"12\"}"
        ;;
    channel_down)
        echo "{\"returnValue\":true,\"action\":\"channel_down\",\"channel\":\"10\"}"
        ;;
    go_to_channel)
        CH="${1:-1}"
        echo "{\"returnValue\":true,\"action\":\"go_to_channel\",\"channel\":\"$CH\"}"
        ;;
    volume_up)
        echo "{\"returnValue\":true,\"action\":\"volume_up\",\"volume\":16}"
        ;;
    volume_down)
        echo "{\"returnValue\":true,\"action\":\"volume_down\",\"volume\":14}"
        ;;
    set_volume)
        VOL="${1:-15}"
        echo "{\"returnValue\":true,\"action\":\"set_volume\",\"volume\":$VOL}"
        ;;
    *)
        echo "{\"returnValue\":false,\"errorText\":\"Unknown action: $ACTION\"}"
        exit 1
        ;;
esac
