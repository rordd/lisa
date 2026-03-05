# 2026-03-03: 타겟 아키텍처 ARM32 → ARM64 변경

## Summary

타겟 바이너리를 ARM32에서 ARM64로 변경.
config 파일명, 배포 스크립트, 가이드 문서 전체 반영.

## 배경

- webOS TV 커널이 aarch64 (ARM64)
- ARM32 바이너리는 dynamic linker 경로 불일치 (`/lib/ld-linux-armhf.so.3` vs `/lib/ld-linux.so.3`) 문제 발생
- `/lib/`이 Read-Only 영역이라 심볼릭 링크 생성 불가
- ARM64 바이너리로 전환하여 해결

## Changes

### Renamed

| # | From | To | Description |
|---|---|---|---|
| 1 | `lisa/config/config.arm32.toml` | `lisa/config/config.arm64.toml` | config 파일명 변경 (내용 동일, 헤더만 ARM64로 수정) |

### Modified

| # | Path | Description |
|---|---|---|
| 1 | `lisa/scripts/deploy-target.sh` | 바이너리 경로 `release/arm32/` → `release/arm64/`, config 경로 `config.arm32.toml` → `config.arm64.toml`, 메시지 ARM32 → ARM64 |
| 2 | `lisa/scripts/deploy-target.ps1` | 동일 변경 |
| 3 | `lisa/docs/deploy-target-guide.md` | ARM32 → ARM64 전체 반영 |
| 4 | `lisa/docs/setup-guide.md` | 프로젝트 구조에서 arm32 → arm64 |
