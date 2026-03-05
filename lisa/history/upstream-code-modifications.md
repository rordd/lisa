# upstream 코드 수정 이력

upstream ZeroClaw 소스코드에 가한 수정 사항 기록.
Lisa 프로젝트 전용 파일(profiles/, scripts/, docs/)은 `lisa/` 디렉토리로 이동 완료 — 여기서는 **src/ 등 upstream 코드 수정**만 다룸.

---

## Commit 1: `495d6324` — feat(profiles): add Lisa profile and setup script

**Date**: 2026-03-02 05:44:16

### `.gitignore` 수정

- `.env` 항목 추가 (시크릿 파일 Git 추적 방지)

```diff
+.env
```

---

## Commit 2: `13cdca54` — feat(profiles): make model/provider configurable per user via .env

**Date**: 2026-03-02 06:52:13

### 수정 파일 (lisa/ 이동 완료)

- `profiles/.env.example` — `ZEROCLAW_PROVIDER`, `ZEROCLAW_MODEL` 환경변수 추가
- `profiles/lisa/config.shared.toml` — provider/model을 환경변수 참조로 변경
- `scripts/setup-lisa.sh` — `.env`에서 provider/model 읽어서 config에 주입하도록 변경

---

## Commit 3: `2a2c4cf0` — feat(providers): add auth_header config for Azure OpenAI support

**Date**: 2026-03-02 14:43:52

Azure OpenAI의 비표준 인증 헤더(`api-key: <key>`) 지원을 위한 핵심 변경.

### `src/config/schema.rs`

- `Config` 구조체에 `custom_provider_auth_header: Option<String>` 필드 추가 (`#[serde(skip)]`)
- `ModelProviderConfig` 구조체에 `auth_header: Option<String>` 필드 추가
  - `None` or `"bearer"` → `Authorization: Bearer <key>` (기본)
  - `"api-key"` → `api-key: <key>` (Azure OpenAI)
  - `"x-api-key"` → `x-api-key: <key>` (Anthropic 스타일)
- `Config::apply_env_overrides()`에서 profile의 `auth_header` → `custom_provider_auth_header`로 전파 로직 추가
- `Default for Config` 및 테스트 내 Config 리터럴에 `custom_provider_auth_header: None` 추가
- 테스트 내 `ModelProviderConfig` 리터럴에 `auth_header: None` 추가
- 신규 테스트 2개 추가:
  - `model_provider_profile_propagates_auth_header` — auth_header 전파 검증
  - `model_provider_profile_auth_header_none_leaves_default` — None일 때 기본값 유지 검증

### `src/providers/mod.rs`

- `ProviderRuntimeOptions`에 `custom_provider_auth_header: Option<String>` 필드 추가
- `resolve_auth_style()` 함수 신규 추가 — auth_header 문자열을 `AuthStyle` enum으로 변환
- `create_provider_with_url_and_options()`에서 custom provider 생성 시 `resolve_auth_style()` 적용
- 신규 테스트 4개 추가:
  - `resolve_auth_style_none_defaults_to_bearer`
  - `resolve_auth_style_bearer_string`
  - `resolve_auth_style_x_api_key`
  - `resolve_auth_style_custom_header`

### `src/providers/compatible.rs`

- `chat_completions_url()` 메서드 수정 — base_url에 query parameter가 있을 때 보존 (Azure `?api-version=...`)
- 신규 테스트 3개 추가:
  - `chat_completions_url_preserves_query_params`
  - `chat_completions_url_preserves_query_when_already_has_endpoint`
  - `chat_completions_url_no_query_unchanged`

### `src/agent/loop_.rs`

- `ProviderRuntimeOptions` 생성 시 `custom_provider_auth_header` 필드 전달 (2개소)

### `src/channels/mod.rs`

- `ProviderRuntimeOptions` 생성 시 `custom_provider_auth_header` 필드 전달

### `src/gateway/mod.rs`

- `ProviderRuntimeOptions` 생성 시 `custom_provider_auth_header` 필드 전달

### `src/onboard/wizard.rs`

- Config 리터럴 2개소에 `custom_provider_auth_header: None` 추가

### `src/providers/openai_codex.rs`

- 테스트 내 `ProviderRuntimeOptions` 리터럴에 `custom_provider_auth_header: None` 추가

