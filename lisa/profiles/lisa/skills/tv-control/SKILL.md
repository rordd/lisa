---
name: tv-control
description: Control this webOS TV via the exec tool. Supported actions: change channels, adjust volume, and manage apps.
---

# TV Control

## Mandatory Rules

1. Never guess command execution results or reuse previous results.
2. Every command must be actually executed by calling the exec tool each time, even if it is the same command.
3. Even if you have seen the result of the same command in a previous conversation, you must re-execute it to obtain the actual value at the current point in time.
4. "Generating" command results is prohibited. You must only use results obtained through tool calls.

## Commands

### Channel control

Change the TV channel. Automatically launches live TV if not already in the foreground.

- `action` (required): `up` | `down` | `goto`
- `channel_number` (required for `goto`): Channel number to switch to (e.g. 9, 12, 190).

```
sh skills/tv-control/scripts/channel-control.sh {action} {channel_number}
```

### Volume control

Control the TV volume.

- `action` (required): `up` | `down` | `set`
- `level` (required for `set`): Volume level (integer, 0-100).

```
sh skills/tv-control/scripts/volume-control.sh {action} {level}
```

### App control

Manage webOS apps.

- `action` (required): `list` | `launch_id` | `launch_category` | `foreground`
- `target`:
  - `list`: one or more space-separated keywords (OR match) to filter apps by title, id, or appCategory (e.g. `유투브 youtube`)
  - `launch_id`: app ID — use only when the app has no `appCategory` (e.g. `com.webos.app.netflix`)
  - `launch_category`: `appCategory` value from the app info — use when the app has an `appCategory` (e.g. `home`)
  - `foreground`: leave empty

> **Launch rule**: if `appCategory` is present in the app info, always use `launch_category`. Use `launch_id` only when there is no `appCategory`.

```
sh skills/tv-control/scripts/app-control.sh {action} {target}
```

## Safety Guidelines

- If a command fails, show the error output and suggest alternatives.
