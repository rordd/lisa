# Lisa 가이드

## 빠른 시작

### 릴리즈 번들 (권장)

```bash
# 1. 릴리즈 다운로드
gh release download v0.2.0-lisa --repo rordd/lisa --pattern "*apple-darwin*"
tar xzf lisa-v0.2.0-lisa-aarch64-apple-darwin.tar.gz
cd lisa-v0.2.0-lisa-aarch64-apple-darwin

# 2. 시크릿 설정
cp .env.example .env
# .env 편집: API 키, 텔레그램 토큰 등

# 3. 온보딩
./onboard.sh

# 4. 실행
source ~/.zeroclaw/.env && zeroclaw daemon   # 백그라운드 데몬 (텔레그램 등)
source ~/.zeroclaw/.env && zeroclaw agent    # 대화형 CLI 모드
```

### 소스에서

```bash
# 1. 클론
git clone https://github.com/rordd/lisa.git
cd lisa

# 2. 시크릿 설정
cp lisa/profiles/.env.example .env
# .env 편집

# 3. 빌드 + 온보딩
./lisa/scripts/onboard.sh --build

# 4. 실행
source ~/.zeroclaw/.env && zeroclaw daemon   # 백그라운드 데몬 (텔레그램 등)
source ~/.zeroclaw/.env && zeroclaw agent    # 대화형 CLI 모드
```

## 설정 구조

```
config.default.toml (레포)  ← 앱 설정, 시크릿 없음 (커밋 안전)
.env (로컬)                 ← 시크릿 & 개인정보 (gitignore)
USER.md (로컬)              ← 사용자 프로필 (로컬 전용)
```

온보딩 후 `~/.zeroclaw/`에 설치되는 파일:
```
~/.zeroclaw/
├── config.toml   ← config.default.toml에서 복사
├── .env          ← 소스 .env에서 복사
└── workspace/
    ├── USER.md
    ├── SOUL.md
    └── AGENTS.md
```

- `config.default.toml`에 개인정보 없음 — 커밋 안전
- 텔레그램 토큰, API 키 등은 `.env` 환경변수로 주입
- `onboard.sh`가 config와 `.env`를 `~/.zeroclaw/`에 복사 (설치 후 repo 삭제 가능)
- `config.default.toml`이나 `.env` 수정 후 `onboard.sh --config`로 재적용

## .env 설정

LLM 프로바이더를 하나 선택하세요 — Gemini 또는 Azure OpenAI.

```bash
# --- Google Gemini (기본) ---
export ZEROCLAW_API_KEY=<API 키>
export ZEROCLAW_PROVIDER=gemini
export ZEROCLAW_MODEL=gemini-2.5-flash

# --- Azure OpenAI (대안) ---
# 아래 주석을 해제하고 위의 Gemini 섹션을 주석 처리하세요.
# export ZEROCLAW_PROVIDER=custom:https://<리소스>.openai.azure.com/openai/v1
# export ZEROCLAW_MODEL=gpt-5-mini
# export ZEROCLAW_API_KEY=<azure-api-key>
# export ZEROCLAW_TEMPERATURE=1              # Reasoning 모델 필수 (gpt-5-mini, o-시리즈)
# export AZURE_PRIVATE_ENDPOINT=<private-ip>  # Private endpoint 사용 시
# export ZEROCLAW_PROVIDER_REASONING_LEVEL=minimal  # Reasoning effort: minimal, low, medium(default), high

# 텔레그램 (선택 — 회사 내부망에서는 사용 불가)
# export TELEGRAM_BOT_TOKEN=<봇 토큰>
# export TELEGRAM_ALLOWED_USERS=<유저 ID>  # 쉼표 구분
# export TELEGRAM_MENTION_ONLY=true

# 구글 캘린더 (선택)
export GOG_ACCOUNT=you@gmail.com
export GOG_KEYRING_PASSWORD=<비밀번호>
export GOG_KEYRING_BACKEND=file
```

