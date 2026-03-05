# Lisa Project Change History Summary

> Last updated: 2026-03-05

## Timeline

### 2026-03-05: Full review, English conversion, sensitive data cleanup

- All lisa/ materials converted from Korean to English
- config.arm64.toml: created `.example` template with sensitive data removed
- config.arm64.toml: removed exposed Telegram bot token/user ID from comments
- deploy-target.ps1: added timestamp to config backup filename
- setup-guide.md: fixed step numbering (was 3->4->6, now 3->4->5)
- setup-guide.md: added gogcli-oauth-setup-guide.md reference and updated project structure
- SUMMARY.md: added gogcli-oauth-setup-guide.md and gogcli/ to project structure
- setup-lisa.sh: converted to English
- All guides + setup script: added `.example` template file copy instructions for first-time setup
- tv-control SKILL.md: clarified Target TV section — `local` + `N/A` is normal, commands run directly

| File | Change |
|------|--------|
| `lisa/config/config.arm64.toml` | Removed exposed Telegram token from comments, English comments |
| `lisa/config/config.arm64.toml.example` | New — config template with no sensitive data |
| `lisa/config/config.default.toml` | English comments |
| `lisa/config/config.linux.toml` | English comments |
| `lisa/profiles/.env.example` | English |
| `lisa/profiles/lisa/SOUL.md` | English |
| `lisa/profiles/lisa/AGENTS.md` | English |
| `lisa/profiles/lisa/USER.md.example` | English |
| `lisa/profiles/lisa/lisa.env.example` | English (already was) |
| `lisa/profiles/lisa/skills/calendar/SKILL.md` | English |
| `lisa/profiles/lisa/skills/weather/SKILL.md` | English |
| `lisa/profiles/lisa/skills/tv-control/SKILL.md` | Removed Korean from description |
| `lisa/docs/setup-guide.md` | English, step numbering fix, gogcli reference, updated project structure |
| `lisa/docs/deploy-target-guide.md` | English, added gogcli-oauth-setup-guide.md cross-reference |
| `lisa/docs/deploy-linux-guide.md` | English |
| `lisa/docs/gogcli-oauth-setup-guide.md` | English |
| `lisa/open-issues/README.md` | English |
| `lisa/history/SUMMARY.md` | English, added missing entries |
| `lisa/scripts/deploy-target.ps1` | Config backup now includes timestamp |
| `lisa/scripts/setup-lisa.sh` | English conversion, .example template guidance |

-> Detail: [2026-03-05-english-conversion.md](2026-03-05-english-conversion.md)

---

### 2026-03-05: clean-target.sh individual confirmation + tv-control Target TV + deploy-linux.sh

- clean-target.sh: separated target cleanup and local token deletion into individual confirmations
- tv-control skill: added Target TV section (N/A/local/remote + IP)
- Deploy scripts: added step 7 for Target TV configuration
- deploy-linux.sh: added Ubuntu Linux deploy script (local/remote support)
- clean-linux.sh: added Ubuntu Linux cleanup script (local/remote support)
- Guide docs split: new Linux-specific guide, restored webOS guide
- x86_64 build: added zeroclaw + gog binaries, deploy-linux.sh multi-architecture support
- config.linux.toml: added Linux-specific config, used by deploy-linux.sh

| File | Change |
|------|--------|
| `lisa/scripts/clean-target.sh` | Single confirmation -> separate target/local confirmations |
| `lisa/profiles/lisa/skills/tv-control/SKILL.md` | Added Target TV section |
| `lisa/scripts/deploy-target.sh` | Added step 7 for Target TV setup, step renumbering |
| `lisa/scripts/deploy-target.ps1` | Same changes |
| `lisa/scripts/deploy-linux.sh` | New — Ubuntu Linux deploy script |
| `lisa/scripts/clean-linux.sh` | New — Ubuntu Linux cleanup script |
| `lisa/docs/deploy-linux-guide.md` | New — Linux deploy guide (split) |
| `lisa/docs/deploy-target-guide.md` | Restored to webOS TV only |
| `lisa/release/x86_64/zeroclaw` | New — x86_64 binary |
| `lisa/release/x86_64/gog` | New — x86_64 gog binary |
| `lisa/config/config.linux.toml` | New — Linux-specific config |

-> Detail: [2026-03-05-clean-script-confirm.md](2026-03-05-clean-script-confirm.md)

---

### 2026-03-04: gog (calendar) integration and deploy automation

Improved deploy scripts to automatically handle gog OAuth authentication, lisa.env creation, and credential transfer.
Stabilized post-deploy tests and added target cleanup script.

