#!/usr/bin/env python3
"""Find an app by keyword and launch it with the best method.

Uses app-search.py to find matching apps (with exact-match priority),
then launches via launchDefaultApp (if appCategory exists) or launch (by id).
Outputs only the luna-send JSON result to stdout.

Usage: python3 app-launch.py <keyword>
"""
import os
import sys
import json
import subprocess

target = sys.argv[1]
script_dir = os.path.dirname(os.path.abspath(__file__))

# Search using app-search.py
result = subprocess.run(
    ['python3', os.path.join(script_dir, 'app-search.py'), target],
    capture_output=True, text=True
)

if result.returncode != 0:
    print(json.dumps({"returnValue": False, "errorText": result.stderr.strip() or f'search failed for "{target}"'}))
    sys.exit(0)

try:
    matched = json.loads(result.stdout)
except (json.JSONDecodeError, ValueError):
    print(json.dumps({"returnValue": False, "errorText": "failed to parse app search result"}))
    sys.exit(0)
if not matched:
    print(json.dumps({"returnValue": False, "errorText": f'no app found matching "{target}"'}))
    sys.exit(0)

app = matched[0]
app_id = app.get('id', '')
app_category_raw = app.get('appCategory', '')
# appCategory may be a list (e.g. ["home"]) — extract the first value as string
if isinstance(app_category_raw, list):
    app_category = app_category_raw[0] if app_category_raw else ''
else:
    app_category = str(app_category_raw) if app_category_raw else ''

if app_category:
    launch_result = subprocess.run(
        ['luna-send', '-n', '1',
         'luna://com.webos.applicationManager/launchDefaultApp',
         json.dumps({'category': app_category})],
        capture_output=True, text=True
    )
else:
    launch_result = subprocess.run(
        ['luna-send', '-n', '1',
         'luna://com.webos.applicationManager/launch',
         json.dumps({'id': app_id})],
        capture_output=True, text=True
    )

# Output luna-send result directly
output = launch_result.stdout.strip()
if output:
    print(output)
else:
    print(json.dumps({"returnValue": False, "errorText": launch_result.stderr.strip() or "luna-send failed"}))