### Reasoning Level (Azure OpenAI)

`ZEROCLAW_PROVIDER_REASONING_LEVEL`은 모델이 응답 전에 수행하는 추론 수준을 제어합니다.
`gpt-5-mini`, o-시리즈 등 reasoning 모델에 적용됩니다.

| 레벨 | Reasoning 토큰 | 속도 | 권장 용도 |
|---|---|---|---|
| `minimal` | 0 | 가장 빠름 | 단순 작업 — 날씨, 인사, 빠른 조회 |
| `low` | ~64 | 빠름 | 가벼운 추론 — 요약, 기본 Q&A |
| `medium` | ~192 | 기본값 | 일반 용도 (미설정 시 기본값) |
| `high` | 전체 | 가장 느림 | 복잡한 분석, 다단계 추론 |

> **팁:** 일상적인 홈 어시스턴트 용도(날씨, 일정, 기기 제어)에는
> `minimal` 권장 — 추론을 완전히 생략하여 응답 속도가 크게 빨라집니다.

`config.default.toml`에서도 설정 가능:
```toml
[provider]
reasoning_level = "minimal"
```

> **참고:** 현재 `custom:` 프로바이더(예: Azure OpenAI)에서만 지원됩니다.
> 빌트인 프로바이더 프리셋(openai, gemini 등)에서는 이 설정이 적용되지 않습니다.

## 개인 파일

온보딩 후 `~/.zeroclaw/workspace/USER.md`를 편집하세요:

```markdown
# USER.md
- **이름:** (닉네임)
- **타임존:** Asia/Seoul

## 캘린더
- **기본(primary):** you@gmail.com
- **추가 캘린더:** calendar-id@group.calendar.google.com

## 날씨
- 위치: 서울 강서구 (latitude=37.55, longitude=126.85)
```

캘린더 ID 확인:
```bash
GOG_KEYRING_BACKEND=file GOG_KEYRING_PASSWORD=<pw> gog calendar calendars -a you@gmail.com
```

**업그레이드 시 백업 필요:**
- `~/.zeroclaw/.env` — API 키, 토큰
- `~/.zeroclaw/workspace/USER.md` — 개인정보

## onboard.sh 사용법

```bash
onboard.sh                              # 풀 온보딩 (첫 설치)
onboard.sh --build                      # 빌드 + 풀 온보딩
onboard.sh --binary                     # 바이너리만 교체
onboard.sh --build --binary             # 빌드 + 바이너리만
onboard.sh --skills                     # 스킬만 교체
onboard.sh --config                     # 설정만 (config.toml + 프로필)
onboard.sh --clear                      # 설치된 파일 전체 제거
onboard.sh --target 192.168.1.50        # 타겟 배포
onboard.sh --target IP --build          # 타겟에 크로스빌드 + 배포
onboard.sh --target IP --binary         # 타겟에 바이너리만 교체
onboard.sh --target IP --build --binary # 타겟에 빌드 + 바이너리만
onboard.sh --target IP --skills         # 타겟에 스킬만
onboard.sh --target IP --config         # 타겟에 설정만
onboard.sh --target IP --clear          # 타겟에서 전체 제거
```

### 바이너리 교체 동작

`--binary`, `--skills`, `--config`는 실행 중인 프로세스를 자동으로 처리합니다:
- 바이너리 교체 전 모든 zeroclaw 프로세스(daemon, agent)를 중지
- **daemon**이 실행 중이었으면 교체 후 자동 재시작
- agent만 실행 중이었거나 아무것도 없었으면 재시작 안 함

### 풀 온보딩 자동 테스트

풀 온보딩 (scope 플래그 없이 `onboard.sh` 실행) 시 설치 후 자동 테스트를 수행합니다.
모든 스킬 테스트는 zeroclaw를 통해 실행되므로 전체 파이프라인을 검증합니다.

