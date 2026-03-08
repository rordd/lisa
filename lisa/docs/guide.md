# Lisa Guide

## Quick Start / 빠른 시작

### From Release Bundle (Recommended / 권장)

```bash
# 1. Download release / 릴리즈 다운로드
gh release download v0.2.0-lisa --repo rordd/lisa --pattern "*apple-darwin*"
tar xzf lisa-v0.2.0-lisa-aarch64-apple-darwin.tar.gz
cd lisa-v0.2.0-lisa-aarch64-apple-darwin

# 2. Configure secrets / 시크릿 설정
cp .env.example .env
# Edit .env: API keys, Telegram token, etc.
# .env 편집: API 키, 텔레그램 토큰 등

# 3. Onboard / 온보딩
./onboard.sh

# 4. Run / 실행
source .env && zeroclaw daemon
```

### From Source / 소스에서

```bash
# 1. Clone / 클론
git clone https://github.com/rordd/lisa.git
cd lisa

# 2. Configure secrets / 시크릿 설정
cp lisa/profiles/.env.example .env
# Edit .env / .env 편집

# 3. Build + onboard / 빌드 + 온보딩
./lisa/scripts/onboard.sh --build

# 4. Run / 실행
source .env && zeroclaw daemon
```

## Config Structure / 설정 구조

```
config.default.toml (repo)  ← App settings, no secrets / 앱 설정, 시크릿 없음
.env (local)                ← Secrets & personal info / 시크릿 & 개인정보
USER.md (local)             ← User profile / 사용자 프로필
```

- `config.default.toml` contains no personal data — safe to commit
  `config.default.toml`에 개인정보 없음 — 커밋 안전
- Telegram token, API keys, etc. are injected via `.env` environment variables
  텔레그램 토큰, API 키 등은 `.env` 환경변수로 주입
- Local dev: `~/.zeroclaw/config.toml` symlinks to `config.default.toml` (auto-set by onboard.sh)
  로컬 개발: `~/.zeroclaw/config.toml`은 `config.default.toml`로의 심링크 (onboard.sh가 자동 설정)
- Edit `config.default.toml` directly — changes apply on daemon restart
  `config.default.toml` 직접 편집 — daemon restart 시 반영

## .env Configuration / .env 설정

```bash
# Required / 필수
ZEROCLAW_API_KEY=<your-api-key>
ZEROCLAW_PROVIDER=gemini          # gemini | openai | azure
ZEROCLAW_MODEL=gemini-2.5-flash

# Telegram (optional / 선택)
TELEGRAM_BOT_TOKEN=<bot-token>
TELEGRAM_ALLOWED_USERS=<user-id>  # Comma-separated / 쉼표 구분
TELEGRAM_MENTION_ONLY=true

# Google Calendar (optional / 선택)
GOG_ACCOUNT=you@gmail.com
GOG_KEYRING_PASSWORD=<password>
GOG_KEYRING_BACKEND=file

# Azure OpenAI (optional / 선택)
# AZURE_OPENAI_BASE_URL=https://your-resource.openai.azure.com/openai/deployments/your-model
# AZURE_OPENAI_API_KEY=<key>
# AZURE_OPENAI_AUTH_HEADER=api-key
```

## Personal Files / 개인 파일

Edit `~/.zeroclaw/workspace/USER.md` after onboarding:
온보딩 후 `~/.zeroclaw/workspace/USER.md`를 편집하세요:

```markdown
# USER.md
- **Name / 이름:** (nickname / 닉네임)
- **Timezone / 타임존:** Asia/Seoul

## Calendar / 캘린더
- **Primary:** you@gmail.com
- **Additional / 추가:** calendar-id@group.calendar.google.com

## Weather / 날씨
- Location / 위치: Seoul Gangseo-gu (latitude=37.55, longitude=126.85)
```

List your calendar IDs / 캘린더 ID 확인:
```bash
GOG_KEYRING_BACKEND=file GOG_KEYRING_PASSWORD=<pw> gog calendar calendars -a you@gmail.com
```

**Backup these files when upgrading / 업그레이드 시 백업 필요:**
- `.env` — API keys, tokens / API 키, 토큰
- `~/.zeroclaw/workspace/USER.md` — personal info / 개인정보

## onboard.sh

