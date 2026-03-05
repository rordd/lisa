# Lisa Linux (Ubuntu) Deploy Guide

How to deploy Lisa on Linux (Ubuntu).
Supports both local installation and remote SSH installation.

## Supported Architectures

| Architecture | Release Path | Notes |
|-------------|-------------|-------|
| x86_64 (Intel/AMD) | `lisa/release/x86_64/` | Standard Linux server/PC |
| aarch64 (ARM64) | `lisa/release/arm64/` | ARM Linux (Raspberry Pi, etc.) |

The deploy script auto-detects the target architecture and installs the matching binary.
If no binary is found, it suggests building locally (same architecture only).

## Prerequisites

- Lisa repository cloned
- `lisa/config/config.linux.toml` configured (API key, etc.)
- Pre-built binary in `lisa/release/<arch>/`, or Rust build environment (`cargo build --release`)
- Remote install: SSH access to the target server (key authentication)

> **First-time setup:** If only `.example` template files exist (no actual config files), create them first:
> ```bash
> cp lisa/config/config.linux.toml.example lisa/config/config.linux.toml
> cp lisa/profiles/lisa/lisa.env.example lisa/profiles/lisa/lisa.env
> cp lisa/profiles/lisa/USER.md.example lisa/profiles/lisa/USER.md
> ```
> Then edit each file with your actual values (API keys, account info, etc.).

## Deploy

```bash
# Local install (auto-suggests build if no binary found)
./lisa/scripts/deploy-linux.sh

# Remote install
./lisa/scripts/deploy-linux.sh <remote-IP>
```

## What the Deploy Script Does

1. **Install binary** — `target/release/zeroclaw` or `lisa/release/<arch>/zeroclaw` -> `~/lisa/`
2. **Install config.toml** — `config.linux.toml` -> `~/.zeroclaw/config.toml`
3. **Install workspace** — `SOUL.md`, `AGENTS.md`, `USER.md`, skills -> `~/.zeroclaw/workspace/`
4. **Configure Target TV** — tv-control skill target TV location (N/A/local/remote) and IP
5. **gog (calendar) setup** — OAuth authentication, credential transfer (remote), lisa.env creation
6. **/etc/hosts** — Add Azure private endpoint via `sudo` if needed
7. **Start scripts** — `start-lisa.sh` (daemon), `lisa-agent.sh` (agent), PATH (.bashrc)
8. **Functional tests** — agent/daemon mode, per-skill command verification

## Differences from webOS TV

| Item | webOS TV (`deploy-target.sh`) | Ubuntu (`deploy-linux.sh`) |
|------|-------------------------------|----------------------------|
| User | `root` | Current user |
| Deploy path | `/home/root/lisa/` | `~/lisa/` |
| /etc/hosts | Read-Only -> bind mount workaround | Direct `sudo` edit |
| PATH | `.profile` hook | `.bashrc` addition |
| Binary | Pre-built ARM64 required | Auto-detect target arch + release binary or build suggestion |
| luna-send test | Included (webOS API) | Not included |
| Local install | Not possible (always remote) | Possible (run without IP) |

## Directory Structure

```
~/
├── lisa/
│   ├── zeroclaw              # Binary
│   ├── gog                   # Google Calendar CLI (if available)
│   ├── start-lisa.sh         # Start daemon mode
│   ├── lisa-agent.sh         # Start agent mode
│   └── lisa.env              # Environment variables (GOG_ACCOUNT, etc.)
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
```

## Usage

```bash
# After local install
~/lisa/start-lisa.sh      # daemon
~/lisa/lisa-agent.sh      # agent (interactive)
~/lisa/lisa-agent.sh hi!  # agent (one-shot)
~/lisa/zeroclaw status    # status check

# After remote install
ssh user@<IP> '~/lisa/start-lisa.sh'
ssh user@<IP> '~/lisa/lisa-agent.sh hi!'
```

## Cleanup

To remove all deployed files:

```bash
# Local cleanup
./lisa/scripts/clean-linux.sh

# Remote cleanup
./lisa/scripts/clean-linux.sh <IP>
```

Individual confirmations during cleanup:
1. **Remove deployed files** — `~/lisa/`, `~/.zeroclaw/`, `~/.config/gogcli/`, `/etc/hosts` entries, `.bashrc` PATH
2. **Remove local gog tokens** (remote mode only) — `~/.config/gogcli/keyring/`

## Redeploy

Run the same deploy script again. The existing config.toml is automatically backed up.

## Troubleshooting

| Symptom | Solution |
|---------|----------|
| SSH connection failure | `ssh-copy-id user@<IP>` |
| Azure connection failure | Check domain with `cat /etc/hosts` |
| Skill script blocked (`script-like files are blocked`) | Add `[skills] allow_scripts = true` to config.toml |
| `Invalid schema for function` error | Binary rebuild needed (upstream schema bug fix) |
| `temperature does not support 0.7` | agent CLI default (0.7) conflicts with model — add `-t 1.0` to `lisa-agent.sh` (rerun deploy script) |
| gateway `/health` no response | Check if daemon is running (`ps aux \| grep zeroclaw`), verify `[gateway]` port/bind in config |
| Telegram bot not responding | 1) Check `bot_token` value 2) Ensure user ID is in `allowed_users` 3) Verify running in daemon mode |
| Calendar `gog not found` | Rerun deploy script (transfers gog binary), check PATH with `which gog` |
| Calendar `no credentials` | Complete `gog auth credentials` + `gog auth add` locally, then redeploy |
| Calendar `keyring error` | Check `GOG_KEYRING_PASSWORD` in `lisa.env`, ensure `chmod 600 lisa.env` |
