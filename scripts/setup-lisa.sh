#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROFILE_DIR="$REPO_DIR/profiles/lisa"
ZEROCLAW_DIR="${ZEROCLAW_CONFIG_DIR:-$HOME/.zeroclaw}"
WORKSPACE_DIR="$ZEROCLAW_DIR/workspace"
ENV_FILE="$REPO_DIR/.env"

echo "🎀 Lisa 프로필 셋업"
echo "===================="

# 1) 디렉토리 생성
mkdir -p "$ZEROCLAW_DIR" "$WORKSPACE_DIR"

# 2) .env 파일 확인 및 로드
if [ ! -f "$ENV_FILE" ]; then
    echo ""
    echo "⚠  .env 파일이 없습니다."
    echo "   cp profiles/.env.example .env"
    echo "   그리고 API 키를 채워넣으세요."
    exit 1
fi
# shellcheck disable=SC1090
source "$ENV_FILE"

# 3) 필수 값 확인
if [ -z "${ZEROCLAW_API_KEY:-}" ] && [ -z "${API_KEY:-}" ]; then
    echo "❌ ZEROCLAW_API_KEY가 설정되지 않았습니다. .env 파일을 확인하세요."
    exit 1
fi

# 4) config.toml 생성 (공유 설정 + 시크릿 주입)
if [ -f "$ZEROCLAW_DIR/config.toml" ]; then
    BACKUP="$ZEROCLAW_DIR/config.toml.bak.$(date +%s)"
    cp "$ZEROCLAW_DIR/config.toml" "$BACKUP"
    echo "📦 기존 config 백업: $BACKUP"
fi

cp "$PROFILE_DIR/config.shared.toml" "$ZEROCLAW_DIR/config.toml"

# 텔레그램 설정 주입
if [ -n "${TELEGRAM_BOT_TOKEN:-}" ]; then
    cat >> "$ZEROCLAW_DIR/config.toml" << EOF

[channels_config.telegram]
bot_token = "${TELEGRAM_BOT_TOKEN}"
allowed_users = [$(echo "${TELEGRAM_ALLOWED_USERS:-}" | sed 's/,/", "/g; s/^/"/; s/$/"/' )]
mention_only = ${TELEGRAM_MENTION_ONLY:-true}
EOF
    echo "✅ 텔레그램 채널 설정 완료"
fi

# Azure OpenAI 프로필 주입
if [ -n "${AZURE_OPENAI_BASE_URL:-}" ]; then
    AZURE_KEY="${AZURE_OPENAI_API_KEY:-${ZEROCLAW_API_KEY:-}}"
    cat >> "$ZEROCLAW_DIR/config.toml" << EOF

[model_providers.azure]
name = "openai"
base_url = "${AZURE_OPENAI_BASE_URL}"
auth_header = "api-key"
api_key = "${AZURE_KEY}"
EOF
    echo "✅ Azure OpenAI 프로필 설정 완료"
fi

chmod 600 "$ZEROCLAW_DIR/config.toml"

# 5) 공유 workspace 파일 복사
for f in SOUL.md AGENTS.md; do
    if [ -f "$PROFILE_DIR/$f" ]; then
        cp "$PROFILE_DIR/$f" "$WORKSPACE_DIR/$f"
        echo "✅ $f → workspace/"
    fi
done

# 6) 스킬 복사
if [ -d "$PROFILE_DIR/skills" ]; then
    mkdir -p "$WORKSPACE_DIR/skills"
    cp -r "$PROFILE_DIR/skills/"* "$WORKSPACE_DIR/skills/"
    SKILL_COUNT=$(ls -d "$PROFILE_DIR/skills/"*/ 2>/dev/null | wc -l | tr -d ' ')
    echo "✅ 스킬 ${SKILL_COUNT}개 → workspace/skills/"
fi

# 7) USER.md 확인
if [ ! -f "$WORKSPACE_DIR/USER.md" ]; then
    echo ""
    echo "📝 USER.md가 없습니다. 예시 파일을 참고해서 작성하세요:"
    echo "   cp $PROFILE_DIR/USER.md.example $WORKSPACE_DIR/USER.md"
    echo "   그리고 자신의 정보로 수정하세요."
fi

echo ""
echo "🎀 Lisa 셋업 완료!"
echo "   실행: source .env && zeroclaw daemon"
echo "   상태: zeroclaw status"
