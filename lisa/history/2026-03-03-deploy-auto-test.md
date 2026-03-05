# 2026-03-03: 배포 후 자동 기능 테스트 추가

## Summary

배포 스크립트(deploy-target.sh / deploy-target.ps1)에 step 12 자동 기능 테스트 추가. 배포 완료 후 타겟에서 각 기능이 정상 동작하는지 자동 검증.

## 테스트 항목

| # | 테스트 | 검증 내용 |
|---|--------|-----------|
| 12-1 | agent 모드 단일 메시지 | `lisa-agent.sh 안녕` → Azure OpenAI 응답 확인 |
| 12-2 | device-control: getForegroundAppInfo | `luna-send` 명령 실행 가능 여부 |
| 12-3 | device-control: getVolume | 볼륨 API 호출 가능 여부 |
| 12-4 | weather: wttr.in 조회 | `curl` 기반 외부 API 접근 여부 |
| 12-5 | calendar: gog CLI | gog 설치 여부 확인 (미설치 시 건너뜀) |
| 12-6 | daemon 모드 | daemon 시작 → `zeroclaw status` 확인 |

## 구현

- `run_test()` 헬퍼 함수: 테스트명, 결과, exit code를 받아 pass/fail 카운트
- 테스트 결과 `✅ N 통과 / ❌ N 실패`로 요약 출력

## Changes

| # | Path | Description |
|---|---|---|
| 1 | `lisa/scripts/deploy-target.sh` | step 12 자동 테스트 추가 |
| 2 | `lisa/scripts/deploy-target.ps1` | 동일 변경 |
| 3 | `lisa/docs/deploy-target-guide.md` | 배포 후 자동 테스트 섹션 추가 |
