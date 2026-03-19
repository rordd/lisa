---
name: tv-control
description: Control this webOS TV via the exec tool. Supported actions: change channels, adjust volume, and manage apps.
always: true
---

# TV Control

## Mandatory Rules

1. Never guess command execution results or reuse previous results.
2. Every command must be actually executed by calling the exec tool each time, even if it is the same command.
3. Even if you have seen the result of the same command in a previous conversation, you must re-execute it to obtain the actual value at the current point in time.
4. "Generating" command results is prohibited. You must only use results obtained through tool calls.
5. Tool results are JSON. Check `returnValue`: `true` means success, `false` means error (see `errorText`).

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

- `action` (required): `launch` | `foreground`
- `target`:
  - `launch`: app keyword to find and launch — automatically searches and resolves the correct method (e.g. `netflix`, `home`). Always translate Korean app names to English (e.g. 넷플릭스→netflix, 유튜브→youtube, 디즈니플러스→disney, 티빙→tving, 쿠팡플레이→coupang, 웨이브→wavve, 홈→home, 설정→settings, 라이브→livetv)
  - `foreground`: leave empty

> **TV 틀어/켜/실행**: When the user says 'TV 틀어', 'TV 켜', 'TV 실행', or similar requests to turn on/start TV, always launch 'livetv' (Live TV app).

> **Retry rule**: If the launch fails with 'no app found', retry with the English name or a shorter keyword before telling the user the app is not found.

```
sh skills/tv-control/scripts/app-control.sh {action} {target}
```

## Safety Guidelines

- For every tv-control request, always call the appropriate tool first. Do not respond with text alone — execute the tool, then reply based on the result.
- If a command fails, show the error output and suggest alternatives.