**Key changes:**
- gogcli ARM64 build (21MB static binary)
- history compaction temperature hardcoded (0.2) -> parameterized
- Deploy script step 7: integrated gog auto-setup (OAuth `--manual` + lisa.env auto-creation)
- gog binary/credentials/lisa.env target transfer
- USER.md target transfer added
- lisa.env `export` prefix missing fix (child process env access issue)
- `gog calendar list` -> `gog calendar calendars` command fix
- Post-deploy test stabilization (webOS curl compatibility, gog path detection, `|| true`)
- `clean-target.sh` — target cleanup and local token deletion script
- `shell_env_passthrough` — GOG env var forwarding for shell tool

| File | Change |
|------|--------|
| `lisa/scripts/deploy-target.sh` | gog auto-setup, lisa.env, PATH, test stabilization |
| `lisa/scripts/deploy-target.ps1` | Same changes |
| `lisa/scripts/clean-target.sh` | New — target cleanup script |
| `src/agent/loop_/history.rs` | temperature parameterization |
| `lisa/profiles/lisa/lisa.env.example` | New — env var template |
| `lisa/config/config.arm64.toml` | Added `shell_env_passthrough` (GOG env var forwarding) |
| `lisa/profiles/lisa/lisa.env` | Added `export` prefix, added `GOG_KEYRING_BACKEND=file` |

-> Detail: [2026-03-04-gog-integration.md](2026-03-04-gog-integration.md)

---

### 2026-03-04: Deploy script improvements (ProxyJump, PicoClaw pattern, English)

Comprehensive deploy script improvement: PicoClaw pattern, ProxyJump support, Korean->English conversion, SCP/JSON compatibility fixes.

**Key changes:**
- deploy-target.ps1: PicoClaw pattern refactoring (Invoke-SSH/Invoke-SCP/Write-RemoteFile helpers)
- deploy-target.ps1: `-ProxyJump` parameter (SSH `-J` option)
- deploy-target.ps1: SCP `-O` flag (ProxyJump SFTP compatibility)
- deploy-target.ps1: chat test JSON via `Write-RemoteFile` (escape workaround)
- deploy-target.sh/ps1: Korean/emoji -> English/ASCII conversion
- deploy-target.sh: `[1/N]` step progress display

| File | Change |
|------|--------|
| `lisa/scripts/deploy-target.ps1` | PicoClaw pattern, ProxyJump, SCP -O, JSON fix, English |
| `lisa/scripts/deploy-target.sh` | English, step display |
| `lisa/scripts/deploy-target.bat` | English |
| `lisa/docs/deploy-target-guide.md` | Windows troubleshooting update |

-> Detail: [2026-03-04-deploy-script-improvements.md](2026-03-04-deploy-script-improvements.md)

---

### 2026-03-03: Windows deploy wrapper (deploy-target.bat)

Added `.bat` wrapper to bypass PowerShell ExecutionPolicy script blocking on Windows.

| File | Change |
|------|--------|
| `lisa/scripts/deploy-target.bat` | New — ExecutionPolicy Bypass wrapper |
| `lisa/docs/deploy-target-guide.md` | Updated Windows deploy commands, added troubleshooting |

---

### 2026-03-03: Daemon gateway test addition

Added daemon gateway API tests to deploy script. After daemon startup, sequentially verifies `/health`, `/pair`, `/api/chat` endpoints.

| File | Change |
|------|--------|
| `lisa/scripts/deploy-target.sh` | Added step 12-7 gateway /health, 12-8 /pair + /api/chat tests |
| `lisa/scripts/deploy-target.ps1` | Same changes |
| `lisa/docs/deploy-target-guide.md` | Added daemon test guide section, updated auto-test table, added troubleshooting |

-> Detail: [2026-03-03-daemon-gateway-test.md](2026-03-03-daemon-gateway-test.md)

---

### 2026-03-03: Telegram channel setup

Added Telegram bot configuration to enable remote control of the target (webOS TV). Receives/responds to messages via Telegram long-polling in daemon mode.

| File | Change |
|------|--------|
| `lisa/config/config.arm64.toml` | Added `[channels_config.telegram]` section (sample values) |
| `lisa/scripts/deploy-target.sh` | Added Telegram Bot API getMe test in step 12 |
| `lisa/scripts/deploy-target.ps1` | Same changes |
| `lisa/docs/deploy-target-guide.md` | Added Telegram setup guide, troubleshooting |

-> Detail: [2026-03-03-telegram-channel.md](2026-03-03-telegram-channel.md)

---

