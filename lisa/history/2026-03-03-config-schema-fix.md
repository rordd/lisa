# 2026-03-03: config 스키마 수정

## Summary

타겟에서 발생한 config 파싱 에러 2건 수정.

## 문제 및 수정

| 항목 | 수정 전 | 수정 후 | 원인 |
|------|---------|---------|------|
| `[memory]` | `enabled = true` | `backend = "markdown"`, `auto_save = true` | ZeroClaw 스키마가 `enabled` 필드를 지원하지 않음 |
| `[security]` | `sandbox = false` | `[security.sandbox]` `enabled = false` | nested table 구조 필요 |

## Changes

| # | Path | Description |
|---|---|---|
| 1 | `lisa/config/config.arm64.toml` | `[memory]`, `[security.sandbox]` 스키마 수정 |
