# Lisa Setup Guide

Lisa is a ZeroClaw-based on-device AI home agent.

## Prerequisites

### Rust Toolchain
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Build Dependencies

**Ubuntu/Debian:**
```bash
sudo apt update && sudo apt install -y build-essential pkg-config libssl-dev cmake
```

**macOS:**
```bash
xcode-select --install
```

## Installation

### 1. Clone & Build
```bash
git clone https://github.com/rordd/lisa.git ~/project/lisa
cd ~/project/lisa
cargo install --path .
```

### 2. Environment Setup
```bash
cp lisa/profiles/.env.example .env
```

Edit the `.env` file and enter your settings:

#### Gemini
```bash
ZEROCLAW_API_KEY=AIza...
ZEROCLAW_PROVIDER=gemini
ZEROCLAW_MODEL=gemini-2.5-flash
```

#### Azure OpenAI
```bash
ZEROCLAW_API_KEY=<Azure API Key>
ZEROCLAW_PROVIDER=custom:azure
ZEROCLAW_MODEL=gpt-4o
AZURE_OPENAI_BASE_URL=https://<resource>.openai.azure.com/openai/deployments/<deployment>/chat/completions?api-version=2024-02-01
AZURE_OPENAI_API_KEY=<Azure API Key>
```

#### OpenAI
```bash
ZEROCLAW_API_KEY=sk-...
ZEROCLAW_PROVIDER=openai
ZEROCLAW_MODEL=gpt-4o
```

#### Ollama (local)
```bash
ZEROCLAW_API_KEY=http://localhost:11434
ZEROCLAW_PROVIDER=ollama
ZEROCLAW_MODEL=llama3.2
```

#### Telegram (optional)
```bash
TELEGRAM_BOT_TOKEN=<token from BotFather>
TELEGRAM_ALLOWED_USERS=<comma-separated user_ids>
TELEGRAM_MENTION_ONLY=true
```

### 3. Create Config Files from Templates

Some configuration files contain sensitive data (API keys, tokens) and are not tracked by Git.
Only `.example` templates are provided. Copy them and fill in your values before running the setup script:

```bash
# Required — already done in step 2 above
cp lisa/profiles/.env.example .env

# Optional — only needed for target deployment (see deploy guides)
cp lisa/config/config.arm64.toml.example lisa/config/config.arm64.toml    # webOS TV target
cp lisa/config/config.linux.toml.example lisa/config/config.linux.toml    # Linux target
cp lisa/profiles/lisa/lisa.env.example lisa/profiles/lisa/lisa.env         # Target env vars
cp lisa/profiles/lisa/USER.md.example lisa/profiles/lisa/USER.md          # User info
```

> **Note:** If only `.example` files exist and no actual file is present, you **must** create the actual file by copying from the `.example` template and editing it with your values. The application will not work with `.example` files alone.

### 4. Run Setup
```bash
./lisa/scripts/setup-lisa.sh
```

What this script does:
- Creates `~/.zeroclaw/` directory
- Copies default config (`config/config.default.toml`) to `config.toml`
- Injects Telegram settings (if bot token is in `.env`)
- Injects Azure OpenAI profile (if configured in `.env`)
- Copies personality files (`SOUL.md`, `AGENTS.md`) to workspace

### 5. User Information Setup

```bash
cp lisa/profiles/lisa/USER.md.example ~/.zeroclaw/workspace/USER.md
```

Edit `USER.md` with your personal information (name, timezone, calendar, etc.).

### 6. Run
```bash
source .env && zeroclaw daemon
```

## Verification

```bash
# Status check
zeroclaw status

# CLI chat
source .env && zeroclaw chat "Hello Lisa!"

# Web dashboard
# http://localhost:42617
```

## Troubleshooting

| Symptom | Solution |
|---------|----------|
| Build failure related to `libssl-dev` | `sudo apt install libssl-dev` |
| Build failure related to `cmake` | `sudo apt install cmake` |
| Port conflict | Change `[gateway]` port in `config.toml` |
| Azure auth failure | Check `auth_header = "api-key"`, ensure `?api-version=` is in base_url |
| Telegram not connecting | Check that your user_id is in `allowed_users` |

## Related Guides

- [Google Calendar OAuth Setup (gogcli)](gogcli-oauth-setup-guide.md) — Google Cloud Console OAuth client setup for the calendar skill
- [webOS TV Deploy Guide](deploy-target-guide.md) — Deploy to webOS TV (ARM64)
- [Linux Deploy Guide](deploy-linux-guide.md) — Deploy to Ubuntu Linux

## Project Structure

```
lisa/
├── config/
│   ├── config.default.toml       # Default config (local/general, setup-lisa.sh)
│   ├── config.linux.toml         # Linux (Ubuntu) target (deploy-linux.sh)
│   ├── config.arm64.toml         # webOS TV (ARM64) target (deploy-target.sh)
│   ├── config.arm64.toml.example # ARM64 config template (no sensitive data)
│   └── config.linux.toml.example # Linux config template (no sensitive data)
├── profiles/
│   ├── .env.example              # Secret template (not tracked by Git)
│   └── lisa/
│       ├── SOUL.md               # Lisa's personality
│       ├── AGENTS.md             # Agent rules
│       ├── USER.md.example       # User info template
│       ├── lisa.env.example      # Target env vars template
│       └── skills/
│           ├── calendar/SKILL.md # Calendar skill
│           ├── weather/SKILL.md  # Weather skill
│           └── tv-control/       # TV control skill
├── scripts/
│   ├── setup-lisa.sh             # Local setup script
│   ├── deploy-target.sh          # webOS TV deploy (Linux/macOS)
│   ├── deploy-target.ps1         # webOS TV deploy (Windows)
│   ├── deploy-target.bat         # Windows ExecutionPolicy bypass wrapper
│   ├── deploy-linux.sh           # Linux (Ubuntu) deploy
│   ├── clean-target.sh           # webOS TV cleanup
│   ├── clean-linux.sh            # Linux (Ubuntu) cleanup
│   └── issues.sh                 # Issue management CLI
├── docs/
│   ├── setup-guide.md            # This document
│   ├── deploy-target-guide.md    # webOS TV deploy guide
│   ├── deploy-linux-guide.md     # Linux deploy guide
│   └── gogcli-oauth-setup-guide.md # Google Calendar OAuth setup guide
├── release/
│   ├── arm64/
│   │   ├── zeroclaw              # ARM64 binary
│   │   └── gog                   # ARM64 gog (calendar CLI)
│   └── x86_64/
│       ├── zeroclaw              # x86_64 binary
│       └── gog                   # x86_64 gog (calendar CLI)
└── history/                      # Change history
```

- **Default config** is managed in Git as `lisa/config/config.default.toml`
- **Target configs** are managed per-target as `lisa/config/config.<target>.toml`
- **Secrets** are managed via `.env` only — never committed to Git
- **User info** is managed individually via `USER.md`