### 2026-03-03: device-control skill picoclaw path removal

Skill referenced picoclaw-specific path (`~/.picoclaw/workspace/`), making it unusable with other AI agents. Changed to relative path (`apps.json`) for agent-agnostic operation.

| File | Change |
|------|--------|
| `lisa/profiles/lisa/skills/device-control/SKILL.md` | `~/.picoclaw/workspace/apps.json` -> `apps.json` (2 places) |

-> Detail: [2026-03-03-device-control-picoclaw-fix.md](2026-03-03-device-control-picoclaw-fix.md)

---

### 2026-03-03: Open issues management system

Added `lisa/open-issues/` directory and `lisa/scripts/issues.sh` management script. Local issue creation/viewing/closing/deletion/summary.

**Script functions:** `new`, `list` (with filtering), `show`, `close`, `reopen`, `delete`, `summary`

| File | Change |
|------|--------|
| `lisa/open-issues/README.md` | New — issue file format and usage |
| `lisa/scripts/issues.sh` | New — issue management CLI script |

-> Detail: [2026-03-03-open-issues.md](2026-03-03-open-issues.md)

---

### 2026-03-03: Post-deploy automatic functional tests

Added automatic test step (step 12) to deploy script after installation. Verifies agent/daemon mode execution and per-skill commands.

**Test items:** agent single message, device-control (getForegroundAppInfo, getVolume), weather (wttr.in), calendar (gog check), daemon start/status

| File | Change |
|------|--------|
| `lisa/scripts/deploy-target.sh` | Added step 12 automatic tests |
| `lisa/scripts/deploy-target.ps1` | Same changes |
| `lisa/docs/deploy-target-guide.md` | Added post-deploy automatic test section |

-> Detail: [2026-03-03-deploy-auto-test.md](2026-03-03-deploy-auto-test.md)

---

### 2026-03-03: config.arm64.toml autonomy schema fix

Fixed `[autonomy]` section parsing error. Moved top-level `auto_approve`, `allowed_commands` into `[autonomy]` and added required fields (`workspace_only`, `forbidden_paths`, `max_actions_per_hour`, `max_cost_per_day_cents`).

| Issue | Cause | Fix |
|-------|-------|-----|
| `missing field workspace_only` | Required fields missing when `[autonomy]` section is explicit | Added all required fields, moved top-level fields into `[autonomy]` |

| File | Change |
|------|--------|
| `lisa/config/config.arm64.toml` | Full `[autonomy]` schema fix |
| `lisa/docs/deploy-target-guide.md` | Updated config example |

---

### 2026-03-03: Added luna-send to allowed_commands

Added `luna-send` to `allowed_commands` so the device-control skill can execute it.

| File | Change |
|------|--------|
| `lisa/config/config.arm64.toml` | Added `luna-send` to `allowed_commands` |

-> Detail: [2026-03-03-allowed-commands-luna-send.md](2026-03-03-allowed-commands-luna-send.md)

---

### 2026-03-03: Agent mode temperature error fix

Fixed error caused by gpt-5-mini not supporting temperature=0.7.

| Issue | Cause | Fix |
|-------|-------|-----|
| `temperature does not support 0.7` | agent CLI `--temperature` default hardcoded to 0.7 | Explicitly set `-t 1.0` in `lisa-agent.sh` |

| File | Change |
|------|--------|
| `lisa/scripts/deploy-target.sh` | Added `-t 1.0` to agent script, added temperature comment |
| `lisa/scripts/deploy-target.ps1` | Same changes |
| `lisa/docs/deploy-target-guide.md` | Added temperature column to mode comparison table, agent note, troubleshooting |

-> Detail: [2026-03-03-agent-temperature-fix.md](2026-03-03-agent-temperature-fix.md)

---

### 2026-03-03: Agent mode runtime error fixes

Fixed 2 errors when running agent mode on target.

| Issue | Cause | Fix |
|-------|-------|-----|
| Skill script blocked | `allow_scripts` default is `false` | Added `[skills] allow_scripts = true` to config |
| `channel_ack_config` schema error | `rules` array missing `items` | Fixed schema in `src/tools/channel_ack_config.rs` |

| File | Change |
|------|--------|
| `lisa/config/config.arm64.toml` | Added `[skills] allow_scripts = true` |
| `src/tools/channel_ack_config.rs` | Fixed `rules` schema for Azure OpenAI compatibility |
| `lisa/docs/deploy-target-guide.md` | Updated config example, troubleshooting |

-> Detail: [2026-03-03-agent-runtime-fixes.md](2026-03-03-agent-runtime-fixes.md)

