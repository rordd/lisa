#!/usr/bin/env python3
"""Search apps.json by keywords with exact-match priority.

Usage: python3 app-search.py <keyword> [keyword...]
"""
import sys
import json

keywords = [k.lower() for k in sys.argv[1:]]
with open('apps.json') as f:
    apps = json.load(f)

matched = [a for a in apps if any(
    kw in str(a.get(field, '')).lower()
    for kw in keywords
    for field in ('title', 'id', 'appCategory')
)]


# Sort exact matches first
def exact_score(a):
    for kw in keywords:
        cats = a.get('appCategory', [])
        cat_list = cats if isinstance(cats, list) else [str(cats)]
        if (kw == a.get('title', '').lower()
                or kw == a.get('id', '').lower()
                or kw in [c.lower() for c in cat_list]):
            return 0
    return 1


matched.sort(key=exact_score)
print(json.dumps(matched))