| 테스트 | 방법 | 통과 기준 |
|---|---|---|
| agent | zeroclaw에 "안녕~" 전송 | Exit 0 |
| weather | zeroclaw에 날씨 요청 | Agent OK + 유효한 응답 |
| calendar | zeroclaw에 일정 요청 | Agent OK + gog 설치 + 유효한 응답 |
| tv-control | zeroclaw에 실행 앱 요청 | Agent OK + luna-send 사용 가능 |

Agent 테스트 실패 시 (LLM 연결 불가) 스킬 테스트는 자동 SKIP됩니다.

### 제거 (--clear)

`--clear`는 onboard.sh로 설치한 모든 것을 제거합니다:
- zeroclaw daemon 중지
- `~/.local/bin/zeroclaw` 바이너리 제거
- `~/.zeroclaw/` 전체 제거 (설정 + 워크스페이스)
- Azure private endpoint의 `/etc/hosts` 항목 제거 (bind mount 사용 시 해제)

실행 전 확인 프롬프트가 표시됩니다.

## 개발 워크플로우

> **주의:** 소스 코드 수정 후에는 반드시 릴리즈 빌드(`--build`)를 해야 합니다.
> `onboard.sh --binary`만으로는 기존 릴리즈 바이너리를 복사할 뿐 다시 빌드하지 않습니다.

```bash
# 코드 수정 후: 릴리즈 빌드 + 바이너리 교체 (--build 필수)
onboard.sh --build --binary

# 스킬 수정 후: 스킬 교체
onboard.sh --skills

# config/.env 수정 후: 재적용 + restart
onboard.sh --config
pkill -f "zeroclaw daemon" && source ~/.zeroclaw/.env && zeroclaw daemon
```

## release.sh 사용법

```bash
release.sh --version v0.2.0-lisa                # 전 플랫폼 (기본)
release.sh --version v0.2.0-lisa --target host  # 호스트만
release.sh --version v0.2.0-lisa --skip-build   # 빌드 스킵
release.sh --version v0.2.0-lisa --dry-run      # 드라이런 (업로드 안 함)
```

### 릴리즈 번들 내용물

```
lisa-v0.2.0-lisa-<platform>/
├── onboard.sh          # 온보딩 스크립트
├── zeroclaw            # 바이너리
├── .env.example        # 시크릿 템플릿
├── config/
│   └── config.default.toml
├── docs/               # 가이드 문서
│   ├── guide.md
│   ├── guide-ko.md
│   └── gogcli-oauth-setup-guide.md
├── profiles/
│   └── lisa/
│       ├── SOUL.md
│       ├── AGENTS.md
│       ├── USER.md.example
│       └── skills/
└── bin/                # 의존 바이너리 (gog 등)
```

## 구글 캘린더 설정

### gog CLI 설치

`onboard.sh`가 `gog`가 없으면 자동으로 설치합니다.
바이너리 탐색 순서:
1. `bin/gog-linux-{arm64,amd64}` — `lisa/bin/`의 아키텍처별 바이너리 (로컬 빌드)
2. `bin/gog` — 범용 바이너리 (릴리즈 번들)
3. GitHub 릴리즈 다운로드 (fallback)

로컬: `~/.local/bin/gog`에 설치, 타겟: 배포 디렉토리에 설치

#### 소스에서 빌드

