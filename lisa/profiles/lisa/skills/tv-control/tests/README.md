# TV Control Skill — E2E Benchmark Test

TV 제어 스킬의 3가지 로딩 전략(TOML / MD always / MD on-demand)을 비교하는 벤치마크 테스트.

## 개요

| 시나리오 | 스킬 파일 | 로딩 방식 | 기대 동작 |
|---------|----------|----------|----------|
| S1 | SKILL.toml | Native tool calling | LLM이 `tv_set_volume` 직접 호출 |
| S2 | SKILL.md (always=true) | System prompt injection | LLM이 shell tool로 명령 실행 |
| S3 | SKILL.md (always=false) | On-demand (read_skill) | LLM이 스킬 읽고 → shell tool 실행 |

## 성공 판정

`mock-tv.sh`가 **실제로 호출됐는지** (`/tmp/mock-tv-calls.log` 줄 수)로 판정.
- `mock > 0` = 성공 (스크립트 실행됨)
- `mock = 0` = 실패 (호출 안 됨 또는 잘못된 명령 실행)

## 사전 준비

```bash
# 1. ZeroClaw 빌드 (tool_handler current_dir fix 포함)
cd ~/project/lisa
cargo build

# 2. .env 설정
cat .env
# ZEROCLAW_PROVIDER=gemini          # 또는 azure
# ZEROCLAW_MODEL=gemini-2.5-pro     # 또는 gpt-4o 등
# GEMINI_API_KEY=...                # 또는 AZURE_OPENAI_KEY=...

# 3. allowed_commands에 mock-tv.sh 추가 (config.default.toml)
# allowed_commands = [..., "mock-tv.sh"]
```

## 실행

```bash
# 기본 (10회, 결과 → /tmp/tv-skill-benchmark)
bash skills/tv-control/tests/run.sh

# 결과 디렉토리 + 횟수 지정
bash skills/tv-control/tests/run.sh /tmp/tv-bench-pro 5

# 다른 모델로
ZEROCLAW_MODEL=gemini-2.5-flash bash skills/tv-control/tests/run.sh /tmp/tv-bench-flash 10
```

## 출력 예시

```
========================================
 TV Control 스킬 E2E 벤치마크
 모델: gemini-2.5-pro
 각 시나리오 10회
========================================

▶ S1: SKILL.toml (native tool call)
  Run 1: exec=1 ok=2 fail=0 mock=1 blocked=0 toml=2 native=8
  ...

--- 시나리오별 성공률 ---
  s1_toml: 10/10
  s2_md_always: 10/10
  s3_md_ondemand: 7/10
```

## 결과 파일

```
/tmp/tv-skill-benchmark/
├── summary.csv                ← 전체 결과 CSV
├── s1_toml_run1.txt           ← 각 Run의 전체 로그
├── s1_toml_run2.txt
├── s2_md_always_run1.txt
└── ...
```

### CSV 컬럼

| 컬럼 | 설명 |
|------|------|
| scenario | s1_toml / s2_md_always / s3_md_ondemand |
| run | 실행 번호 |
| resp_chars | 전체 출력 크기 (bytes) |
| tool_exec | Tool call executing 횟수 |
| success | success=true 횟수 |
| fail | success=false 횟수 |
| **mock_calls** | **mock-tv.sh 실제 호출 횟수 (핵심 지표)** |
| blocked | 보안 정책 차단 횟수 |
| toml_loaded | TOML 스킬 로딩 횟수 |
| native_reg | Native tool 등록 횟수 |

## 주의사항

- **메모리 리셋**: 매 Run마다 자동 리셋 (이전 실패가 다음 Run에 영향 방지)
- **mock-tv.sh 호출 로그**: `/tmp/mock-tv-calls.log` (매 Run 초기화)
- **원복**: 테스트 종료 후 원본 SKILL.toml/SKILL.md 자동 복구
- **macOS 전용**: `sed -i ''` 사용 (Linux는 `sed -i` 수정 필요)

## 기존 결과 (2026-03-17)

| | S1: TOML | S2: MD always | S3: MD ondemand |
|---|---|---|---|
| **Pro** | 9/10 | **10/10** | 7/10 |
| **Flash** | **10/10** | **10/10** | 8/10 |
| **Flash Lite** | **10/10** | 1/10 | 0/10 |

> Pro S1 1회 실패 = Gemini API 503 에러 (TOML 자체 문제 아님)
> Flash Lite S2/S3 실패 = 프롬프트 읽고도 잘못된 명령 실행 (모델 한계)
