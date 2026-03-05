# Lisa Target Deploy Guide (webOS TV)

How to deploy Lisa to a webOS TV (ARM64) target.

> **Linux (Ubuntu) deployment**: See [deploy-linux-guide.md](deploy-linux-guide.md).

## Prerequisites

#### Host PC
- SSH client (OpenSSH)
- Lisa repository cloned
- `lisa/config/config.arm64.toml` configured
- `lisa/profiles/lisa/lisa.env` configured (if using calendar skill)

> **First-time setup:** If only `.example` template files exist (no actual config files), create them first:
> ```bash
> cp lisa/config/config.arm64.toml.example lisa/config/config.arm64.toml
> cp lisa/profiles/lisa/lisa.env.example lisa/profiles/lisa/lisa.env
> cp lisa/profiles/lisa/USER.md.example lisa/profiles/lisa/USER.md
> ```
> Then edit each file with your actual values (API keys, account info, etc.).

#### Target Device (webOS TV)
- SSH accessible (root, no password)

## SSH Key Registration

```bash
ssh-keygen -t ed25519                # Generate key (if none exists)
ssh-copy-id root@<target-IP>         # Copy key to target
ssh root@<target-IP> "echo ok"       # Test connection
```

## Configuration

### lisa/config/config.arm64.toml

Complete config file for the ARM64 target.
Copied directly to `~/.zeroclaw/config.toml` on the target during deployment.

See `config.arm64.toml.example` for a template with placeholder values.

### Telegram Channel Setup

Control Lisa remotely via a Telegram bot. Only works in daemon mode.

#### 1) Create a Bot