```bash
# Full onboard (first install) / 풀 온보딩 (첫 설치)
onboard.sh

# Build + full onboard / 빌드 + 풀 온보딩
onboard.sh --build

# Binary only (quick swap) / 바이너리만 교체
onboard.sh --binary
onboard.sh --build --binary

# Skills only / 스킬만 교체
onboard.sh --skills

# Config only (config.toml + profile) / 설정만 교체
onboard.sh --config

# Deploy to target / 타겟 배포
onboard.sh --target 192.168.1.50
onboard.sh --build --target 192.168.1.50
onboard.sh --target 192.168.1.50 --skills
onboard.sh --target 192.168.1.50 --config
```

## Dev Workflow / 개발 워크플로우

```bash
# After code change: build + replace binary
# 코드 수정 후: 빌드 + 바이너리 교체
onboard.sh --build --binary

# After skill change: replace skills
# 스킬 수정 후: 스킬 교체
onboard.sh --skills

# After config change: restart daemon (symlink, no copy needed)
# config 수정 후: daemon restart (심링크라 복사 불필요)
pkill -f "zeroclaw daemon" && source .env && zeroclaw daemon
```

## release.sh

```bash
# All platforms (default: macOS + Linux ARM64 + Linux x86_64)
# 전 플랫폼 (기본: macOS + Linux ARM64 + Linux x86_64)
release.sh --version v0.2.0-lisa

# Host only / 호스트만
release.sh --version v0.2.0-lisa --target host

# Skip build (already built) / 빌드 스킵
release.sh --version v0.2.0-lisa --skip-build

# Dry run (no upload) / 드라이런
release.sh --version v0.2.0-lisa --dry-run
```

### Release Bundle Contents / 릴리즈 번들 내용물

```
lisa-v0.2.0-lisa-<platform>/
├── onboard.sh          # Onboard script / 온보딩 스크립트
├── zeroclaw            # Binary / 바이너리
├── .env.example        # Secret template / 시크릿 템플릿
├── config/
│   └── config.default.toml
├── docs/               # Guides / 가이드 문서
│   ├── guide.md
│   └── gogcli-oauth-setup-guide.md
├── profiles/
│   └── lisa/
│       ├── SOUL.md
│       ├── AGENTS.md
│       ├── USER.md.example
│       └── skills/
└── bin/                # Dependency binaries (gog, etc.)
                        # 의존 바이너리 (gog 등)
```

## Google Calendar Setup / 구글 캘린더 설정

```bash
# Install gog CLI / gog CLI 설치
# macOS:
brew install steipete/tap/gogcli
# Linux (included in release bundle, or manual):
# Linux (릴리즈 번들에 포함됨, 또는 수동):
gh release download v0.11.0 --repo steipete/gogcli --pattern "*linux_amd64*"  # x86_64
gh release download v0.11.0 --repo steipete/gogcli --pattern "*linux_arm64*"  # ARM64
tar xzf gogcli_*.tar.gz && sudo mv gog /usr/local/bin/

# Authenticate (once, requires browser)
# 인증 (최초 1회, 브라우저 필요)
GOG_KEYRING_BACKEND=file GOG_KEYRING_PASSWORD=<pw> gog auth add you@gmail.com --services calendar --manual
# No browser? Open URL on another device, copy redirect URL and paste
# 브라우저 없는 환경: 다른 기기에서 URL 열고 리다이렉트 URL 복사 → 붙여넣기

# Test / 테스트
GOG_KEYRING_BACKEND=file GOG_KEYRING_PASSWORD=<pw> gog calendar list --from today --to tomorrow
```

## Target Board Deployment / 타겟 보드 배포

```bash
# 1. Using release bundle (recommended) / 릴리즈 번들 사용 (권장)
gh release download v0.2.0-lisa --repo rordd/lisa --pattern "*linux-gnu*"
tar xzf lisa-v0.2.0-lisa-aarch64-unknown-linux-gnu.tar.gz
cd lisa-v0.2.0-lisa-aarch64-unknown-linux-gnu
cp .env.example .env && vi .env
./onboard.sh --target <board-ip>

# 2. Cross-build from source / 소스에서 크로스빌드
cd ~/project/lisa
./lisa/scripts/onboard.sh --build --target <board-ip>
```

### Requirements / 요구사항
- Docker Desktop (for cross-build / 크로스 빌드 시)
- `cross` CLI (`cargo install cross`)
- SSH key-based access / SSH 키 기반 접속 설정

## Platform Bundles / 플랫폼별 번들

| Platform / 플랫폼 | Filename / 파일명 | Use / 용도 |
|---|---|---|
| macOS ARM64 | `aarch64-apple-darwin` | Mac (M-series) |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | Target board (webOS, etc.) / 타겟 보드 |
| Linux x86_64 | `x86_64-unknown-linux-gnu` | Linux server / 리눅스 서버 |
