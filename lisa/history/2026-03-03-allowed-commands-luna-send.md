# 2026-03-03: allowed_commands에 luna-send 추가

## Summary

device-control 스킬에서 `luna-send` 명령을 실행할 수 있도록 `allowed_commands`에 추가.

## 문제

device-control 스킬(SKILL.md)이 `luna-send`를 사용하여 webOS TV API를 호출하지만, `allowed_commands`에 등록되어 있지 않아 shell 실행이 차단됨.

**스킬에서 사용하는 luna-send 호출 예시:**
```bash
luna-send -n 1 luna://com.webos.applicationManager/launch '{"id":"com.webos.app.hdmi1"}'
luna-send -n 1 luna://com.webos.service.audio/master/setVolume '{"volume":15}'
luna-send -n 1 -f luna://com.lge.inputgenerator/pushKeyEvent '{"eventtype":"key","keycodenum":402}'
```

## 수정

`config.arm64.toml`의 `allowed_commands`에 `luna-send` 추가.

```toml
# Before
allowed_commands = ["git", "npm", "cargo", "ls", "cat", "grep", "find", "echo", "pwd", "wc", "head", "tail", "date", "gog", "memo", "remindctl", "jq", "python3", "bash"]

# After
allowed_commands = ["git", "npm", "cargo", "ls", "cat", "grep", "find", "echo", "pwd", "wc", "head", "tail", "date", "gog", "memo", "remindctl", "jq", "python3", "bash", "luna-send"]
```

## Changes

| # | Path | Description |
|---|---|---|
| 1 | `lisa/config/config.arm64.toml` | `allowed_commands`에 `luna-send` 추가 |
