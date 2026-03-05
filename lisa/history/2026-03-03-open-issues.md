# 2026-03-03: open-issues 관리 체계 구축

## Summary

`lisa/open-issues/` 디렉토리와 `lisa/scripts/issues.sh` 관리 스크립트를 추가하여, 프로젝트의 미해결 이슈를 로컬에서 관리할 수 있도록 함.

## 배경

기존에는 `lisa/history/`에서 완료된 변경 이력만 추적하고, 진행 중이거나 해결해야 할 이슈를 관리하는 체계가 없었음.

## 이슈 파일 포맷

파일명: `{id}-{short-title}.md` (예: `001-temperature-hardcoded.md`)

YAML frontmatter로 메타데이터 관리:
```yaml
---
id: 001
title: 이슈 제목
status: open          # open | closed
priority: high        # high | medium | low
category: bug         # bug | feature | improvement | config
created: 2026-03-03
updated: 2026-03-03
---
```

## 스크립트 기능 (`lisa/scripts/issues.sh`)

| 커맨드 | 설명 |
|--------|------|
| `new` | 대화형으로 새 이슈 생성 (title, priority, category 입력, ID 자동 채번) |
| `list` | open 이슈 목록 (테이블 형태) |
| `list --all` | closed 포함 전체 목록 |
| `list --priority <p>` | 우선순위 필터링 |
| `list --category <c>` | 카테고리 필터링 |
| `show <id>` | 이슈 상세 보기 |
| `close <id>` | 이슈 닫기 (status 변경 + updated 갱신) |
| `reopen <id>` | 이슈 다시 열기 |
| `delete <id>` | 이슈 파일 삭제 (확인 프롬프트) |
| `summary` | 전체 통계 (상태별/우선순위별/카테고리별) |

## 구현 세부사항

- ID: 3자리 zero-pad (001, 002, ...), 기존 최대 ID + 1 자동 채번
- frontmatter 파싱: `sed`/`grep`으로 YAML frontmatter에서 필드 추출
- slug 생성: ASCII 소문자 + 숫자 + 하이픈만 허용 (한글 제거)
- `set -euo pipefail` + `shopt -s nullglob`: 빈 디렉토리에서도 안전하게 동작
- `set -e` 호환: `[ ... ] && echo` 패턴 대신 `if [ ... ]; then echo; fi` 사용

## 테스트 결과

```
✅ bash -n issues.sh (문법 검사 통과)
✅ new — 이슈 생성 (001-agent-cli-temperature-hardcoded.md)
✅ list — open 이슈 테이블 출력
✅ list --priority high — 필터링 동작
✅ list --category bug — 필터링 동작
✅ show 1 — 상세 보기
✅ close 1 — status: open → closed, updated 갱신
✅ reopen 1 — status: closed → open
✅ summary — 통계 출력 (0건, 1건 모두 정상)
✅ delete 1 — 파일 삭제
```

## Changes

| # | Path | Description |
|---|---|---|
| 1 | `lisa/open-issues/README.md` | 신규 — 이슈 파일 포맷, 필드 설명, 스크립트 사용법 |
| 2 | `lisa/scripts/issues.sh` | 신규 — 이슈 관리 CLI (CRUD + summary, 필터링) |
| 3 | `lisa/history/SUMMARY.md` | 변경 이력 추가, 프로젝트 구조에 open-issues/ 반영 |
