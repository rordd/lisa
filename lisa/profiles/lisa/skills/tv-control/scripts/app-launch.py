#!/usr/bin/env python3
"""Find an app by keyword and launch it with the best method.

Uses app-search.py to find matching apps (with exact-match priority),
then launches via launchDefaultApp (if appCategory exists) or launch (by id).

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
    print(result.stderr.strip() or f'Error: search failed for "{target}"', file=sys.stderr)
    sys.exit(1)

matched = json.loads(result.stdout)
if not matched:
    print(f'Error: no app found matching "{target}". Try a different keyword.', file=sys.stderr)
    sys.exit(1)

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
    method = f'launch_category (appCategory={app_category})'
else:
    launch_result = subprocess.run(
        ['luna-send', '-n', '1',
         'luna://com.webos.applicationManager/launch',
         json.dumps({'id': app_id})],
        capture_output=True, text=True
    )
    method = f'launch_id (id={app_id})'

output = launch_result.stdout.strip() or launch_result.stderr.strip()
print(f'Launched {app.get("title", app_id)} via {method}')
if output:
    print(output)
sys.exit(launch_result.returncode)
