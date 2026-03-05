# 2026-03-03: agent 모드 temperature 에러 수정

## Summary

gpt-5-mini 모델이 temperature=0.7을 지원하지 않아 발생한 에러 수정.

## 문제

**에러:**
```
error=Custom API error (400 Bad Request): {
  "error": {
    "message": "Unsupported value: 'temperature' does not support 0.7 with this model.
    Only the default (1) value is supported."
  }
}
```

**원인:** `src/main.rs:222`에서 agent 서브커맨드의 `--temperature` 기본값이 `0.7`로 하드코딩되어 있어, config의 `default_temperature = 1.0`을 무시함. gpt-5-mini는 temperature=1.0만 지원.

```rust
// src/main.rs:222
#[arg(short, long, default_value = "0.7", value_parser = parse_temperature)]
temperature: f64,
```

**daemon과의 차이:** daemon 모드는 CLI 인자 없이 config의 `default_temperature`를 직접 사용하므로 영향 없음. agent 모드만 CLI 기본값(0.7)이 config보다 우선 적용됨.

## 수정

`lisa-agent.sh`에서 `zeroclaw agent` 호출 시 `-t 1.0`을 명시.

```bash
# Before
exec /home/root/lisa/zeroclaw agent -m "$*"
exec /home/root/lisa/zeroclaw agent

# After
exec /home/root/lisa/zeroclaw agent -t 1.0 -m "$*"
exec /home/root/lisa/zeroclaw agent -t 1.0
```

## Changes

| # | Path | Description |
|---|---|---|
| 1 | `lisa/scripts/deploy-target.sh` | agent 스크립트에 `-t 1.0` 추가, temperature 주석 추가 |
| 2 | `lisa/scripts/deploy-target.ps1` | 동일 변경 |
| 3 | `lisa/docs/deploy-target-guide.md` | 모드 비교 표에 temperature 열 추가, agent 참고 설명, 트러블슈팅 항목 추가 |
