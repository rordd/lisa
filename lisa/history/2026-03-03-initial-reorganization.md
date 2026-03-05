# 2026-03-03: Initial Reorganization

## Summary

changwook.im이 추가한 Lisa 프로젝트 관련 파일들을 프로젝트 root의 `lisa/` 디렉토리 아래로 이동하여 관리 범위를 명확히 분리.

## Changes

### Moved (Rename)

| # | Original Path (project root 기준) | New Path (lisa/ 기준) | Type |
|---|---|---|---|
| 1 | `profiles/.env.example` | `lisa/profiles/.env.example` | Moved |
| 2 | `profiles/lisa/AGENTS.md` | `lisa/profiles/lisa/AGENTS.md` | Moved |
| 3 | `profiles/lisa/SOUL.md` | `lisa/profiles/lisa/SOUL.md` | Moved |
| 4 | `profiles/lisa/USER.md.example` | `lisa/profiles/lisa/USER.md.example` | Moved |
| 5 | `profiles/lisa/config.shared.toml` | `lisa/profiles/lisa/config.shared.toml` | Moved |
| 6 | `profiles/lisa/skills/calendar/SKILL.md` | `lisa/profiles/lisa/skills/calendar/SKILL.md` | Moved |
| 7 | `profiles/lisa/skills/weather/SKILL.md` | `lisa/profiles/lisa/skills/weather/SKILL.md` | Moved |
| 8 | `docs/setup-guide.md` | `lisa/docs/setup-guide.md` | Moved |
| 9 | `scripts/setup-lisa.sh` | `lisa/scripts/setup-lisa.sh` | Moved |

| 10 | `release/arm64/zeroclaw` | `lisa/release/arm64/zeroclaw` | Moved |

### Created

| # | Path | Description |
|---|---|---|
| 1 | `lisa/` | Lisa 프로젝트 전용 루트 디렉토리 |
| 2 | `lisa/history/` | 파일 변경 이력 기록 디렉토리 |
| 3 | `lisa/history/2026-03-03-initial-reorganization.md` | 이 파일 (최초 재구성 기록) |
| 4 | `lisa/history/changwook.im-code-modifications.md` | changwook.im 코드 수정 이력 |

### Deleted

- 원본 위치의 빈 디렉토리 (`profiles/`, `profiles/lisa/`, `profiles/lisa/skills/`, `release/` 등) 자동 정리됨

## Rationale

- upstream ZeroClaw 코드와 Lisa 프로젝트 고유 파일을 명확히 분리
- `lisa/` 하위에서 독립적으로 파일 관리 가능
- 변경 이력을 `lisa/history/`에 체계적으로 기록
