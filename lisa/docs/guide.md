# Lisa Guide

## Quick Start

### From Release Bundle (Recommended)

```bash
# 1. Download release
gh release download v0.2.0-lisa --repo rordd/lisa --pattern "*apple-darwin*"
tar xzf lisa-v0.2.0-lisa-aarch64-apple-darwin.tar.gz
cd lisa-v0.2.0-lisa-aarch64-apple-darwin

# 2. Configure secrets
cp .env.example .env
# Edit .env: API keys, Telegram token, etc.

# 3. Onboard
./onboard.sh

# 4. Run
source .env && zeroclaw daemon
```

### From Source

```bash
# 1. Clone
git clone https://github.com/rordd/lisa.git
cd lisa

# 2. Configure secrets
cp lisa/profiles/.env.example .env
# Edit .env

# 3. Build + onboard
./lisa/scripts/onboard.sh --build

# 4. Run
source .env && zeroclaw daemon
```

## Config Structure

```
config.default.toml (repo)  ← App settings, no secrets (safe to commit)
.env (local)                ← Secrets & personal info (gitignored)
USER.md (local)             ← User profile (local only)
```

- `config.default.toml` contains no personal data — safe to commit
- Telegram token, API keys, etc. are injected via `.env` environment variables
- Local dev: `~/.zeroclaw/config.toml` symlinks to `config.default.toml` (auto-set by onboard.sh)
- Edit `config.default.toml` directly — changes apply on daemon restart

## .env Configuration

```bash
# Required
ZEROCLAW_API_KEY=<your-api-key>
ZEROCLAW_PROVIDER=gemini          # gemini | openai | azure
ZEROCLAW_MODEL=gemini-2.5-flash

# Telegram (optional)
TELEGRAM_BOT_TOKEN=<bot-token>
TELEGRAM_ALLOWED_USERS=<user-id>  # Comma-separated
TELEGRAM_MENTION_ONLY=true

# Google Calendar (optional)
GOG_ACCOUNT=you@gmail.com
GOG_KEYRING_PASSWORD=<password>
GOG_KEYRING_BACKEND=file

# Azure OpenAI (optional)
# AZURE_OPENAI_BASE_URL=https://your-resource.openai.azure.com/openai/deployments/your-model
# AZURE_OPENAI_API_KEY=<key>
# AZURE_OPENAI_AUTH_HEADER=api-key
```

## Personal Files

Edit `~/.zeroclaw/workspace/USER.md` after onboarding:

```markdown
# USER.md
- **Name:** (nickname)
- **Timezone:** Asia/Seoul

## Calendar
- **Primary:** you@gmail.com
- **Additional:** calendar-id@group.calendar.google.com

## Weather
- Location: Seoul Gangseo-gu (latitude=37.55, longitude=126.85)
```

List your calendar IDs:
```bash
GOG_KEYRING_BACKEND=file GOG_KEYRING_PASSWORD=<pw> gog calendar calendars -a you@gmail.com
```

**Backup these files when upgrading:**
- `.env` — API keys, tokens
- `~/.zeroclaw/workspace/USER.md` — personal info

## onboard.sh

```bash
onboard.sh                        # Full onboard (first install)
onboard.sh --build                # Build + full onboard
onboard.sh --binary               # Binary only (quick swap)
onboard.sh --build --binary       # Build + binary only
onboard.sh --skills               # Skills only
onboard.sh --config               # Config only (config.toml + profile)
onboard.sh --target 192.168.1.50  # Deploy to target
onboard.sh --build --target IP    # Cross-build + deploy
onboard.sh --target IP --skills   # Skills only to target
onboard.sh --target IP --config   # Config only to target
```

## Dev Workflow

```bash
# After code change: build + replace binary
onboard.sh --build --binary

# After skill change: replace skills
onboard.sh --skills

# After config change: restart daemon (symlink, no copy needed)
pkill -f "zeroclaw daemon" && source .env && zeroclaw daemon
```

## release.sh

```bash
release.sh --version v0.2.0-lisa                # All platforms (default)
release.sh --version v0.2.0-lisa --target host  # Host only
release.sh --version v0.2.0-lisa --skip-build   # Skip build
release.sh --version v0.2.0-lisa --dry-run      # Dry run (no upload)
```

### Release Bundle Contents

```
lisa-v0.2.0-lisa-<platform>/
├── onboard.sh
├── zeroclaw
├── .env.example
├── config/
│   └── config.default.toml
├── docs/
│   ├── guide.md
│   ├── guide-ko.md
│   └── gogcli-oauth-setup-guide.md
├── profiles/
│   └── lisa/
│       ├── SOUL.md
│       ├── AGENTS.md
│       ├── USER.md.example
│       └── skills/
└── bin/                # Dependency binaries (gog, etc.)
```

## Google Calendar Setup

```bash
# Install gog CLI
# macOS:
brew install steipete/tap/gogcli
# Linux (included in release bundle, or manual):
gh release download v0.11.0 --repo steipete/gogcli --pattern "*linux_amd64*"  # x86_64
gh release download v0.11.0 --repo steipete/gogcli --pattern "*linux_arm64*"  # ARM64
tar xzf gogcli_*.tar.gz && sudo mv gog /usr/local/bin/

# Authenticate (once, requires browser)
GOG_KEYRING_BACKEND=file GOG_KEYRING_PASSWORD=<pw> gog auth add you@gmail.com --services calendar --manual
# No browser? Open URL on another device, copy redirect URL and paste

# Test
GOG_KEYRING_BACKEND=file GOG_KEYRING_PASSWORD=<pw> gog calendar list --from today --to tomorrow

# List all calendar IDs (for USER.md setup)
GOG_KEYRING_BACKEND=file GOG_KEYRING_PASSWORD=<pw> gog calendar calendars -a you@gmail.com
```

## Target Board Deployment

```bash
# 1. Using release bundle (recommended)
gh release download v0.2.0-lisa --repo rordd/lisa --pattern "*linux-gnu*"
tar xzf lisa-v0.2.0-lisa-aarch64-unknown-linux-gnu.tar.gz
cd lisa-v0.2.0-lisa-aarch64-unknown-linux-gnu
cp .env.example .env && vi .env
./onboard.sh --target <board-ip>

# 2. Cross-build from source
cd ~/project/lisa
./lisa/scripts/onboard.sh --build --target <board-ip>
```

### Requirements
- Docker Desktop (for cross-build)
- `cross` CLI (`cargo install cross`)
- SSH key-based access

## Platform Bundles

| Platform | Filename | Use |
|---|---|---|
| macOS ARM64 | `aarch64-apple-darwin` | Mac (M-series) |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | Target board (webOS, etc.) |
| Linux x86_64 | `x86_64-unknown-linux-gnu` | Linux server |
