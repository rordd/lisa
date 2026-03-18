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
source ~/.zeroclaw/.env && zeroclaw daemon   # Background daemon (Telegram, etc.)
source ~/.zeroclaw/.env && zeroclaw agent    # Interactive CLI mode
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
source ~/.zeroclaw/.env && zeroclaw daemon   # Background daemon (Telegram, etc.)
source ~/.zeroclaw/.env && zeroclaw agent    # Interactive CLI mode
```

## Config Structure

```
config.default.toml (repo)  ← App settings, no secrets (safe to commit)
.env (local)                ← Secrets & personal info (gitignored)
USER.md (local)             ← User profile (local only)
```

After onboarding, files are installed to `~/.zeroclaw/`:
```
~/.zeroclaw/
├── config.toml   ← Copied from config.default.toml
├── .env          ← Copied from source .env
└── workspace/
    ├── USER.md
    ├── SOUL.md
    ├── AGENTS.md
    └── skills/
        ├── weather/
        ├── calendar/
        └── tv-control/
            ├── scripts/
            │   └── mock/       ← Installed only on non-webOS environments
            │       └── luna-send
            └── ...
```

- `config.default.toml` contains no personal data — safe to commit
- Telegram token, API keys, etc. are injected via `.env` environment variables
- `onboard.sh` copies config and `.env` to `~/.zeroclaw/` (repo can be removed after install)
- After editing `config.default.toml` or `.env`, re-run `onboard.sh --config` to apply

## .env Configuration

Pick one LLM provider section — either Gemini or Azure OpenAI.

```bash
# --- Google Gemini (default) ---
export ZEROCLAW_API_KEY=<your-api-key>
export ZEROCLAW_PROVIDER=gemini
export ZEROCLAW_MODEL=gemini-2.5-flash

# --- Azure OpenAI (alternative) ---
# Uncomment and fill these, comment out the Gemini section above.
# export ZEROCLAW_PROVIDER=custom:https://<resource>.openai.azure.com/openai/v1
# export ZEROCLAW_MODEL=gpt-5-mini
# export ZEROCLAW_API_KEY=<azure-api-key>
# export ZEROCLAW_TEMPERATURE=1              # Required for reasoning models (gpt-5-mini, o-series)
# export AZURE_PRIVATE_ENDPOINT=<private-ip>  # If using private endpoint
# export ZEROCLAW_CUSTOM_REASONING_EFFORT=minimal  # Reasoning effort: minimal, low, medium(default), high

# Telegram (optional — not available behind company firewalls)
# export TELEGRAM_BOT_TOKEN=<bot-token>
# export TELEGRAM_ALLOWED_USERS=<user-id>  # Comma-separated
# export TELEGRAM_MENTION_ONLY=true

# Google Calendar (optional)
export GOG_ACCOUNT=you@gmail.com
export GOG_KEYRING_PASSWORD=<password>
export GOG_KEYRING_BACKEND=file
```

### Reasoning Level (Azure OpenAI)

`ZEROCLAW_CUSTOM_REASONING_EFFORT` controls how much reasoning the model performs before responding.
Applies to reasoning models like `gpt-5-mini` and o-series.

| Level | Reasoning tokens | Speed | Recommended use |
|---|---|---|---|
| `minimal` | 0 | Fastest | Simple tasks — weather, greetings, quick lookups |
| `low` | ~64 | Fast | Light reasoning — summaries, basic Q&A |
| `medium` | ~192 | Default | General use (default when not set) |
| `high` | Full | Slowest | Complex analysis, multi-step reasoning |

> **Tip:** For everyday home assistant use (weather, schedules, device control),
> `minimal` is recommended — it skips reasoning entirely and responds significantly faster.

Can also be set in `config.default.toml`:
```toml
[provider]
reasoning_level = "minimal"
```

> **Note:** Currently supported for `custom:` providers only (e.g. Azure OpenAI).
> Built-in provider presets (openai, gemini, etc.) do not use this setting.

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
- `~/.zeroclaw/.env` — API keys, tokens
- `~/.zeroclaw/workspace/USER.md` — personal info

## onboard.sh

```bash
onboard.sh                              # Full onboard (first install)
onboard.sh --build                      # Build + full onboard
onboard.sh --binary                     # Binary only (quick swap)
onboard.sh --build --binary             # Build + binary only
onboard.sh --skills                     # Skills only
onboard.sh --config                     # Config only (config.toml + profile)
onboard.sh --clear                      # Remove all installed files
onboard.sh --target 192.168.1.50        # Deploy to target
onboard.sh --target IP --build          # Cross-build + deploy to target
onboard.sh --target IP --binary         # Binary only (quick swap) to target
onboard.sh --target IP --build --binary # Build + binary only to target
onboard.sh --target IP --skills         # Skills only to target
onboard.sh --target IP --config         # Config only to target
onboard.sh --target IP --clear          # Remove all from target
```

### Binary replacement behavior

`--binary`, `--skills`, and `--config` automatically handle running processes:
- All zeroclaw processes (daemon, agent) are stopped before replacing the binary
- If the **daemon** was running before replacement, it is restarted automatically
- If only agent (or nothing) was running, no restart occurs

### Full onboard tests

Full onboard (`onboard.sh` without scope flags) runs automatic tests after installation.
All skill tests go through zeroclaw (not direct API calls), so they verify the full pipeline.

| Test | Method | Pass criteria |
|---|---|---|
| agent | Send "안녕~" via zeroclaw | Exit 0 |
| weather | Ask zeroclaw for weather | Agent OK + valid response |
| calendar | Ask zeroclaw for schedule | Agent OK + gog installed + valid response |
| tv-control | Ask zeroclaw for foreground app | Agent OK + luna-send available (real or mock) |

If the agent test fails (no LLM connection), skill tests are automatically skipped.

> **Mock luna-send:** On non-webOS environments (regular Linux/macOS), `luna-send` is not available.
> During skill installation, the mock script (`skills/tv-control/scripts/mock/luna-send`) is
> symlinked to `~/.local/bin/luna-send`. This allows tv-control to work in daemon, agent, and
> tests without modification.
> On webOS targets, the real `luna-send` exists, so the mock is not installed.

### Clear (uninstall)

`--clear` removes everything installed by onboard.sh:
- Stops the zeroclaw daemon
- Removes the binary from `~/.local/bin/`
- Removes dependency binaries (`gog`, etc.) from `~/.local/bin/`
- Removes mock `luna-send` symlink from `~/.local/bin/` (only if it is a symlink)
- Removes `~/.zeroclaw/` (config + workspace)
- Removes Azure private endpoint from `/etc/hosts` (unmounts bind if used)

Requires interactive confirmation before proceeding.

## Dev Workflow

> **Note:** After any source code change, you must run a release build (`--build`) before installing.
> `onboard.sh --binary` alone only copies the existing release binary — it does not rebuild.

```bash
# After code change: release build + replace binary (--build is required)
onboard.sh --build --binary

