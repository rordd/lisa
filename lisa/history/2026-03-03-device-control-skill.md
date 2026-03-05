# 2026-03-03: device-control 스킬 추가

## Summary

webOS TV 디바이스 제어를 위한 `device-control` 스킬 추가.
배포 스크립트에 스킬 내 실행 스크립트 권한 부여 로직 추가.

## Changes

### Created

| # | Path | Description |
|---|---|---|
| 1 | `lisa/profiles/lisa/skills/device-control/SKILL.md` | 디바이스 제어 스킬 정의 |
| 2 | `lisa/profiles/lisa/skills/device-control/scripts/go-to-channel.sh` | 채널 번호 직접 이동 스크립트 |

### Modified

| # | Path | Description |
|---|---|---|
| 1 | `lisa/scripts/deploy-target.sh` | 스킬 스크립트에 `chmod +x` 실행 권한 부여 추가 |
| 2 | `lisa/scripts/deploy-target.ps1` | 동일 — 스킬 스크립트 실행 권한 부여 추가 |
| 3 | `lisa/docs/deploy-target-guide.md` | device-control 스킬 반영 (배포 단계, 디렉토리 구조) |

## 스킬 상세

### device-control

`exec` 도구를 통해 luna-send 명령으로 webOS TV를 제어하는 스킬.

**지원 기능:**

| 기능 | 명령 |
|------|------|
| 앱 실행 (by id) | `luna-send ... applicationManager/launch` |
| 앱 실행 (by category) | `luna-send ... applicationManager/launchDefaultApp` |
| 현재 앱 확인 | `luna-send ... applicationManager/getForegroundAppInfo` |
| 채널 올리기/내리기 | `luna-send ... inputgenerator/pushKeyEvent` (keycodenum 402/403) |
| 채널 번호 이동 | `scripts/go-to-channel.sh {number}` (networkinput/sendSpecialKey) |
| 볼륨 올리기/내리기 | `luna-send ... audio/master/volumeUp|volumeDown` |
| 볼륨 설정 | `luna-send ... audio/master/setVolume` |

**주요 규칙:**
- 모든 동작은 반드시 `exec` 도구를 통해 실행
- 채널 제어는 Live TV 앱(`com.webos.app.livetv`)이 foreground일 때만 동작
- `apps.json` 캐시를 통해 앱 목록 관리
