---
name: calendar
description: "Google Calendar event lookup/creation. Uses gog CLI. Used when the user asks about schedules, meetings, or events."
version: "1.0.0"
always: true
---

# Calendar Skill

Manage Google Calendar via the gog CLI.

## Prerequisites

gog is installed automatically by onboard.sh.
For manual setup, see docs/gogcli-oauth-setup-guide.md.

## When to Use

- "What's on my schedule today?"
- "Do I have any meetings tomorrow?"
- "This week's schedule"
- "Schedule a meeting"

## Calendar List

Refer to USER.md for calendar IDs. Primary calendars:
- primary — personal default
- Additional calendars defined in USER.md

## Commands

### View Events
```bash
# Today's events
gog calendar events <calendarId> --from $(date +%Y-%m-%dT00:00:00) --to $(date +%Y-%m-%dT23:59:59) --json

# Tomorrow's events
gog calendar events <calendarId> --from $(date -d "+1 day" +%Y-%m-%dT00:00:00) --to $(date -d "+1 day" +%Y-%m-%dT23:59:59) --json

# This week's events
gog calendar events <calendarId> --from $(date +%Y-%m-%dT00:00:00) --to $(date -d "+7 days" +%Y-%m-%dT23:59:59) --json

# On macOS, use date -v+1d instead
# gog calendar events <calendarId> --from $(date -v+1d +%Y-%m-%dT00:00:00) --to $(date -v+1d +%Y-%m-%dT23:59:59) --json
```

### Create Event
```bash
gog calendar create <calendarId> --summary "Meeting title" --from 2026-03-03T14:00:00 --to 2026-03-03T15:00:00
```

### Update Event
```bash
gog calendar update <calendarId> <eventId> --summary "New title"
```

### Delete Event
```bash
# Delete by event ID (use calendar_events --json to find the event ID first)
gog calendar delete <calendarId> <eventId> --force
```

### Colors
```bash
gog calendar colors
# Use --event-color <1-11> to set a color
```

## Environment Variables

```bash
export GOG_ACCOUNT=<email>              # Default account
export GOG_KEYRING_PASSWORD=<password>  # Keyring password (keyring=file mode)
export GOG_KEYRING_BACKEND=file         # File-based keyring (no DBUS required)
```

## Multiple Calendars

When briefing, iterate over all calendars defined in USER.md:
```bash
for cal in "primary" "cal_id_1" "cal_id_2"; do
  echo "=== $cal ==="
  gog calendar events "$cal" --from ... --to ... --json
done
```

## Rules

- Always confirm with the user before creating/updating/deleting events
- Use `--json` flag for parseable output
- Without `GOG_ACCOUNT`, `--account` is required each time
