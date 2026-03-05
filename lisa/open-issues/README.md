# Open Issues

Directory for tracking unresolved issues in the Lisa project.

## Issue File Format

Filename: `{id}-{short-title}.md` (e.g., `001-temperature-hardcoded.md`)

```markdown
---
id: 001
title: Issue title
status: open
priority: high
category: bug
created: 2026-03-03
updated: 2026-03-03
---

## Description

Detailed description of the issue.

## Current Workaround

(if applicable)

## Proposed Solution

(if known)
```

### Field Descriptions

| Field | Values | Description |
|-------|--------|-------------|
| `status` | `open`, `closed` | Issue status |
| `priority` | `high`, `medium`, `low` | Priority level |
| `category` | `bug`, `feature`, `improvement`, `config` | Category |

## Management Script

```bash
# Create issue
./lisa/scripts/issues.sh new

# List (open only)
./lisa/scripts/issues.sh list

# List all (including closed)
./lisa/scripts/issues.sh list --all

# Filter
./lisa/scripts/issues.sh list --priority high
./lisa/scripts/issues.sh list --category bug

# View details
./lisa/scripts/issues.sh show 1

# Change status
./lisa/scripts/issues.sh close 1
./lisa/scripts/issues.sh reopen 1

# Delete
./lisa/scripts/issues.sh delete 1

# Summary statistics
./lisa/scripts/issues.sh summary
```
