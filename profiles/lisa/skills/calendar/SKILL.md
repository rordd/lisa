---
name: calendar
description: "Google 캘린더 일정 조회/생성. gog CLI 사용. 사용자가 일정, 스케줄, 미팅을 물어볼 때 사용."
version: "1.0.0"
always: true
---

# 캘린더 스킬

gog CLI로 Google Calendar를 관리한다.

## 사전 조건

gog 설치 및 OAuth 인증 완료 필요:
```bash
brew install steipete/tap/gogcli
gog auth credentials /path/to/client_secret.json
gog auth add <email> --services calendar
```

## 사용 시점

- "오늘 일정 뭐야?"
- "내일 미팅 있어?"
- "이번 주 스케줄"
- "회의 잡아줘"

## 캘린더 목록

USER.md에서 캘린더 ID를 참조. 주요 캘린더:
- primary — 개인 기본
- 추가 캘린더는 USER.md에 정의

## 명령어

### 일정 조회
```bash
# 오늘 일정
gog calendar events <calendarId> --from $(date +%Y-%m-%dT00:00:00) --to $(date +%Y-%m-%dT23:59:59) --json

# 내일 일정
gog calendar events <calendarId> --from $(date -v+1d +%Y-%m-%dT00:00:00) --to $(date -v+1d +%Y-%m-%dT23:59:59) --json

# 이번 주 일정
gog calendar events <calendarId> --from $(date +%Y-%m-%dT00:00:00) --to $(date -v+7d +%Y-%m-%dT23:59:59) --json

# 리눅스에서는 date -d "+1 day" 사용
gog calendar events <calendarId> --from $(date -d "+1 day" +%Y-%m-%dT00:00:00) --to $(date -d "+1 day" +%Y-%m-%dT23:59:59) --json
```

### 일정 생성
```bash
gog calendar create <calendarId> --summary "미팅 제목" --from 2026-03-03T14:00:00 --to 2026-03-03T15:00:00
```

### 일정 수정
```bash
gog calendar update <calendarId> <eventId> --summary "새 제목"
```

### 색상
```bash
gog calendar colors
# --event-color <1-11> 로 색상 지정 가능
```

## 환경변수

```bash
GOG_ACCOUNT=<email>              # 기본 계정
GOG_KEYRING_PASSWORD=<password>  # 키링 비밀번호 (keyring=file 모드)
```

## 여러 캘린더 조회

브리핑 시 USER.md에 정의된 모든 캘린더를 순회:
```bash
for cal in "primary" "cal_id_1" "cal_id_2"; do
  echo "=== $cal ==="
  gog calendar events "$cal" --from ... --to ... --json
done
```

## 주의
- 일정 생성/수정은 반드시 사용자 확인 후 실행
- `--json` 플래그로 파싱 가능한 출력
- `GOG_ACCOUNT` 설정 안 하면 `--account` 매번 필요
