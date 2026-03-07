# Lisa Guide

## Quick Start

### From Release Bundle (권장)

```bash
# 1. 릴리즈 다운로드
gh release download v0.2.0-lisa --repo rordd/lisa --pattern "*apple-darwin*"
tar xzf lisa-v0.2.0-lisa-aarch64-apple-darwin.tar.gz
cd lisa-v0.2.0-lisa-aarch64-apple-darwin

# 2. .env 설정
cp .env.example .env
# .env 편집: API 키, 텔레그램 토큰 등

# 3. 온보딩
./onboard.sh

# 4. 실행
source .env && zeroclaw daemon
```

### From Source

```bash
# 1. 클론
git clone https://github.com/rordd/lisa.git
cd lisa

# 2. .env 설정
cp lisa/profiles/.env.example .env
# .env 편집

# 3. 빌드 + 온보딩
./lisa/scripts/onboard.sh --build

# 4. 실행
source .env && zeroclaw daemon
```

## .env 설정

```bash
# 필수
ZEROCLAW_API_KEY=<your-api-key>
ZEROCLAW_PROVIDER=gemini          # gemini | openai | azure
ZEROCLAW_MODEL=gemini-2.5-flash

# 텔레그램 (선택)
TELEGRAM_BOT_TOKEN=<bot-token>
TELEGRAM_ALLOWED_USERS=<user-id>
TELEGRAM_MENTION_ONLY=true

# Google Calendar (선택)
GOG_ACCOUNT=you@gmail.com
GOG_KEYRING_PASSWORD=<password>
GOG_KEYRING_BACKEND=file

# Azure OpenAI (선택)
# AZURE_OPENAI_BASE_URL=https://your-resource.openai.azure.com/openai/deployments/your-model
# AZURE_OPENAI_API_KEY=<key>
# AZURE_OPENAI_AUTH_HEADER=api-key
```

## 개인 파일

온보딩 후 `~/.zeroclaw/workspace/USER.md`를 편집하세요:

```markdown
# USER.md
- **이름:** (닉네임)
- **호칭:** 삼촌
- **타임존:** Asia/Seoul

## 캘린더
- **기본(primary):** you@gmail.com
- **추가 캘린더:** calendar-id@group.calendar.google.com

## 날씨
- 위치: 서울 강서구 (latitude=37.55, longitude=126.85)
```

**백업 필요한 파일 (릴리즈 업그레이드 시):**
- `.env` — API 키, 토큰
- `~/.zeroclaw/workspace/USER.md` — 개인정보

## onboard.sh 사용법

```bash
# 풀 온보딩 (첫 설치)
onboard.sh

# 빌드 + 풀 온보딩
onboard.sh --build

# 바이너리만 교체
onboard.sh --binary
onboard.sh --build --binary

# 스킬만 교체
onboard.sh --skills

# 설정만 교체 (config.toml + .env + SOUL.md + AGENTS.md)
onboard.sh --config

# 타겟 배포
onboard.sh --target 192.168.1.50
onboard.sh --build --target 192.168.1.50
onboard.sh --target 192.168.1.50 --skills
onboard.sh --target 192.168.1.50 --config
```

## release.sh 사용법

```bash
# macOS 번들만
release.sh --version v0.2.0-lisa

# 전 플랫폼 (macOS + Linux ARM64 + Linux x86_64)
release.sh --version v0.2.0-lisa --target all

# 빌드 스킵 (이미 빌드됨)
release.sh --version v0.2.0-lisa --target all --skip-build

# 드라이런 (업로드 안 함)
release.sh --version v0.2.0-lisa --dry-run
```

### 릴리즈 번들 내용물

```
lisa-v0.2.0-lisa-<platform>/
├── onboard.sh          # 온보딩 스크립트
├── zeroclaw            # 바이너리
├── .env.example        # 시크릿 템플릿
├── config/
│   └── config.default.toml
├── profiles/
│   └── lisa/
│       ├── SOUL.md
│       ├── AGENTS.md
│       ├── USER.md.example
│       └── skills/
└── bin/                # 의존 바이너리 (gog 등)
```

## Google Calendar 설정

```bash
# gog CLI 설치 (macOS)
brew install steipete/tap/gogcli

# 인증 (최초 1회, 브라우저 필요)
GOG_KEYRING_BACKEND=file GOG_KEYRING_PASSWORD=<pw> gog auth add you@gmail.com --services calendar --manual

# 테스트
GOG_KEYRING_BACKEND=file GOG_KEYRING_PASSWORD=<pw> gog calendar list --from today --to tomorrow
```

## 타겟 보드 배포

```bash
# 1. 릴리즈 번들 사용 (권장)
gh release download v0.2.0-lisa --repo rordd/lisa --pattern "*linux-gnu*"
tar xzf lisa-v0.2.0-lisa-aarch64-unknown-linux-gnu.tar.gz
cd lisa-v0.2.0-lisa-aarch64-unknown-linux-gnu
cp .env.example .env && vi .env
./onboard.sh --target <board-ip>

# 2. 소스에서 크로스빌드
cd ~/project/lisa
./lisa/scripts/onboard.sh --build --target <board-ip>
```

### 요구사항
- Docker Desktop (크로스 빌드 시)
- `cross` CLI (`cargo install cross`)
- SSH 키 기반 접속 설정

## 플랫폼별 번들

| 플랫폼 | 파일명 | 용도 |
|---|---|---|
| macOS ARM64 | `aarch64-apple-darwin` | 맥 (M-시리즈) |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | 타겟 보드 (webOS 등) |
| Linux x86_64 | `x86_64-unknown-linux-gnu` | 리눅스 서버 |
