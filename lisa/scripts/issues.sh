#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob

# ─────────────────────────────────────────────
# Lisa Open Issues 관리 스크립트
# 사용법: ./lisa/scripts/issues.sh <command> [options]
# ─────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LISA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ISSUES_DIR="$LISA_DIR/open-issues"

# ── 유틸리티 함수 ──

# frontmatter에서 특정 필드 값 추출
get_field() {
    local file="$1" field="$2"
    sed -n '/^---$/,/^---$/p' "$file" | grep "^${field}:" | sed "s/^${field}: *//"
}

# 다음 이슈 ID 계산 (3자리 zero-pad)
next_id() {
    local max_id=0
    for f in "$ISSUES_DIR"/*.md; do
        [ "$f" = "$ISSUES_DIR/README.md" ] && continue
        [ ! -f "$f" ] && continue
        local id
        id=$(get_field "$f" "id" 2>/dev/null || echo "0")
        id=$((10#$id))  # 앞의 0 제거
        [ "$id" -gt "$max_id" ] && max_id="$id"
    done
    printf "%03d" $((max_id + 1))
}

# ID로 이슈 파일 찾기
find_issue() {
    local id
    id=$(printf "%03d" $((10#$1)))
    local found=""
    for f in "$ISSUES_DIR"/${id}-*.md; do
        [ -f "$f" ] && found="$f" && break
    done
    if [ -z "$found" ]; then
        echo "❌ 이슈 #${id}를 찾을 수 없습니다." >&2
        exit 1
    fi
    echo "$found"
}

# ── 커맨드: new ──

cmd_new() {
    local id
    id=$(next_id)
    local today
    today=$(date +%Y-%m-%d)

    echo "📝 새 이슈 생성 (#${id})"
    echo ""

    read -rp "제목: " title
    if [ -z "$title" ]; then
        echo "❌ 제목은 필수입니다."
        exit 1
    fi

    echo ""
    echo "우선순위:"
    echo "  1) high"
    echo "  2) medium"
    echo "  3) low"
    read -rp "선택 [1-3, 기본=2]: " pri_choice
    case "${pri_choice:-2}" in
        1) priority="high" ;;
        3) priority="low" ;;
        *) priority="medium" ;;
    esac

    echo ""
    echo "카테고리:"
    echo "  1) bug"
    echo "  2) feature"
    echo "  3) improvement"
    echo "  4) config"
    read -rp "선택 [1-4, 기본=1]: " cat_choice
    case "${cat_choice:-1}" in
        2) category="feature" ;;
        3) category="improvement" ;;
        4) category="config" ;;
        *) category="bug" ;;
    esac

    # 파일명용 slug 생성 (공백 → 하이픈, ASCII 이외 및 특수문자 제거)
    local slug
    slug=$(echo "$title" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | sed 's/[^a-z0-9-]//g' | sed 's/--*/-/g' | sed 's/^-//;s/-$//')
    [ -z "$slug" ] && slug="issue"

    local filename="${id}-${slug}.md"
    local filepath="$ISSUES_DIR/$filename"

    echo ""
    read -rp "설명 (한 줄, 엔터로 건너뜀): " description

    cat > "$filepath" << EOF
---
id: ${id}
title: ${title}
status: open
priority: ${priority}
category: ${category}
created: ${today}
updated: ${today}
---

## 설명

${description:-TODO: 상세 설명 추가}

## 현재 워크어라운드



## 해결 방안

EOF

    echo ""
    echo "✅ 이슈 생성: $filename"
    echo "   📂 $filepath"
}

# ── 커맨드: list ──