---

### 2026-03-03: Agent mode support

Enabled agent mode (interactive chat / one-shot message) on target in addition to daemon mode.

- Deploy script: added `lisa-agent.sh` generation
- Guide: added daemon/agent mode comparison table, agent examples
- Added `/home/root/lisa/lisa-agent.sh` to target directory

| File | Change |
|------|--------|
| `lisa/scripts/deploy-target.sh` | Added `lisa-agent.sh` generation, updated completion message |
| `lisa/scripts/deploy-target.ps1` | Same changes |
| `lisa/docs/deploy-target-guide.md` | Added daemon/agent guide, config example update, directory structure update |

-> Detail: [2026-03-03-agent-mode-support.md](2026-03-03-agent-mode-support.md)

---

### 2026-03-03: Config schema fix

Fixed 2 config parsing errors on target.

| Item | Before | After |
|------|--------|-------|
| `[memory]` | `enabled = true` | `backend = "markdown"`, `auto_save = true` |
| `[security]` | `sandbox = false` | `[security.sandbox]` `enabled = false` |

-> Detail: [2026-03-03-config-schema-fix.md](2026-03-03-config-schema-fix.md)

---

### 2026-03-03: Target architecture ARM32 -> ARM64

Changed target binary from ARM32 to ARM64. Updated config, scripts, and docs throughout.

| File | Change |
|------|--------|
| `lisa/config/config.arm32.toml` | -> Renamed to `lisa/config/config.arm64.toml` |
| `lisa/scripts/deploy-target.sh` | Binary/config paths arm32 -> arm64 |
| `lisa/scripts/deploy-target.ps1` | Binary/config paths arm32 -> arm64 |
| `lisa/docs/deploy-target-guide.md` | ARM32 -> ARM64 throughout |
| `lisa/docs/setup-guide.md` | Updated project structure arm32 -> arm64 |

-> Detail: [2026-03-03-arm64-migration.md](2026-03-03-arm64-migration.md)

---

### 2026-03-03: device-control skill addition

Added webOS TV device control skill and updated deploy scripts.

