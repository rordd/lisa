# 🎀 Lisa 설치 가이드

Lisa는 ZeroClaw 기반 온디바이스 AI 홈 에이전트입니다.

## 사전 요구사항

### Rust 툴체인
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 빌드 의존성

**Ubuntu/Debian:**
```bash
sudo apt update && sudo apt install -y build-essential pkg-config libssl-dev cmake
```

**macOS:**
```bash
xcode-select --install
```

## 설치

### 1. 클론 & 빌드
```bash
git clone https://github.com/rordd/lisa.git ~/project/lisa
cd ~/project/lisa
cargo install --path .
```

### 2. 환경 설정
```bash
cp profiles/.env.example .env
```

`.env` 파일을 편집해서 자신의 설정을 입력합니다:

#### Gemini 사용 시
```bash
ZEROCLAW_API_KEY=AIza...
ZEROCLAW_PROVIDER=gemini
ZEROCLAW_MODEL=gemini-2.5-flash
```

#### Azure OpenAI 사용 시
```bash
ZEROCLAW_API_KEY=<Azure API Key>
ZEROCLAW_PROVIDER=custom:azure
ZEROCLAW_MODEL=gpt-4o
AZURE_OPENAI_BASE_URL=https://<리소스명>.openai.azure.com/openai/deployments/<배포명>/chat/completions?api-version=2024-02-01
AZURE_OPENAI_API_KEY=<Azure API Key>
```

#### OpenAI 사용 시
```bash
ZEROCLAW_API_KEY=sk-...
ZEROCLAW_PROVIDER=openai
ZEROCLAW_MODEL=gpt-4o
```

#### Ollama (로컬) 사용 시
```bash
ZEROCLAW_API_KEY=http://localhost:11434
ZEROCLAW_PROVIDER=ollama
ZEROCLAW_MODEL=llama3.2
```

#### 텔레그램 (선택)
```bash
TELEGRAM_BOT_TOKEN=<BotFather에서 받은 토큰>
TELEGRAM_ALLOWED_USERS=<쉼표 구분 user_id>
TELEGRAM_MENTION_ONLY=true
```

### 3. 셋업 실행
```bash
./scripts/setup-lisa.sh
```

이 스크립트가 하는 일:
- `~/.zeroclaw/` 디렉토리 생성
- 공유 설정 (`config.shared.toml`) → `config.toml`로 복사
- 텔레그램 설정 주입 (`.env`에 봇토큰이 있으면)
- 성격 파일 (`SOUL.md`, `AGENTS.md`) → workspace에 복사

### 4. 개인 정보 설정

```bash
cp profiles/lisa/USER.md.example ~/.zeroclaw/workspace/USER.md
```

`USER.md`를 편집해서 자신의 정보를 입력합니다 (이름, 타임존, 캘린더 등).

### 6. 실행
```bash
source .env && zeroclaw daemon
```

## 확인

```bash
# 상태 확인
zeroclaw status

# CLI 채팅
source .env && zeroclaw chat "안녕 리사!"

# 웹 대시보드
# http://localhost:42617
```

## 트러블슈팅

| 증상 | 해결 |
|------|------|
| `libssl-dev` 관련 빌드 실패 | `sudo apt install libssl-dev` |
| `cmake` 관련 빌드 실패 | `sudo apt install cmake` |
| 포트 충돌 | `config.toml`에서 `[gateway]` port 변경 |
| Azure 인증 실패 | `auth_header = "api-key"` 확인, base_url에 `?api-version=` 포함 확인 |
| 텔레그램 연결 안 됨 | `allowed_users`에 자신의 user_id 확인 |

## 프로젝트 구조

```
profiles/
├── .env.example           # 시크릿 템플릿 (Git 추적 안 됨)
└── lisa/
    ├── config.shared.toml  # 공유 설정 (시크릿 제외)
    ├── SOUL.md             # 리사 성격
    ├── AGENTS.md           # 에이전트 규칙
    └── USER.md.example     # 개인 정보 템플릿
```

- **공유 설정**은 Git으로 관리 → `git pull`로 업데이트
- **시크릿**은 `.env`로만 관리 → Git에 커밋 안 됨
- **개인 정보**는 `USER.md`로 각자 관리
