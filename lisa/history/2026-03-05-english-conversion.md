# 2026-03-05: Full Review, English Conversion, Sensitive Data Cleanup

## 1. Full Review of lisa/ Materials

Conducted comprehensive review of all files under `lisa/` directory, identifying:
- Sensitive data exposure in config.arm64.toml (API key, Telegram token in comments)
- setup-guide.md step numbering gap (3 -> 4 -> 6, missing step 5)
- deploy-target.ps1 config backup without timestamp
- gogcli-oauth-setup-guide.md missing from setup guide and SUMMARY.md project structure
- gogcli/ directory missing from SUMMARY.md project structure

## 2. config.arm64.toml.example

Created template file with all sensitive data replaced by placeholders:
- API key: `<Azure API Key>`
- Provider URL: `<resource>.openai.azure.com`
- Telegram: commented out with `YOUR_BOT_TOKEN` / `YOUR_USER_ID`

## 3. Sensitive Data Cleanup in config.arm64.toml

- Removed real Telegram bot token and user ID from commented-out section
- Replaced with generic `YOUR_BOT_TOKEN` / `YOUR_USER_ID`
- Note: API key remains as it's needed for actual deployment (not committed to upstream)

## 4. deploy-target.ps1 Config Backup Fix

Changed backup filename from `config.toml.bak` (overwrites) to `config.toml.bak.$(date +%s)` (timestamped), matching deploy-target.sh behavior.

## 5. setup-guide.md Fixes

- Fixed step numbering: 3 -> 4 -> 5 (was 3 -> 4 -> 6)
- Added "Related Guides" section with link to gogcli-oauth-setup-guide.md
- Updated project structure to include:
  - config.arm64.toml.example
  - deploy-linux.sh, clean-target.sh, clean-linux.sh scripts
  - deploy-linux-guide.md, gogcli-oauth-setup-guide.md docs
  - release/x86_64/ directory
  - tv-control/ skill

## 6. English Conversion

Converted all lisa/ materials from Korean to English:

### Config files (3)
- config.default.toml — comments
- config.linux.toml — comments
- config.arm64.toml — comments

### Profile files (7)
- .env.example
- SOUL.md
- AGENTS.md
- USER.md.example
- lisa.env.example (already English)
- skills/calendar/SKILL.md
- skills/weather/SKILL.md
- skills/tv-control/SKILL.md (removed Korean from description)

### Documentation (4)
- setup-guide.md
- deploy-target-guide.md
- deploy-linux-guide.md
- gogcli-oauth-setup-guide.md

### Other (2)
- open-issues/README.md
- history/SUMMARY.md

## 7. SUMMARY.md Updates

- Added gogcli-oauth-setup-guide.md to project structure docs section
- Added gogcli/ source directory to project structure
- Added config.arm64.toml.example to project structure
- Added 2026-03-05-english-conversion.md history entry

## 8. Template File (.example) Guidance

Added instructions across setup script and all guide docs for creating actual config files from `.example` templates.

### setup-lisa.sh
- Converted all Korean text to English
- `.env` missing message now says "Create it from the example template"
- USER.md missing message now says "Create it from the example template"

### setup-guide.md
- Added new step 3 "Create Config Files from Templates" with `cp` commands for all `.example` files
- Renumbered subsequent steps (4 -> 5 -> 6)
- Added note: actual files must be created from `.example` templates
- Added `config.linux.toml.example` to project structure

### deploy-target-guide.md
- Added "First-time setup" callout in Prerequisites with `cp` commands for config.arm64.toml, lisa.env, USER.md

### deploy-linux-guide.md
- Added "First-time setup" callout in Prerequisites with `cp` commands for config.linux.toml, lisa.env, USER.md

## 9. tv-control SKILL.md Target TV Clarification

Rewrote the Target TV section to use a table format explaining each Location/IP combination clearly.
Key fix: when Location is `local` and IP is `N/A`, the AI was incorrectly refusing to execute commands.
Added explicit note that `local` + `N/A` is the normal state and commands should run directly via `exec`.

## Modified Files

| File | Change |
|------|--------|
| `lisa/config/config.arm64.toml` | Removed exposed Telegram token, English comments |
| `lisa/config/config.arm64.toml.example` | New — config template (no sensitive data) |
| `lisa/config/config.default.toml` | English comments |
| `lisa/config/config.linux.toml` | English comments |
| `lisa/profiles/.env.example` | English |
| `lisa/profiles/lisa/SOUL.md` | English |
| `lisa/profiles/lisa/AGENTS.md` | English |
| `lisa/profiles/lisa/USER.md.example` | English |
| `lisa/profiles/lisa/skills/calendar/SKILL.md` | English |
| `lisa/profiles/lisa/skills/weather/SKILL.md` | English |
| `lisa/profiles/lisa/skills/tv-control/SKILL.md` | Removed Korean from description |
| `lisa/docs/setup-guide.md` | English, step fix, gogcli ref, project structure |
| `lisa/docs/deploy-target-guide.md` | English |
| `lisa/docs/deploy-linux-guide.md` | English |
| `lisa/docs/gogcli-oauth-setup-guide.md` | English |
| `lisa/open-issues/README.md` | English |
| `lisa/history/SUMMARY.md` | English, updated structure |
| `lisa/scripts/deploy-target.ps1` | Config backup timestamp fix |
| `lisa/scripts/setup-lisa.sh` | English conversion, .example template guidance |