cmd_list() {
    local filter_status="open"
    local filter_priority=""
    local filter_category=""

    while [ $# -gt 0 ]; do
        case "$1" in
            --all) filter_status="" ;;
            --priority) shift; filter_priority="$1" ;;
            --category) shift; filter_category="$1" ;;
            *) echo "❌ 알 수 없는 옵션: $1"; exit 1 ;;
        esac
        shift
    done

    local count=0

    # 헤더
    printf "  %-5s %-8s %-10s %-12s %s\n" "ID" "STATUS" "PRIORITY" "CATEGORY" "TITLE"
    printf "  %-5s %-8s %-10s %-12s %s\n" "-----" "--------" "----------" "------------" "-----"

    for f in "$ISSUES_DIR"/*.md; do
        [ "$f" = "$ISSUES_DIR/README.md" ] && continue
        [ ! -f "$f" ] && continue

        local id status priority category title
        id=$(get_field "$f" "id")
        status=$(get_field "$f" "status")
        priority=$(get_field "$f" "priority")
        category=$(get_field "$f" "category")
        title=$(get_field "$f" "title")

        # 필터 적용
        [ -n "$filter_status" ] && [ "$status" != "$filter_status" ] && continue
        [ -n "$filter_priority" ] && [ "$priority" != "$filter_priority" ] && continue
        [ -n "$filter_category" ] && [ "$category" != "$filter_category" ] && continue

        # 상태 아이콘
        local icon="🔴"
        [ "$status" = "closed" ] && icon="✅"

        printf "  %-5s %s %-6s %-10s %-12s %s\n" "#${id}" "$icon" "$status" "$priority" "$category" "$title"
        count=$((count + 1))
    done

    echo ""
    echo "  총 ${count}건"
}

# ── 커맨드: show ──

cmd_show() {
    [ -z "${1:-}" ] && echo "❌ 사용법: issues.sh show <id>" && exit 1
    local filepath
    filepath=$(find_issue "$1")
    echo ""
    cat "$filepath"
}

# ── 커맨드: close ──

cmd_close() {
    [ -z "${1:-}" ] && echo "❌ 사용법: issues.sh close <id>" && exit 1
    local filepath
    filepath=$(find_issue "$1")
    local today
    today=$(date +%Y-%m-%d)

    local current_status
    current_status=$(get_field "$filepath" "status")
    if [ "$current_status" = "closed" ]; then
        echo "⚠️  이슈 #$(printf "%03d" $((10#$1)))는 이미 closed 상태입니다."
        exit 0
    fi

    sed -i "s/^status: open/status: closed/" "$filepath"
    sed -i "s/^updated: .*/updated: ${today}/" "$filepath"

    local title
    title=$(get_field "$filepath" "title")
    echo "✅ 이슈 #$(printf "%03d" $((10#$1))) closed: $title"
}

# ── 커맨드: reopen ──

cmd_reopen() {
    [ -z "${1:-}" ] && echo "❌ 사용법: issues.sh reopen <id>" && exit 1
    local filepath
    filepath=$(find_issue "$1")
    local today
    today=$(date +%Y-%m-%d)

    local current_status
    current_status=$(get_field "$filepath" "status")
    if [ "$current_status" = "open" ]; then
        echo "⚠️  이슈 #$(printf "%03d" $((10#$1)))는 이미 open 상태입니다."
        exit 0
    fi

    sed -i "s/^status: closed/status: open/" "$filepath"
    sed -i "s/^updated: .*/updated: ${today}/" "$filepath"

    local title
    title=$(get_field "$filepath" "title")
    echo "🔄 이슈 #$(printf "%03d" $((10#$1))) reopened: $title"
}

# ── 커맨드: delete ──

