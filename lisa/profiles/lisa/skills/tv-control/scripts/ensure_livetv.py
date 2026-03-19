#!/usr/bin/env python3
"""
Ensure live TV (com.webos.app.livetv) is in the foreground.
Launches it if not, then waits until it becomes active (up to 10 seconds).
Exits with code 0 on success, 1 on timeout.
"""
import sys
import json
import subprocess
import signal
import time

LIVETV = "com.webos.app.livetv"


def get_foreground_app():
    r = subprocess.run(
        ["luna-send", "-n", "1",
         "luna://com.webos.applicationManager/getForegroundAppInfo", "{}"],
        capture_output=True, text=True)
    try:
        return json.loads(r.stdout).get("appId", "")
    except Exception:
        return ""


def launch_livetv():
    subprocess.run(
        ["luna-send", "-n", "1",
         "luna://com.webos.applicationManager/launch",
         '{"id":"' + LIVETV + '"}'],
        capture_output=True)


def wait_for_livetv(timeout=10):
    proc = subprocess.Popen(
        ["luna-send", "-i",
         "luna://com.webos.applicationManager/getForegroundAppInfo",
         '{"subscribe":true}'],
        stdout=subprocess.PIPE, text=True)

    def _timeout(signum, frame):
        proc.kill()
        sys.exit(1)

    signal.signal(signal.SIGALRM, _timeout)
    signal.alarm(timeout)

    for line in proc.stdout:
        try:
            if json.loads(line.strip()).get("appId") == LIVETV:
                break
        except Exception:
            continue

    signal.alarm(0)
    proc.kill()
    proc.wait()


if get_foreground_app() == LIVETV:
    sys.exit(0)

launch_livetv()
wait_for_livetv()
time.sleep(1)