1. Send `/newbot` to [@BotFather](https://t.me/BotFather) on Telegram
2. Enter a bot name and username (e.g., `LisaHomeBot`, `lisa_home_bot`)
3. Enter the issued Bot Token in `bot_token` in `config.arm64.toml`

#### 2) Get Your User ID

1. Send any message to [@userinfobot](https://t.me/userinfobot) on Telegram
2. Enter the `Id` value from the response into `allowed_users`

#### 3) Edit config.arm64.toml

```toml
[channels_config.telegram]
bot_token = "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
allowed_users = ["987654321"]
mention_only = false
stream_mode = "partial"
ack_enabled = true
```

| Field | Description | Default |
|-------|-------------|---------|
| `bot_token` | Token from @BotFather (required) | — |
| `allowed_users` | Allowed user ID list (required, empty = block all) | — |
| `mention_only` | Only respond to @mentions in group chats | `false` |
| `stream_mode` | `"off"` (send after completion) or `"partial"` (progressive display) | `"off"` |
| `ack_enabled` | Show emoji reaction on message receipt | `true` |

#### 4) Deploy and Test

```bash
# Deploy (full redeploy including config)
./lisa/scripts/deploy-target.sh <target-IP>

# Start daemon (Telegram only works in daemon mode)
ssh root@<target-IP> '/home/root/lisa/start-lisa.sh'

# Send a message to the bot on Telegram to verify
```

> **Note:** Telegram uses long-polling, so no additional ports need to be opened on the target.

### Calendar (gog) Setup

The calendar skill accesses Google Calendar via the `gog` CLI.
Since the target has no browser, **authenticate on your local PC first, then copy to the target**.

For detailed OAuth setup instructions, see [gogcli-oauth-setup-guide.md](gogcli-oauth-setup-guide.md).

#### 1) Google Cloud Console Setup

1. Create a project in [Google Cloud Console](https://console.cloud.google.com/)
2. Enable Google Calendar API
3. Create OAuth 2.0 Client ID (Desktop app) -> download `client_secret.json`

#### 2) Authenticate gog on Local PC

```bash
# Install gog
brew install steipete/tap/gogcli       # macOS
# or: go install github.com/steipete/gogcli/cmd/gog@latest

# Register OAuth client
gog auth credentials /path/to/client_secret.json

# Authenticate Google account (browser opens)
gog auth add your@gmail.com --services calendar --manual

# Verify
gog calendar events primary --from $(date +%Y-%m-%dT00:00:00) --to $(date +%Y-%m-%dT23:59:59)
```

#### 3) Configure lisa.env

```bash
cp lisa/profiles/lisa/lisa.env.example lisa/profiles/lisa/lisa.env
```

Enter gog environment variables in `lisa.env`:
```bash
export GOG_ACCOUNT=your@gmail.com
export GOG_KEYRING_PASSWORD=your-keyring-password
export GOG_KEYRING_BACKEND=file
```

> `GOG_KEYRING_PASSWORD` is the password set during `gog auth add` in keyring=file mode.
> `GOG_KEYRING_BACKEND=file` forces file-based keyring instead of DBUS SecretService.

#### 4) Deploy

The deploy script handles this automatically:
- `~/.config/gogcli/` -> target `/home/root/.config/gogcli/` (auth files)
- `lisa/profiles/lisa/lisa.env` -> target `/home/root/lisa/lisa.env` (env vars)
- `lisa/release/arm64/gog` -> target `/home/root/lisa/gog` (binary)

```bash
./lisa/scripts/deploy-target.sh <target-IP>
```

#### 5) Verify on Target

```bash
ssh root@<target-IP> '. /home/root/lisa/lisa.env; gog calendar events primary --from $(date +%Y-%m-%dT00:00:00) --to $(date +%Y-%m-%dT23:59:59) --json'
```

> **Note:** Internet access is required on the target to access Google Calendar API.

### TV Control (tv-control) Setup

The deploy script prompts you to configure the target TV for the tv-control skill.

```
[7/14] Configuring Target TV...
  Select TV location for tv-control skill:
    1) N/A    — no target TV (skip tv-control)
    2) local  — commands run directly on this device
    3) remote — commands run via SSH to another TV
  Choice [1/2/3] (default: 1):
```

| Choice | Location | Behavior |
|--------|----------|----------|
| 1 (default) | N/A | tv-control skill disabled — no TV control commands attempted |
| 2 | local | Run commands directly on this device via `exec` tool |
| 3 | remote | Run commands via `ssh root@{ip} '<command>'` to remote TV |

The selected value is automatically applied to the Target TV section in `skills/tv-control/SKILL.md` on the target.
You can re-select during redeployment.

### Adding New Targets

Add target-specific config files in the `lisa/config/` directory:
- `config.arm64.toml` — webOS TV (ARM64)
- `config.linux.toml` — Linux (Ubuntu)
- `config.default.toml` — local/general (used by setup-lisa.sh)

## Deploy

### Linux / macOS

```bash
# Specify IP directly
./lisa/scripts/deploy-target.sh 192.168.0.10

# Enter IP at prompt
./lisa/scripts/deploy-target.sh
```

### Windows

Run directly in PowerShell (recommended):
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\lisa\scripts\deploy-target.ps1 192.168.0.10
```

Run via .bat wrapper from cmd.exe:
```cmd
.\lisa\scripts\deploy-target.bat 192.168.0.10
```

> **Note:** Running `.bat` directly from PowerShell may cause "Access denied" errors.
> Use the `powershell -ExecutionPolicy Bypass` method above, or run from cmd.exe.

## What the Deploy Script Does

1. **Transfer binary** — `zeroclaw` + `gog` (if available) -> `/home/root/lisa/`
2. **Transfer config.toml** — `config.arm64.toml` -> `/home/root/.zeroclaw/config.toml`
3. **Transfer workspace** — `SOUL.md`, `AGENTS.md`, `USER.md`, skills -> `/home/root/.zeroclaw/workspace/`
4. **Skill script permissions** — Set execute permission on `.sh` files in `skills/`
5. **Target TV setup** — tv-control skill target TV location (N/A/local/remote) and IP
6. **gog auth transfer** — `~/.config/gogcli/` -> `/home/root/.config/gogcli/` (if available)
7. **lisa.env transfer** — `lisa/profiles/lisa/lisa.env` -> `/home/root/lisa/lisa.env` (if available)
8. **/etc/hosts setup** — Add Azure OpenAI domain resolution hosts entry
9. **Auto bind mount + PATH** — Apply RW hosts on SSH login + add `/home/root/lisa` to PATH
10. **Start scripts** — Create `start-lisa.sh` (daemon), `lisa-agent.sh` (agent)
11. **Functional tests** — Run agent/daemon mode, per-skill command verification (see below)

## /etc/hosts Handling

webOS TV's `/etc/hosts` is in a Read-Only filesystem.

```
/etc/hosts (Read-Only)
       ^ bind mount
/home/root/hosts (Read-Write)
       └── 10.182.173.75 tvdevops.openai.azure.com
```

- Copy to `/home/root/hosts` and add domain entry
- Overlay onto `/etc/hosts` via `mount --bind`
- Auto-applied on SSH login via `.profile` hook

## Running on Target

Two execution modes are supported:

| Mode | Script | Description | Temperature |
|------|--------|-------------|-------------|
| **daemon** | `start-lisa.sh` | Background service (gateway + channels + scheduler) | Uses `default_temperature` from config |
| **agent** | `lisa-agent.sh` | Interactive chat or one-shot message | CLI default 0.7 -> explicitly set `-t 1.0` |

### Daemon Mode (Background Service)

```bash
# Start daemon
ssh root@<target-IP> '/home/root/lisa/start-lisa.sh'

# Check status
ssh root@<target-IP> '/home/root/lisa/zeroclaw status'
```

### Agent Mode (Interactive / One-shot)

> **Note:** The agent CLI hardcodes `--temperature` default to 0.7, ignoring config's `default_temperature`.
> Since gpt-5-mini only supports temperature=1.0, `lisa-agent.sh` explicitly sets `-t 1.0`.

```bash
# Interactive chat session
ssh root@<target-IP> '/home/root/lisa/lisa-agent.sh'

# One-shot message (execute and exit)
ssh root@<target-IP> '/home/root/lisa/lisa-agent.sh hello Lisa!'
ssh root@<target-IP> '/home/root/lisa/lisa-agent.sh what is the weather today?'
```

### Azure OpenAI Connection Test

```bash
ssh root@<target-IP> 'curl -s -H "Content-Type: application/json" \
  -H "api-key: <API_KEY>" \
  -d "{\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}],\"max_completion_tokens\":5}" \
  "https://tvdevops.openai.azure.com/openai/deployments/gpt-5-mini/chat/completions?api-version=2024-02-01"'
```

### Daemon Tests (Gateway API)

When the daemon is running, you can verify gateway API behavior from the host PC via SSH.

#### 1) Health Check (no auth required)

```bash
ssh root@<target-IP> "curl -s http://127.0.0.1:42617/health"
```

Example response:
```json
{"paired":false,"status":"ok","uptime_secs":42,"version":"0.x.x"}
```

#### 2) Pairing (issue Bearer token)

Pass the pairing code shown in daemon startup logs via the `X-Pairing-Code` header.

```bash
# Find pairing code in daemon log
# Example log: "Send: POST /pair with header X-Pairing-Code: 766826"

ssh root@<target-IP> "curl -s -X POST http://127.0.0.1:42617/pair \
  -H 'X-Pairing-Code: <PAIRING_CODE>'"
```

Example response:
```json
{"message":"Save this token — use it as Authorization: Bearer <token>","paired":true,"persisted":true,"token":"zc_a510d0..."}
```

> **Note:** The pairing code must be sent as an `X-Pairing-Code` **header**, not in the JSON body.

#### 3) Chat Test (/api/chat)

Use the token from pairing in the `Authorization: Bearer` header to send a chat request.

```bash
ssh root@<target-IP> "curl -s -X POST http://127.0.0.1:42617/api/chat \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer <TOKEN>' \
  -d '{\"message\":\"hello\"}'"
```

Example response:
```json
{"model":"gpt-5-mini","reply":"Hello! ...","session_id":null}
```

#### Gateway API Summary

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/health` | GET | Not required | Status check |
| `/pair` | POST | `X-Pairing-Code` header | Issue token (use code from daemon log) |
| `/api/chat` | POST | `Bearer <token>` | Chat request |

## Target Directory Structure

```
/home/root/
├── lisa/
│   ├── zeroclaw              # ARM64 binary
│   ├── gog                   # Google Calendar CLI (ARM64)
│   ├── start-lisa.sh         # Start daemon mode
│   ├── lisa-agent.sh         # Start agent mode
│   ├── lisa.env              # Environment variables (GOG_ACCOUNT, etc.)
│   └── bind-hosts.sh         # hosts bind mount script
├── hosts                     # RW hosts file
├── .profile                  # bind mount, PATH auto-apply hook
├── .config/
│   └── gogcli/               # gog OAuth auth files
│       ├── credentials.json
│       └── keyring/
└── .zeroclaw/
    ├── config.toml            # Deployed config
    └── workspace/
        ├── SOUL.md
        ├── AGENTS.md
        ├── USER.md
        └── skills/
            ├── calendar/
            ├── weather/
            └── tv-control/
                ├── SKILL.md
                └── scripts/
                    └── go-to-channel.sh
```

## Post-Deploy Automatic Tests

The deploy script automatically tests the following items in the final step:

| # | Test | Verification |
|---|------|-------------|
| 1 | agent: single message | Run `lisa-agent.sh hello` -> verify Azure OpenAI response |
| 2 | tv-control: getForegroundAppInfo | Verify `luna-send` command execution |
| 3 | tv-control: getVolume | Verify volume API call |
| 4 | weather: wttr.in query | Verify `curl`-based external API access |
| 5 | calendar: gog calendar calendars | Verify gog calendar list (skipped if gog not installed) |
| 6 | calendar: today's events | Verify gog today's events |
| 7 | daemon: zeroclaw status | Start daemon -> verify `zeroclaw status` |
| 8 | gateway: /health | Verify gateway `/health` endpoint response (no auth) |
| 9 | gateway: /pair + /api/chat | Issue token via pairing code -> verify `/api/chat` response |
| 10 | telegram: Bot API getMe | Verify Telegram API connection via bot_token (skipped if not configured) |

Test results are summarized as `N passed / N failed`.

## Redeploy

Run the same deploy script again. The existing config.toml is automatically backed up.

## Troubleshooting

| Symptom | Solution |
|---------|----------|
| Windows script blocked (`UnauthorizedAccess`) | Method 1: `powershell -NoProfile -ExecutionPolicy Bypass -File .\lisa\scripts\deploy-target.ps1 <IP>` Method 2: from cmd.exe `.\lisa\scripts\deploy-target.bat <IP>` |
| Windows `.bat` "Access denied" | PowerShell blocks `.bat` direct execution. Options: 1) `powershell -NoProfile -ExecutionPolicy Bypass -File .\lisa\scripts\deploy-target.ps1 <IP>` 2) Run from cmd.exe: `cmd /c .\lisa\scripts\deploy-target.bat <IP>` 3) Unblock files: `Unblock-File .\lisa\scripts\deploy-target.bat; Unblock-File .\lisa\scripts\deploy-target.ps1` |
| SSH connection failure | `ssh-copy-id root@<IP>` |
| Azure connection failure | Check domain with `cat /etc/hosts`, verify bind mount with `mount \| grep hosts` |
| bind mount not applied | Manually run `/home/root/lisa/bind-hosts.sh` |
| Library error | Check ARM64 binary dynamic library compatibility (`ldd` or `readelf -d`) |
| Skill script blocked (`script-like files are blocked`) | Add `[skills] allow_scripts = true` to config.toml |
| `Invalid schema for function` error | Binary rebuild needed (upstream schema bug fix) |
| `temperature does not support 0.7` | agent CLI default (0.7) conflicts with model — add `-t 1.0` to `lisa-agent.sh` (rerun deploy script) |
| gateway `/health` no response | Check if daemon is running (`ps aux \| grep zeroclaw`), verify `[gateway]` port/bind in config |
| gateway `/pair` Invalid pairing code | Pairing code must be sent via `X-Pairing-Code` **header** (not JSON body), code changes on daemon restart |
| gateway `/api/chat` Unauthorized | Issue token via `POST /pair` first, use `Authorization: Bearer <token>` header |
| Telegram bot not responding | 1) Check `bot_token` value 2) Ensure user ID is in `allowed_users` 3) Verify running in daemon mode (Telegram doesn't work in agent mode) |
| Telegram `getMe` failure | Verify Bot Token is valid: `curl -s https://api.telegram.org/bot<TOKEN>/getMe` |
| Calendar `gog not found` | Rerun deploy script (transfers gog binary), check PATH with `which gog` |
| Calendar `no credentials` | Complete `gog auth credentials` + `gog auth add` locally, then redeploy |
| Calendar `keyring error` | Check `GOG_KEYRING_PASSWORD` in `lisa.env`, ensure `chmod 600 lisa.env` |