### `src/tools/mod.rs`

- `ProviderRuntimeOptions` 생성 시 `custom_provider_auth_header` 필드 전달

### `tests/openai_codex_vision_e2e.rs`

- `ProviderRuntimeOptions` 리터럴에 `custom_provider_auth_header: None` 추가

---

## Commit 4: `3cedb677` — feat(setup): auto-inject Azure OpenAI profile from .env

**Date**: 2026-03-02 15:25:54

### 수정 파일 (lisa/ 이동 완료)

- `docs/setup-guide.md` — Azure OpenAI 수동 config 섹션 간소화, 자동 주입 설명 추가
- `profiles/.env.example` — Azure OpenAI 관련 환경변수 추가 (`AZURE_OPENAI_BASE_URL`, `AZURE_OPENAI_API_KEY`)
- `scripts/setup-lisa.sh` — Azure OpenAI 프로필 자동 주입 로직 추가 (`[model_providers.azure]` 섹션)

---

## Commit 5: `a3ee6fb1` — feat(skills): add weather and calendar skills

**Date**: 2026-03-02 18:31:31

### 수정 파일 (lisa/ 이동 완료)

- `scripts/setup-lisa.sh` — 스킬 디렉토리 복사 로직 추가 (섹션 6)
- `profiles/lisa/skills/calendar/SKILL.md` — 캘린더 스킬 정의 (신규)
- `profiles/lisa/skills/weather/SKILL.md` — 날씨 스킬 정의 (신규)

---

## CLAUDE.md 수정 이력

upstream에서 `CLAUDE.md`에 가한 수정 (4개 커밋, 순차적 진화):

| Commit | Date | Subject | 변경 내용 |
|--------|------|---------|-----------|
| `9b55a6d5` | 2026-03-02 02:59 | docs: add Project Lisa context to CLAUDE.md | Lisa 프로젝트 개요, 로드맵, 아키텍처 노트, 삽질 교훈 섹션 추가 |
| `11fce700` | 2026-03-02 03:00 | docs: add collaboration workflow to CLAUDE.md | 협업 체계 섹션 추가 (집/회사 환경, 동기화 규칙) |
| `6b550092` | 2026-03-02 03:02 | docs: strip project context from CLAUDE.md | 프로젝트 개요, 로드맵, 아키텍처 노트, WS 멀티턴 등 대부분 삭제 (간소화) |
| `29ecf8ed` | 2026-03-02 03:04 | docs: remove lessons section from CLAUDE.md | 삽질 교훈 섹션 삭제 |

**최종 상태**: CLAUDE.md에는 협업 체계 + 동기화 규칙만 남음 (현재 유지 중)

---

## 영향받은 upstream 파일 요약

| 파일 | 수정 유형 | 커밋 |
|------|-----------|------|
| `.gitignore` | `.env` 항목 추가 | `495d6324` |
| `src/config/schema.rs` | `auth_header` 필드 + 전파 로직 + 테스트 | `2a2c4cf0` |
| `src/providers/mod.rs` | `resolve_auth_style()` + `ProviderRuntimeOptions` 확장 + 테스트 | `2a2c4cf0` |
| `src/providers/compatible.rs` | query param 보존 로직 + 테스트 | `2a2c4cf0` |
| `src/agent/loop_.rs` | `custom_provider_auth_header` 전달 (2개소) | `2a2c4cf0` |
| `src/channels/mod.rs` | `custom_provider_auth_header` 전달 | `2a2c4cf0` |
| `src/gateway/mod.rs` | `custom_provider_auth_header` 전달 | `2a2c4cf0` |
| `src/onboard/wizard.rs` | Config 리터럴에 필드 추가 (2개소) | `2a2c4cf0` |
| `src/providers/openai_codex.rs` | 테스트 리터럴에 필드 추가 | `2a2c4cf0` |
| `src/tools/mod.rs` | `custom_provider_auth_header` 전달 | `2a2c4cf0` |
| `tests/openai_codex_vision_e2e.rs` | 테스트 리터럴에 필드 추가 | `2a2c4cf0` |
| `CLAUDE.md` | Lisa 섹션 추가/축소/정리 | `9b55a6d5`, `11fce700`, `6b550092`, `29ecf8ed` |
