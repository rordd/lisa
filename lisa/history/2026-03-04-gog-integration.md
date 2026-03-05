# 2026-03-04: gog (캘린더) 통합 및 배포 자동화

## 배경

캘린더 스킬이 gog CLI를 사용하지만, 타겟 배포 시 gog 관련 설정이 수동이었음.
이 변경으로 배포 스크립트가 gog OAuth 인증, 환경변수 설정, 바이너리/크레덴셜 전송을 자동 처리.

## 변경 사항

### 1. gogcli ARM64 빌드

```bash
cd lisa/gogcli
CGO_ENABLED=0 GOOS=linux GOARCH=arm64 go build -ldflags="-s -w" -o ../release/arm64/gog ./cmd/gog
```

- 결과: 21MB static 바이너리 (`lisa/release/arm64/gog`)

### 2. history compaction temperature 수정

`src/agent/loop_/history.rs`에서 `auto_compact_history`와 `flush_durable_facts`가 temperature=0.2를 하드코딩하고 있어 gpt-5-mini (temperature=1.0만 지원)에서 에러 발생.

- `temperature: f64` 파라미터 추가
- `src/agent/loop_.rs`에서 `config.default_temperature` 전달

### 3. gog 바이너리 전송 (step 4)

deploy-target.sh/ps1의 바이너리 전송 단계에 gog 바이너리 추가 (존재 시).

### 4. USER.md 전송 (step 6)

workspace 파일 전송 목록에 `USER.md` 추가.

### 5. gog 크레덴셜 전송 (step 7)

`~/.config/gogcli/` 디렉토리를 타겟의 `/home/root/.config/gogcli/`로 전송.

### 6. lisa.env 메커니즘

- `lisa/profiles/lisa/lisa.env.example` 템플릿 생성
- start script에 `[ -f lisa.env ] && . lisa.env` 추가
- deploy script에서 `lisa.env` 전송 + `chmod 600`
- PATH에 `/home/root/lisa` 추가 (.profile + start scripts)

### 7. gog 자동 셋업 (step 7 내 통합)

deploy script 실행 시 gog 사전 조건을 자동으로 확인/설정:

1. **OAuth 토큰 확인**: `~/.config/gogcli/keyring/` 디렉토리에 파일 존재 여부 체크
2. **gog CLI 설치**: 미설치 시 `brew install` 또는 `go install`로 자동 설치 (GOPATH/bin PATH 추가 포함)
3. **Client credentials 설정**: `credentials.json` 없으면 `client_secret.json` 경로 입력받아 `gog auth credentials` 실행
4. **Keyring 백엔드 설정**: `gog auth keyring file` 실행 (headless 타겟용 파일 기반 키링)
5. **OAuth 인증**: 이메일 입력받아 `gog auth add <email> --services calendar --manual` 실행 (URL 복사-붙여넣기 방식, 로컬 콜백 서버 불필요)
6. **lisa.env 자동 생성**: 파일 없으면 `GOG_ACCOUNT`/`GOG_KEYRING_PASSWORD` 입력받아 생성

모든 gog 관련 프롬프트는 `[Y/n]` (기본 Yes)로 스킵 가능.

### 8. lisa.env `export` 누락 수정

`lisa.env`에서 변수 선언 시 `export`가 없으면 `. lisa.env`로 로드해도 자식 프로세스(gog)에서 변수를 읽을 수 없음.

- 모든 변수에 `export` prefix 추가: `export GOG_ACCOUNT=...`
- deploy script의 lisa.env 생성/업데이트 로직 전부 `export` prefix 반영
- grep/sed 패턴에 optional `export` prefix 처리 추가

### 9. gog calendar 명령어 수정

`gog calendar list`는 존재하지 않는 명령. gogcli 소스 확인 결과 올바른 명령은 `gog calendar calendars`.

- deploy script 테스트: `gog calendar list` → `gog calendar calendars`

### 10. 배포 후 테스트 안정화

