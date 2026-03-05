# 2026-03-03: 텔레그램 채널 연결 설정 추가

## Summary

타겟(webOS TV)에서 텔레그램 봇을 통해 원격으로 Lisa를 제어할 수 있도록 설정 추가. daemon 모드에서 텔레그램 long-polling 방식으로 메시지를 수신하고 응답.

## 배경

기존에는 타겟에 SSH로 접속한 후 CLI(agent/daemon)로만 Lisa와 대화할 수 있었음. 텔레그램 봇 채널을 추가하면 모바일에서 언제든 원격으로 명령 가능.

## 변경 내용

### config.arm64.toml

`[channels_config.telegram]` 섹션 추가:

```toml
[channels_config.telegram]
bot_token = "YOUR_BOT_TOKEN"         # @BotFather에서 발급
allowed_users = ["YOUR_USER_ID"]     # @userinfobot으로 확인
mention_only = false
stream_mode = "partial"
ack_enabled = true
```

- `bot_token`, `allowed_users`: 샘플 값 (사용 전 실제 값으로 교체 필요)
- `mention_only = false`: 1:1 채팅 위주이므로 모든 메시지에 응답
- `stream_mode = "partial"`: 응답을 점진적으로 표시
- `ack_enabled = true`: 메시지 수신 시 이모지 반응

### 배포 스크립트

step 12에 텔레그램 Bot API `getMe` 테스트 추가:
- config에서 `bot_token` 추출
- 샘플 값(`YOUR_BOT_TOKEN`)이면 테스트 스킵
- 실제 토큰이면 `https://api.telegram.org/bot<TOKEN>/getMe` 호출로 유효성 검증

### 가이드 문서

- 텔레그램 봇 생성 방법 (@BotFather)
- 사용자 ID 확인 방법 (@userinfobot)
- config 수정 예시 및 필드 설명
- daemon 모드에서만 텔레그램 동작함을 명시
- 트러블슈팅 항목 추가

## 참고

- 텔레그램은 **daemon 모드**에서만 동작 (agent 모드는 CLI 전용)
- long-polling 방식 → 타겟에서 별도 포트 오픈 불필요
- `setup-lisa.sh`는 이미 `.env`에서 텔레그램 설정 주입 로직이 있어 수정 불필요

## Changes

| # | Path | Description |
|---|---|---|
| 1 | `lisa/config/config.arm64.toml` | `[channels_config.telegram]` 섹션 추가 |
| 2 | `lisa/scripts/deploy-target.sh` | step 12에 텔레그램 getMe 테스트 추가 |
| 3 | `lisa/scripts/deploy-target.ps1` | 동일 변경 |
| 4 | `lisa/docs/deploy-target-guide.md` | 텔레그램 설정 가이드, 트러블슈팅 추가 |
