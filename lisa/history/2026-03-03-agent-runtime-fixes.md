# 2026-03-03: agent 모드 런타임 에러 수정

## Summary

타겟에서 agent 모드 실행 시 발생한 2건의 에러 수정.

## 문제 1: 스킬 스크립트 차단

**에러:**
```
WARN zeroclaw::skills: skipping insecure skill directory .../device-control:
scripts/go-to-channel.sh: script-like files are blocked by skill security policy.
```

**원인:** `allow_scripts` 기본값이 `false`로, `.sh` 파일이 포함된 스킬 디렉토리가 보안 정책에 의해 차단됨.

**수정:** `config.arm64.toml`에 `[skills] allow_scripts = true` 추가.

**관련 코드:** `src/skills/audit.rs:303-307`, `src/config/schema.rs:1250-1253`

## 문제 2: Azure OpenAI 스키마 에러

**에러:**
```
Invalid schema for function 'channel_ack_config':
In context=('properties', 'rules', 'type', '0'), array schema missing items.
```

**원인:** `channel_ack_config` 도구의 JSON Schema에서 `rules` 필드가 `"type": ["array", "null"]`로 정의되어 있으나, Azure OpenAI는 array 타입에 `items` 속성이 필수.

**수정:** `src/tools/channel_ack_config.rs:619`를 `emojis`/`defaults` 필드와 동일한 `anyOf` 패턴으로 변경.

```rust
// Before
"rules": {"type": ["array", "null"]},

// After
"rules": {
    "anyOf": [
        {"type": "array", "items": {"type": "object"}},
        {"type": "null"}
    ]
},
```

**테스트:** `cargo test --lib channel_ack_config` — 5개 테스트 전부 통과.

## Changes

| # | Path | Description |
|---|---|---|
| 1 | `lisa/config/config.arm64.toml` | `[skills] allow_scripts = true` 추가 |
| 2 | `src/tools/channel_ack_config.rs` | `rules` 스키마에 `items` 추가 (Azure OpenAI 호환) |
| 3 | `lisa/docs/deploy-target-guide.md` | config 예시에 `[skills]` 추가, 트러블슈팅에 2건 추가 |