- `set -e`로 인해 테스트 실패 시 스크립트 전체 종료 → 모든 테스트 ssh 명령에 `|| true` 추가
- weather 인터넷 체크: `curl -w '%{http_code}'` (webOS 미지원) → 단순 curl exit code 체크
- gog 설치 확인: `which gog` (non-login SSH에서 PATH 미포함) → `test -x /home/root/lisa/gog`
- calendar 테스트에서 `lisa.env` sourcing 추가 (GOG_KEYRING_PASSWORD 필요)
- weather/telegram 테스트: 인터넷 미연결 시 SKIP 메시지 표시

### 11. clean-target.sh 추가

타겟 배포 초기화 스크립트 (`lisa/scripts/clean-target.sh`):

1. Lisa 프로세스 종료 (`pkill -f zeroclaw`)
2. `/etc/hosts` bind mount 제거
3. `.profile` hook 정리
4. 타겟 디렉토리 삭제 (`/home/root/lisa/`, `~/.zeroclaw/`, `~/.config/gogcli/`)
5. 로컬 gog keyring 토큰 삭제 (`~/.config/gogcli/keyring/`)

`set -uo pipefail` 사용 (`-e` 제외 — cleanup 명령 실패 시 스크립트 종료 방지).
로컬 `lisa.env`는 삭제하지 않음 (재배포 시 재사용).

### 12. shell tool 환경변수 전달 문제 수정

zeroclaw의 shell tool은 보안을 위해 `env_clear()` 후 허용된 변수만 자식 프로세스에 전달 (`src/tools/shell.rs`).
`GOG_KEYRING_PASSWORD`, `GOG_ACCOUNT`, `GOG_KEYRING_BACKEND`가 허용 목록에 없어 gog가 패스워드 없이 실행 → 타임아웃/에러 발생.

- `config.arm64.toml`의 `[autonomy]`에 `shell_env_passthrough` 추가
- `lisa.env`에 `GOG_KEYRING_BACKEND=file` 추가 (DBUS SecretService 시도 방지)
- deploy script의 lisa.env 생성 템플릿에 `GOG_KEYRING_BACKEND=file` 추가

## 수정 파일

| 파일 | 변경 |
|------|------|
| `lisa/release/arm64/gog` | 신규 — ARM64 gog 바이너리 |
| `src/agent/loop_/history.rs` | temperature 파라미터화 |
| `src/agent/loop_.rs` | config.default_temperature 전달 |
| `lisa/scripts/deploy-target.sh` | gog 전송, lisa.env, PATH, 자동 셋업 |
| `lisa/scripts/deploy-target.ps1` | 동일 변경 |
| `lisa/profiles/lisa/lisa.env.example` | 신규 — 환경변수 템플릿 |
| `lisa/profiles/lisa/skills/calendar/SKILL.md` | `gog auth keyring file` 단계 추가 |
| `lisa/docs/deploy-target-guide.md` | gog 설정 가이드, 트러블슈팅 추가 |
| `lisa/scripts/clean-target.sh` | 신규 — 타겟 초기화 및 로컬 토큰 삭제 |
| `lisa/profiles/lisa/lisa.env` | `export` prefix 추가, `GOG_KEYRING_BACKEND=file` 추가 |
| `lisa/config/config.arm64.toml` | `shell_env_passthrough` 추가 (GOG 환경변수 전달) |

## 자동 셋업 흐름

```
[7/13] Setting up gog (calendar)...
  No gog OAuth tokens found locally.
  gog CLI not found.
  Install gog now? [Y/n]:
  Installing via go install...
  Installed: /home/user/go/bin/gog
  Set up Google Calendar (gog) now? [Y/n]:
  Path to client_secret.json (from Google Cloud Console): ~/Downloads/client_secret.json
  Setting keyring backend to file...
  Google account email: user@gmail.com
  Browser will open for OAuth consent...
  NOTE: Remember the keyring password you set — needed for lisa.env
  ...
  3 file(s) transferred
  Create lisa.env? [Y/n]:
  GOG_ACCOUNT (email): user@gmail.com
  GOG_KEYRING_PASSWORD: ****
  lisa.env created
  lisa.env
  OK
```
