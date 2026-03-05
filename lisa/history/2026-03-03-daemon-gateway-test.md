# 2026-03-03: daemon gateway 테스트 추가

## Summary

배포 스크립트에 daemon gateway API 테스트를 추가하고, 가이드 문서에 수동 테스트 방법을 문서화.

## 배경

daemon 모드 실행 확인이 `zeroclaw status`만으로는 gateway가 실제로 동작하는지 검증 불가. 텔레그램 채널이 타겟에서 차단되어 있어 gateway API를 통한 테스트가 필요.

## Gateway API 테스트 흐름

1. `/health` — 인증 불필요, daemon 및 gateway 동작 확인
2. `/pair` — daemon 로그의 pairing code를 `X-Pairing-Code` 헤더로 전달하여 Bearer 토큰 발급
3. `/api/chat` — 발급받은 토큰으로 대화 요청 → Azure OpenAI 응답 확인

## 변경 내용

### 배포 스크립트 (deploy-target.sh / deploy-target.ps1)

step 12 기능 테스트 변경:

- **12-6**: daemon 시작 (로그를 `/tmp/lisa-daemon-test.log`에 캡처) + `zeroclaw status`
- **12-7**: gateway `/health` 엔드포인트 확인
- **12-8**: daemon 로그에서 pairing code 추출 → `/pair`로 토큰 발급 → `/api/chat`으로 대화 테스트
- **12-9**: 텔레그램 Bot API getMe (기존 12-7에서 번호 변경)

daemon 시작 방식 변경:
- 기존: `start-lisa.sh &` (로그 캡처 불가)
- 변경: `nohup ./zeroclaw daemon > /tmp/lisa-daemon-test.log 2>&1 &` (pairing code 추출 가능)
- 테스트 종료 후 로그 파일 자동 삭제

### 가이드 문서 (deploy-target-guide.md)

"Daemon 테스트 (Gateway API)" 섹션 추가:
- Health Check (인증 불필요)
- Pairing (X-Pairing-Code 헤더로 토큰 발급)
- 대화 테스트 (/api/chat)
- Gateway API 요약 테이블

자동 테스트 테이블 업데이트 (6→9항목):
- #7: gateway /health
- #8: gateway /pair + /api/chat

트러블슈팅 항목 추가:
- `/health` 응답 없음
- `/pair` Invalid pairing code (헤더 vs JSON body 주의)
- `/api/chat` Unauthorized

## 참고

- pairing code는 daemon 시작 시 1회 생성, 재시작 시 변경됨
- pairing code는 `X-Pairing-Code` **헤더**로 전달 (JSON body 아님)
- gateway는 `127.0.0.1`에 바인딩되므로 SSH를 통해 테스트

## Changes

| # | Path | Description |
|---|---|---|
| 1 | `lisa/scripts/deploy-target.sh` | step 12-7 /health, 12-8 /pair + /api/chat 추가, daemon 시작 방식 변경 |
| 2 | `lisa/scripts/deploy-target.ps1` | 동일 변경 |
| 3 | `lisa/docs/deploy-target-guide.md` | daemon 테스트 가이드, 자동 테스트 테이블, 트러블슈팅 추가 |
