# 2026-03-03: 타겟 배포 스크립트 및 설정

## Summary

webOS TV (ARM32) 타겟에 Lisa를 배포하기 위한 스크립트, 설정, 가이드 문서 추가.
192.168.0.10 타겟에 배포 및 Azure OpenAI 접속 테스트 완료.

## Changes

### Created

| # | Path | Description |
|---|---|---|
| 1 | `lisa/config/config.arm32.toml` | ARM32 타겟용 완전한 config (provider, model, memory, security 포함) |
| 2 | `lisa/config/config.default.toml` | 기본/로컬용 config (`config.shared.toml`에서 이동 및 이름 변경) |
| 3 | `lisa/scripts/deploy-target.sh` | Linux/macOS용 타겟 배포 스크립트 |
| 4 | `lisa/scripts/deploy-target.ps1` | Windows PowerShell용 타겟 배포 스크립트 |
| 5 | `lisa/docs/deploy-target-guide.md` | 타겟 배포 가이드 문서 |
| 6 | `lisa/release/arm32/zeroclaw` | ARM32 바이너리 (별도 빌드) |

### Modified

| # | Path | Description |
|---|---|---|
| 1 | `lisa/scripts/setup-lisa.sh` | config 경로 변경: `profiles/lisa/config.shared.toml` → `config/config.default.toml` |
| 2 | `lisa/docs/setup-guide.md` | 프로젝트 구조, config 경로 업데이트 |

### Deleted / Moved

| # | From | To | Description |
|---|---|---|---|
| 1 | `lisa/profiles/lisa/config.shared.toml` | `lisa/config/config.default.toml` | config 디렉토리로 이동 및 이름 변경 |
| 2 | `lisa/config/config.toml` | (삭제) | `config.arm32.toml`로 대체 |

## Config 구조

```
lisa/config/
├── config.default.toml    # 기본/로컬용 (setup-lisa.sh에서 사용)
└── config.arm32.toml      # webOS TV ARM32 타겟용 (deploy-target.sh에서 사용)
```

- 타겟별 config 파일을 독립적으로 관리
- 배포 시 타겟 config를 직접 전송 (머지 불필요)
- 새 타겟 추가 시 `config.<target>.toml` 파일 추가

## 배포 방식

- `.env` 미사용 — `config.arm32.toml`에 모든 설정 포함
- 타겟에 직접 전송: `config.arm32.toml` → `~/.zeroclaw/config.toml`
- Python3 기반 TOML 머지 불필요 (이전 방식에서 변경)

## 테스트 결과

- 배포 대상: 192.168.0.10 (webOS TV 11.1.0, ARM32)
- config.toml 전송: 성공
- /etc/hosts bind mount: 성공
- Azure OpenAI 접속: 성공 (gpt-5-mini-2025-08-07 모델 응답 확인)
