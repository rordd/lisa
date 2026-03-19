#!/usr/bin/env python3
"""Parse luna-send listApps output into a compact JSON array.

Reads raw listApps JSON from stdin, extracts title/id/appCategory fields,
and writes the result to apps.json.

Usage: luna-send -n 1 luna://...listApps '{}' | python3 app-parse-list.py
"""
import sys
import json

EXCLUDE = ('example', 'test')
apps = json.load(sys.stdin)['apps']
filtered = [a for a in apps if not any(ex in a.get('id', '').lower() for ex in EXCLUDE)]
print(json.dumps([{k: a[k] for k in ('title', 'id', 'appCategory') if k in a} for a in filtered]))
