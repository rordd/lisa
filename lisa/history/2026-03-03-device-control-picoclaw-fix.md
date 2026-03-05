# 2026-03-03: device-control 스킬 picoclaw 경로 제거

## Summary

`device-control` 스킬이 picoclaw 전용 경로(`~/.picoclaw/workspace/`)를 참조하고 있어, ZeroClaw 등 다른 AI Agent에서 사용할 수 없었던 문제를 수정. 상대 경로(`apps.json`)로 변경하여 agent-agnostic하게 동작하도록 수정.

## 배경

`device-control` 스킬은 picoclaw에서 먼저 개발된 스킬로, apps.json 파일의 경로가 `~/.picoclaw/workspace/apps.json`으로 하드코딩되어 있었음. ZeroClaw나 다른 AI Agent에서는 이 경로가 존재하지 않아 스킬이 정상 동작하지 않음.

## 수정 내용

| 위치 | 수정 전 | 수정 후 |
|------|---------|---------|
| Build apps.json | `> ~/.picoclaw/workspace/apps.json` | `> apps.json` |
| Read apps.json | `cat ~/.picoclaw/workspace/apps.json` | `cat apps.json` |

상대 경로를 사용함으로써, 어떤 AI Agent의 작업 디렉토리에서든 apps.json을 생성/읽기할 수 있음.

## Changes

| # | Path | Description |
|---|---|---|
| 1 | `lisa/profiles/lisa/skills/device-control/SKILL.md` | `~/.picoclaw/workspace/apps.json` → `apps.json` (2곳) |
