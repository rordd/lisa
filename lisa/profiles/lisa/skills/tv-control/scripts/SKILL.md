---
name: tv-control
description: Control this webOS TV via the exec tool. Supported actions: launch/open apps, get foreground app, change channels up/down/go to number, and control volume up/down/set level.
---

# TV Control

## Target TV

- **Location**: local
- **IP**: N/A

### How to interpret these values

| Location | IP | Meaning | How to execute commands |
|----------|-----|---------|----------------------|
| **N/A** | N/A | No target TV configured | Do NOT attempt any tv-control commands. Tell the user this skill is disabled. |
| **local** | N/A | This device IS the TV | Run commands directly with the `exec` tool. No SSH, no IP needed. The `luna-send` commands below will work as-is. |
| **remote** | `<IP>` | TV is on another device | Wrap every command with `ssh root@{IP} '<command>'` before passing to `exec`. |

> **Important:** When Location is **local**, the IP field is always N/A because commands run on this device itself. This is the normal and correct state — it does NOT mean the skill is unavailable.

## Core Principles

1. All operations in this skill MUST be executed using the `exec` tool.
2. Never output commands as plain text. Always invoke the `exec` tool to run them.
3. Every code block in this document is a command string to pass to the `exec` tool.
4. For remote TVs, prefix every command with `ssh root@{ip}` before execution.

## Mandatory Rules

1. Never guess command execution results or reuse previous results.
2. Every command must be actually executed by calling the exec tool each time, even if it is the same command.
3. Even if you have seen the result of the same command in a previous conversation, you must re-execute it to obtain the actual value at the current point in time.
4. "Generating" command results is prohibited. You must only use results obtained through tool calls.

## Commands

### Build apps.json

Build the app list cache. Run this command **only when apps.json does not exist yet** (i.e., "Read apps.json" fails with file not found).

```
luna-send -n 1 luna://com.webos.applicationManager/listApps '{}' | python3 -c "import sys,json;apps=json.load(sys.stdin)['apps'];print(json.dumps([{k:a[k] for k in ('title','id','appCategory') if k in a} for a in apps]))" > apps.json
```

### Read apps.json

Read the cached app list. Use when you need app information but apps.json content is not in conversation memory. If the file does not exist, run "Build apps.json" first.

```
cat apps.json
```

apps.json contains entries like:
- `{"title": "AirPlay", "id": "airplay"}` — no appCategory
- `{"title": "Home", "id": "com.webos.app.home", "appCategory": "home"}` — has appCategory
- `{"title": "Settings", "id": "com.palm.app.settings"}` — no appCategory

### Launch an app

Read apps.json to find the app information. If the app information contains a category, execute "Launch an app by category". Otherwise, execute "Launch an app by id" to launch the app.

#### Launch an app by id

- `app_id` (required): The ID of the app to launch.

```
luna-send -n 1 luna://com.webos.applicationManager/launch '{"id":"{app_id}"}'
```

#### Launch an app by category

- `app_category` (required): The category of the app to launch.

```
luna-send -n 1 luna://com.webos.applicationManager/launchDefaultApp '{"category":"{app_category}"}'
```

### Get foreground app

Get the app ID of the currently running foreground app.

```
luna-send -n 1 luna://com.webos.applicationManager/getForegroundAppInfo '{}'
```

The response contains `appId` field with the foreground app's ID.

### Channel control

Channel commands only work when the foreground app is `com.webos.app.livetv`.
**Always run "Get foreground app" first.** If it is not `com.webos.app.livetv`, launch Live TV using "Launch an app by id" with `com.webos.app.livetv`, then run "Get foreground app" again to confirm it is `com.webos.app.livetv` before proceeding.

#### Channel up

```
luna-send -n 1 -f luna://com.lge.inputgenerator/pushKeyEvent '{"eventtype":"key","keycodenum":402}'
```

#### Channel down

```
luna-send -n 1 -f luna://com.lge.inputgenerator/pushKeyEvent '{"eventtype":"key","keycodenum":403}'
```

#### Go to channel number

- `channel_number` (required, string): Channel number to switch to (e.g., "9", "12", "190").

```
sh skills/tv-control/scripts/go-to-channel.sh {channel_number}
```

### Volume up

Increase the volume by 1 step.

```
luna-send -n 1 luna://com.webos.service.audio/master/volumeUp '{}'
```

### Volume down

Decrease the volume by 1 step.

```
luna-send -n 1 luna://com.webos.service.audio/master/volumeDown '{}'
```

### Set volume

Set the volume to a specific level.

- `volume` (required, int, 0-100): Volume level to set.

```
luna-send -n 1 luna://com.webos.service.audio/master/setVolume '{"volume":{volume}}'
```

## Safety Guidelines

- If a command fails, show the error output and suggest alternatives.