[gogcli](https://github.com/steipete/gogcli) 소스에서 빌드하고 `lisa/bin/`에 복사하면 `onboard.sh`가 자동으로 인식합니다:

```bash
# gogcli 소스를 원하는 위치에 클론
git clone https://github.com/steipete/gogcli.git
cd gogcli

# 양쪽 아키텍처로 static 빌드
mkdir -p /path/to/lisa/lisa/bin
CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -ldflags "-s -w" -o /path/to/lisa/lisa/bin/gog-linux-amd64 ./cmd/gog
CGO_ENABLED=0 GOOS=linux GOARCH=arm64 go build -ldflags "-s -w" -o /path/to/lisa/lisa/bin/gog-linux-arm64 ./cmd/gog
```

> `lisa/bin/`은 gitignore 대상 — 빌드된 바이너리는 로컬에만 존재합니다.

#### 수동 설치 (릴리즈 번들에서 추출)

```bash
gh release download --repo rordd/lisa --pattern "*apple-darwin*"       # macOS
gh release download --repo rordd/lisa --pattern "*x86_64*linux-gnu*"   # Linux x86_64
gh release download --repo rordd/lisa --pattern "*aarch64*linux-gnu*"  # Linux ARM64
tar xzf lisa-*.tar.gz
cp lisa-*/bin/gog ~/.local/bin/gog && chmod +x ~/.local/bin/gog
```

### 인증 & 테스트

```bash
# 인증 (최초 1회, 브라우저 필요)
GOG_KEYRING_BACKEND=file GOG_KEYRING_PASSWORD=<pw> gog auth add you@gmail.com --services calendar --manual
# 브라우저 없는 환경: 다른 기기에서 URL 열고 리다이렉트 URL 복사 → 붙여넣기

# 테스트
GOG_KEYRING_BACKEND=file GOG_KEYRING_PASSWORD=<pw> gog calendar events primary --from today --to tomorrow

# 캘린더 ID 목록 확인 (USER.md에 넣을 ID 조회)
GOG_KEYRING_BACKEND=file GOG_KEYRING_PASSWORD=<pw> gog calendar calendars -a you@gmail.com
```

## 타겟 보드 배포

```bash
# 1. 릴리즈 번들 사용 (권장)
gh release download v0.2.0-lisa --repo rordd/lisa --pattern "*linux-gnu*"
tar xzf lisa-v0.2.0-lisa-aarch64-unknown-linux-gnu.tar.gz
cd lisa-v0.2.0-lisa-aarch64-unknown-linux-gnu
cp .env.example .env && vi .env
./onboard.sh --target <보드IP>

# 2. 소스에서 크로스빌드
cd ~/project/lisa
./lisa/scripts/onboard.sh --build --target <보드IP>
```

### 타겟 설치 구조

로컬과 동일한 디렉토리 구조 — 설정과 시크릿은 `~/.zeroclaw/`, 바이너리는 배포 디렉토리:

```
/home/root/lisa/            ← 바이너리 + 의존성 (배포 디렉토리)
├── zeroclaw
└── gog

/home/root/.zeroclaw/       ← 설정 + 시크릿 + 워크스페이스 (로컬 ~/.zeroclaw/과 동일)
├── config.toml
├── .env
└── workspace/
    ├── USER.md
    ├── SOUL.md
    ├── AGENTS.md
    └── skills/
```

### 타겟에서 실행

```bash
# SSH로 실행
ssh root@<보드IP> 'export PATH=/home/root/lisa:$PATH && source ~/.zeroclaw/.env && zeroclaw daemon'
ssh root@<보드IP> 'export PATH=/home/root/lisa:$PATH && source ~/.zeroclaw/.env && zeroclaw agent'
```

### 요구사항
- SSH 키 기반 접속 설정
- 크로스 빌드 툴체인 (택 1):
  - **방법 A**: `cross` CLI + Docker (`cargo install cross`)
  - **방법 B**: 네이티브 musl 툴체인 (`sudo apt install gcc-aarch64-linux-gnu musl-tools` + `rustup target add aarch64-unknown-linux-musl`)

## 플랫폼별 번들

| 플랫폼 | 파일명 | 용도 |
|---|---|---|
| macOS ARM64 | `aarch64-apple-darwin` | 맥 (M-시리즈) |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | 타겟 보드 (webOS 등) |
| Linux x86_64 | `x86_64-unknown-linux-gnu` | 리눅스 서버 |
