# 스킬 모드 벤치마크 가이드

`benchmark-skill-mode.sh`는 SKILL.md 모드와 SKILL.toml 모드의 응답 시간 및 LLM 턴 수를 비교 측정합니다.
데몬 gateway API(`/api/chat`)를 통해 실제 사용자 경험과 동일한 조건에서 측정합니다.

## 사용법

```bash
# 로컬 벤치마크 (10회, 기본값)
lisa/test/benchmark-skill-mode.sh

# 타겟에서 벤치마크
lisa/test/benchmark-skill-mode.sh --target 192.168.0.10

# 횟수 지정
lisa/test/benchmark-skill-mode.sh --runs 50

# 조합
lisa/test/benchmark-skill-mode.sh --target 192.168.0.10 --runs 50
```

| 옵션 | 기본값 | 설명 |
|---|---|---|
| `--target <IP>` | (없음 = 로컬) | SSH 원격 타겟 IP |
| `--runs <N>` | 10 | 스킬당/모드당 측정 횟수 |

## 전제 조건

- `onboard.sh` 설치 완료 (바이너리 + 스킬 + config + .env)
- 타겟 사용 시 SSH 키 인증 설정
- non-webOS 환경에서는 mock `luna-send` 설치 (`onboard.sh`가 심링크로 자동 설치)

## 측정 항목

| 항목 | 방법 |
|---|---|
| 응답 시간 (ms) | `curl -w '%{time_total}'`로 gateway 요청~응답 시간 측정 |
| LLM 턴 수 | runtime trace JSONL의 `llm_response` 이벤트 카운트 |
| 에러 | content filter / provider exhaustion 감지 |

### 테스트 스킬

| 스킬 | 쿼리 | 목적 |
|---|---|---|
| weather | 서울 현재 날씨 API 호출 요청 | 매 실행마다 외부 도구 호출 강제 |
| tv-control | 볼륨 8 / 10 교대 설정 | 값 교대로 매 실행마다 도구 호출 강제 |

## 동작 흐름

```
┌─────────────────────────────────────────────────┐
│  1. config 배포 + runtime trace 활성화          │
├─────────────────────────────────────────────────┤
│  2. Phase 1: SKILL.md 모드                      │
│     - 스킬 배포                                 │
│     - SKILL.toml 파일 삭제                      │
│     - 데몬 시작                                 │
│     - Warm-up (1회, 결과 무시)                  │
│     - weather × N회 측정                        │
│     - tv-control × N회 측정                     │
├─────────────────────────────────────────────────┤
│  3. Phase 2: SKILL.toml 모드                    │
│     - 스킬 재배포 (SKILL.toml 포함)             │
│     - 데몬 시작                                 │
│     - Warm-up (1회, 결과 무시)                  │
│     - weather × N회 측정                        │
│     - tv-control × N회 측정                     │
├─────────────────────────────────────────────────┤
│  4. 최종 비교 리포트 출력                       │
│  5. 원본 config 복원 (runtime trace 비활성화)   │
└─────────────────────────────────────────────────┘
```

## 출력

### 개별 실행 결과

각 실행마다 응답 시간과 턴 수를 출력합니다:

```
  [weather] 10 runs...
    # 1 :  6273 ms  (2 turns)
    # 2 :  7150 ms  (2 turns)
    ...
```

### 최종 리포트

스킬별로 두 모드를 나란히 비교합니다:

```
  ── weather ──────────────────────────────────────────────────
   Run      SKILL.md    turns    SKILL.toml    turns
  ──────────────────────────────────────────────────────────────
     1     14409 ms  4 turns      6163 ms  2 turns
     2      6273 ms  2 turns      6580 ms  2 turns
     ...
  ──────────────────────────────────────────────────────────────
   avg      8957 ms  2.5 turns      6554 ms  2.0 turns

  ── Comparison ──────────────────────────────────────────────────
  Skill              SKILL.md      turns   SKILL.toml      turns  Diff (ms)
  ────────────────────────────────────────────────────────────────
  weather             8957 ms  2.5 turns      6554 ms  2.0 turns  SKILL.toml faster by 2403ms
  tv-control          6369 ms  2.0 turns      5001 ms  2.0 turns  SKILL.toml faster by 1368ms
```

## 설정

### Gateway 포트

`config.default.toml`의 `[gateway] port` 값을 자동으로 읽습니다. 별도 포트 설정 불필요.

### Runtime trace

벤치마크 중 턴 수 측정을 위해 runtime trace를 일시적으로 활성화하고, 종료 시 (에러 포함) 원본 config를 복원합니다.

## 로컬 vs 타겟 모드

| | 로컬 | 타겟 (`--target`) |
|---|---|---|
| 데몬 실행 위치 | localhost | SSH 원격 호스트 |
| 스킬 배포 방법 | `onboard.sh --skills` | `onboard.sh --skills --target <IP>` |
| luna-send | Mock (심링크) | 실제 webOS 바이너리 |
| Gateway 접근 | 직접 `curl` | SSH 경유 `curl` |
| mock/ 디렉토리 | 스킬에 포함 | 제외 (타겟에 실제 luna-send 있음) |

## 문제 해결

### 모든 turns가 0으로 나옴

다른 사용자의 데몬이 gateway 포트를 점유하고 있을 수 있습니다:

```bash
curl -sf http://127.0.0.1:42617/health | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['runtime']['pid'])"
ps -p <PID> -o user,cmd
```

해당 데몬을 종료하고 벤치마크를 다시 실행하세요.

### /tmp/zeroclaw.log 권한 오류

다른 사용자의 로그 파일이 남아있는 경우입니다. 로컬 모드에서는 PID 기반 파일명을 사용하지만, 문제가 발생하면 삭제하세요:

```bash
sudo rm /tmp/zeroclaw.log
```