# After skill change: replace skills
onboard.sh --skills

# After config/.env change: re-apply and restart
onboard.sh --config
pkill -f "zeroclaw daemon" && source ~/.zeroclaw/.env && zeroclaw daemon
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

### gog CLI Installation

`onboard.sh` automatically installs `gog` if it is not found.
It searches for a binary in this order:
1. `bin/gog-linux-{arm64,amd64}` — arch-specific binary in `lisa/bin/` (local build)
2. `bin/gog` — generic binary (release bundle)
3. GitHub release download (fallback)

Installs to `~/.local/bin/gog` (local) or deploy directory (target).

#### Build from source

Build gog from the [gogcli](https://github.com/steipete/gogcli) source and place the binaries in `lisa/bin/` so that `onboard.sh` can find them:

```bash
# Clone gogcli anywhere (location is up to you)
git clone https://github.com/steipete/gogcli.git
cd gogcli

# Static build for both architectures
mkdir -p /path/to/lisa/lisa/bin
CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -ldflags "-s -w" -o /path/to/lisa/lisa/bin/gog-linux-amd64 ./cmd/gog
CGO_ENABLED=0 GOOS=linux GOARCH=arm64 go build -ldflags "-s -w" -o /path/to/lisa/lisa/bin/gog-linux-arm64 ./cmd/gog
```

> `lisa/bin/` is gitignored — built binaries are local only.

#### Manual install from release bundle

```bash
gh release download --repo rordd/lisa --pattern "*apple-darwin*"       # macOS
gh release download --repo rordd/lisa --pattern "*x86_64*linux-gnu*"   # Linux x86_64
gh release download --repo rordd/lisa --pattern "*aarch64*linux-gnu*"  # Linux ARM64
tar xzf lisa-*.tar.gz
cp lisa-*/bin/gog ~/.local/bin/gog && chmod +x ~/.local/bin/gog
```

### Authentication & Test

```bash
# Authenticate (once, requires browser)
GOG_KEYRING_BACKEND=file GOG_KEYRING_PASSWORD=<pw> gog auth add you@gmail.com --services calendar --manual
# No browser? Open URL on another device, copy redirect URL and paste

# Test
GOG_KEYRING_BACKEND=file GOG_KEYRING_PASSWORD=<pw> gog calendar events primary --from today --to tomorrow

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

### Target install layout

Same directory structure as local — config and secrets in `~/.zeroclaw/`, binary in the deploy directory:

```
/home/root/lisa/            ← Binary + deps (deploy dir)
├── zeroclaw
└── gog

/home/root/.zeroclaw/       ← Config + secrets + workspace (same as local ~/.zeroclaw/)
├── config.toml
├── .env
└── workspace/
    ├── USER.md
    ├── SOUL.md
    ├── AGENTS.md
    └── skills/
```

### Running on target

```bash
# Via SSH
ssh root@<board-ip> 'export PATH=/home/root/lisa:$PATH && source ~/.zeroclaw/.env && zeroclaw daemon'
ssh root@<board-ip> 'export PATH=/home/root/lisa:$PATH && source ~/.zeroclaw/.env && zeroclaw agent'
```

### Requirements
- SSH key-based access
- Cross-build toolchain (one of the following):
  - **Option A**: `cross` CLI + Docker (`cargo install cross`)
  - **Option B**: Native musl toolchain (`sudo apt install gcc-aarch64-linux-gnu musl-tools` + `rustup target add aarch64-unknown-linux-musl`)

## Platform Bundles

| Platform | Filename | Use |
|---|---|---|
| macOS ARM64 | `aarch64-apple-darwin` | Mac (M-series) |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | Target board (webOS, etc.) |
| Linux x86_64 | `x86_64-unknown-linux-gnu` | Linux server |