cmd_delete() {
    [ -z "${1:-}" ] && echo "❌ 사용법: issues.sh delete <id>" && exit 1
    local filepath
    filepath=$(find_issue "$1")

    local title
    title=$(get_field "$filepath" "title")
    local id_str
    id_str=$(printf "%03d" $((10#$1)))

    read -rp "🗑️  이슈 #${id_str} '${title}' 삭제? [y/N]: " confirm
    if [ "${confirm:-N}" = "y" ] || [ "${confirm:-N}" = "Y" ]; then
        rm "$filepath"
        echo "🗑️  이슈 #${id_str} 삭제 완료"
    else
        echo "취소됨"
    fi
}

# ── 커맨드: summary ──

cmd_summary() {
    local total=0 open=0 closed=0
    local high=0 medium=0 low=0
    local bugs=0 features=0 improvements=0 configs=0 others=0

    for f in "$ISSUES_DIR"/*.md; do
        [ "$f" = "$ISSUES_DIR/README.md" ] && continue
        [ ! -f "$f" ] && continue

        total=$((total + 1))

        local status priority category
        status=$(get_field "$f" "status")
        priority=$(get_field "$f" "priority")
        category=$(get_field "$f" "category")

        case "$status" in
            open) open=$((open + 1)) ;;
            closed) closed=$((closed + 1)) ;;
        esac

        case "$priority" in
            high) high=$((high + 1)) ;;
            medium) medium=$((medium + 1)) ;;
            low) low=$((low + 1)) ;;
        esac

        case "$category" in
            bug) bugs=$((bugs + 1)) ;;
            feature) features=$((features + 1)) ;;
            improvement) improvements=$((improvements + 1)) ;;
            config) configs=$((configs + 1)) ;;
            *) others=$((others + 1)) ;;
        esac
    done

    echo ""
    echo "📊 이슈 요약"
    echo "  ────────────────────────"
    echo "  전체: ${total}건"
    echo ""
    echo "  상태별:"
    echo "    🔴 open:   ${open}"
    echo "    ✅ closed: ${closed}"
    echo ""
    echo "  우선순위별:"
    echo "    🔥 high:   ${high}"
    echo "    🔶 medium: ${medium}"
    echo "    🔷 low:    ${low}"
    echo ""
    echo "  카테고리별:"
    if [ "$bugs" -gt 0 ]; then echo "    🐛 bug:         ${bugs}"; fi
    if [ "$features" -gt 0 ]; then echo "    ✨ feature:      ${features}"; fi
    if [ "$improvements" -gt 0 ]; then echo "    🔧 improvement: ${improvements}"; fi
    if [ "$configs" -gt 0 ]; then echo "    ⚙️  config:      ${configs}"; fi
    if [ "$others" -gt 0 ]; then echo "    📋 other:       ${others}"; fi
    if [ "$total" -eq 0 ]; then echo "    (이슈 없음)"; fi
}

# ── 사용법 ──

usage() {
    echo "📋 Lisa Open Issues 관리"
    echo ""
    echo "사용법: $(basename "$0") <command> [options]"
    echo ""
    echo "Commands:"
    echo "  new                  새 이슈 생성 (대화형)"
    echo "  list [options]       이슈 목록 조회"
    echo "    --all              closed 포함 전체 표시"
    echo "    --priority <p>     우선순위 필터 (high/medium/low)"
    echo "    --category <c>     카테고리 필터 (bug/feature/improvement/config)"
    echo "  show <id>            이슈 상세 보기"
    echo "  close <id>           이슈 닫기"
    echo "  reopen <id>          이슈 다시 열기"
    echo "  delete <id>          이슈 삭제"
    echo "  summary              전체 요약 통계"
    echo ""
    echo "예시:"
    echo "  $(basename "$0") new"
    echo "  $(basename "$0") list"
    echo "  $(basename "$0") list --priority high"
    echo "  $(basename "$0") close 1"
    echo "  $(basename "$0") summary"
}

# ── 메인 ──

COMMAND="${1:-}"
shift 2>/dev/null || true

case "$COMMAND" in
    new)     cmd_new ;;
    list)    cmd_list "$@" ;;
    show)    cmd_show "$@" ;;
    close)   cmd_close "$@" ;;
    reopen)  cmd_reopen "$@" ;;
    delete)  cmd_delete "$@" ;;
    summary) cmd_summary ;;
    *)       usage ;;
esac