**Added skill:** `device-control` — luna-send based webOS TV control
- Launch/open apps (luna://com.webos.applicationManager)
- Channel control (up/down/go to number)
- Volume control (up/down/set level)
- Get foreground app

| File | Change |
|------|--------|
| `lisa/profiles/lisa/skills/device-control/SKILL.md` | New — device control skill definition |
| `lisa/profiles/lisa/skills/device-control/scripts/go-to-channel.sh` | New — channel number navigation script |
| `lisa/scripts/deploy-target.sh` | Added skill script execute permission |
| `lisa/scripts/deploy-target.ps1` | Added skill script execute permission |
| `lisa/docs/deploy-target-guide.md` | Updated for device-control skill |

-> Detail: [2026-03-03-device-control-skill.md](2026-03-03-device-control-skill.md)

---

### 2026-03-03: Target deploy infrastructure

Built scripts, config, and guide docs for deploying Lisa to webOS TV (ARM64) target.

**Key work:**
- `deploy-target.sh` (Linux/macOS), `deploy-target.ps1` (Windows) deploy scripts
- `/etc/hosts` Read-Only workaround — `mount --bind` + `.profile` hook
- ARM64 binary deployment and Azure OpenAI connection test completed (192.168.0.10)

**Config structure change:**
- `config.shared.toml` -> `lisa/config/config.default.toml` (default/local)
- New `lisa/config/config.arm64.toml` (ARM64 target complete config)
- Independent per-target config management (direct transfer, no merge needed)

| File | Change |
|------|--------|
| `lisa/config/config.arm64.toml` | New — ARM64 target-specific config |
| `lisa/config/config.default.toml` | Moved from `profiles/lisa/config.shared.toml` |
| `lisa/scripts/deploy-target.sh` | New — Linux/macOS deploy script |
| `lisa/scripts/deploy-target.ps1` | New — Windows deploy script |
| `lisa/scripts/setup-lisa.sh` | Modified — config path change |
| `lisa/docs/deploy-target-guide.md` | New — target deploy guide |
| `lisa/docs/setup-guide.md` | Modified — project structure, config path update |

-> Detail: [2026-03-03-target-deploy.md](2026-03-03-target-deploy.md)

---

### 2026-03-03: lisa/ directory organization

Created `lisa/` directory under project root and moved Lisa-specific files to separate from upstream.

- Moved 9 files under `lisa/` (profiles, scripts, docs, release)
- Established `lisa/history/` change history tracking system

-> Detail: [2026-03-03-initial-reorganization.md](2026-03-03-initial-reorganization.md)

---

### 2026-03-02: Upstream code modifications

Initial Lisa project setup and upstream ZeroClaw code modifications for Azure OpenAI support.

| Commit | Subject | Key Content |
|--------|---------|-------------|
| `495d6324` | feat(profiles): add Lisa profile and setup script | Added `.env` to `.gitignore`, created profiles/scripts/docs |
| `13cdca54` | feat(profiles): make model/provider configurable | `.env` provider/model injection support |
| `2a2c4cf0` | feat(providers): add auth_header config | Azure OpenAI `api-key` auth header support (10 files modified, 9 tests added) |
| `3cedb677` | feat(setup): auto-inject Azure OpenAI profile | Azure OpenAI auto-injection in setup-lisa.sh |
| `a3ee6fb1` | feat(skills): add weather and calendar skills | Calendar/weather skill addition |

-> Detail: [upstream-code-modifications.md](upstream-code-modifications.md)

---

## Current Project Structure

```
lisa/
├── config/
│   ├── config.default.toml       # Default/local config (setup-lisa.sh)
│   ├── config.linux.toml         # Linux (Ubuntu) target (deploy-linux.sh)
│   ├── config.arm64.toml         # webOS TV (ARM64) target (deploy-target.sh)
│   ├── config.arm64.toml.example # ARM64 config template (no sensitive data)
│   └── config.linux.toml.example # Linux config template (no sensitive data)
├── profiles/
│   ├── .env.example              # Secret template
│   └── lisa/
│       ├── SOUL.md               # Lisa's personality
│       ├── AGENTS.md             # Agent rules
│       ├── USER.md.example       # User info template
│       ├── lisa.env.example      # Target env vars template
│       └── skills/
│           ├── calendar/         # Calendar skill
│           ├── weather/          # Weather skill
│           └── tv-control/       # TV control skill
├── gogcli/                       # gog (Google Calendar CLI) source
├── open-issues/
│   └── README.md                 # Issue file format and usage
├── scripts/
│   ├── setup-lisa.sh             # Local setup
│   ├── deploy-target.sh          # webOS TV deploy (Linux/macOS)
│   ├── deploy-target.ps1         # webOS TV deploy (Windows)
│   ├── deploy-target.bat         # Windows ExecutionPolicy bypass wrapper
│   ├── deploy-linux.sh           # Linux (Ubuntu) deploy
│   ├── clean-target.sh           # webOS TV cleanup
│   ├── clean-linux.sh            # Linux (Ubuntu) cleanup
│   └── issues.sh                 # Issue management CLI
├── docs/
│   ├── setup-guide.md            # Setup guide
│   ├── deploy-target-guide.md    # webOS TV deploy guide
│   ├── deploy-linux-guide.md     # Linux (Ubuntu) deploy guide
│   └── gogcli-oauth-setup-guide.md # Google Calendar OAuth setup guide
├── release/
│   ├── arm64/
│   │   ├── zeroclaw              # ARM64 binary
│   │   └── gog                   # ARM64 gog (calendar CLI)
│   └── x86_64/
│       ├── zeroclaw              # x86_64 binary
│       └── gog                   # x86_64 gog (calendar CLI)
└── history/
    ├── SUMMARY.md                # This file
    ├── 2026-03-05-english-conversion.md
    ├── 2026-03-05-clean-script-confirm.md
    ├── 2026-03-04-gog-integration.md
    ├── 2026-03-04-deploy-script-improvements.md
    ├── 2026-03-03-initial-reorganization.md
    ├── 2026-03-03-target-deploy.md
    ├── 2026-03-03-device-control-skill.md
    ├── 2026-03-03-arm64-migration.md
    ├── 2026-03-03-agent-mode-support.md
    ├── 2026-03-03-agent-runtime-fixes.md
    ├── 2026-03-03-agent-temperature-fix.md
    ├── 2026-03-03-allowed-commands-luna-send.md
    ├── 2026-03-03-open-issues.md
    ├── 2026-03-03-device-control-picoclaw-fix.md
    ├── 2026-03-03-telegram-channel.md
    ├── 2026-03-03-daemon-gateway-test.md
    ├── 2026-03-03-deploy-auto-test.md
    ├── 2026-03-03-config-schema-fix.md
    └── upstream-code-modifications.md
```

## Deploy Test Results

| Target | IP | Result | Notes |
|--------|-----|--------|-------|
| webOS TV (ARM64) | 192.168.0.10 | Pass | Azure OpenAI (gpt-5-mini) response confirmed |
