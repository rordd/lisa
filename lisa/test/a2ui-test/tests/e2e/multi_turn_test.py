#!/usr/bin/env python3
"""
A2UI Multi-Turn E2E Test Suite

Tests various scenarios with multi-turn button interactions.
Detects common problems:
- LLM hallucinating capabilities (fake search, fake map pins)
- Infinite conversation loops (never providing actual content)
- Wrong action types (event for URLs instead of functionCall)
- Missing A2UI cards when expected
- Empty cards with no useful content
"""

import asyncio
import json
import sys
import time
from dataclasses import dataclass, field
from typing import Optional

try:
    import websockets
except ImportError:
    print("pip install websockets")
    sys.exit(1)

import uuid

WS_BASE = "ws://127.0.0.1:42617/ws/chat"
MAX_TURNS = 5
TIMEOUT = 120  # seconds per turn


@dataclass
class TurnResult:
    turn: int
    sent: str
    text: str
    a2ui_count: int
    components: list
    buttons: list
    data_model_keys: list
    elapsed_ms: float
    issues: list = field(default_factory=list)


@dataclass
class ScenarioResult:
    name: str
    initial_prompt: str
    turns: list
    issues: list = field(default_factory=list)
    passed: bool = True


# ── Issue detectors ──

def detect_issues(turn: TurnResult, all_turns: list) -> list:
    """Detect problems in a turn result."""
    issues = []

    # 1) No A2UI when expected (turn 1 should almost always have a card)
    if turn.turn == 1 and turn.a2ui_count == 0:
        issues.append("NO_CARD_ON_FIRST_TURN: LLM didn't generate A2UI card")

    # 2) Hallucinated capabilities in text
    hallucination_phrases = [
        "실시간 검색", "실시간으로", "검색해서 가져", "검색 진행",
        "잠깐만 기다려", "로딩 중", "검색 중",
        "핀을 표시", "핀으로 찍", "지도에 표시해줄", "핀셋으로 고정",
        "전화번호를 조회", "리뷰를 가져",
        "재생 요청 보냈", "재생을 일시정지", "다시 재생",
        "캘린더에 추가", "캘린더에 등록", "이벤트 생성",
        "찾아줄게", "찾아올게", "가져올게", "조회해줄게",
        "API 호출", "서버에 요청",
    ]
    for phrase in hallucination_phrases:
        if phrase in turn.text:
            issues.append(f"HALLUCINATION: '{phrase}' in response text")

    # 3) Buttons for non-existent capabilities
    hallucination_button_names = [
        "proceed_search", "execute_search", "confirm_find",
        "search_nearby", "실시간", "find_nearby",
        "show_map_pins", "load_reviews",
    ]
    for btn in turn.buttons:
        event = btn.get("action", {}).get("event", {})
        name = event.get("name", "")
        if any(h in name.lower() for h in hallucination_button_names):
            issues.append(f"HALLUCINATION_BUTTON: '{name}' — LLM can't do this")

    # 4) URL in event context (should be functionCall)
    for btn in turn.buttons:
        event = btn.get("action", {}).get("event", {})
        ctx = event.get("context", {})
        for v in ctx.values():
            if isinstance(v, str) and v.startswith("http"):
                issues.append(
                    f"WRONG_ACTION_TYPE: URL '{v[:60]}...' in event — should be functionCall.openUrl"
                )

    # 5) functionCall.openUrl — correct usage (not an issue, just track)
    for btn in turn.buttons:
        fc = btn.get("action", {}).get("functionCall", {})
        if fc.get("call") == "openUrl":
            # This is correct! Not an issue.
            pass

    # 6) Conversation loop detection
    if len(all_turns) >= 3:
        recent_texts = [t.text[:100] for t in all_turns[-3:]]
        # If all recent turns have similar confirmation patterns
        confirm_patterns = ["찾아줄까", "진행해도", "검색할까", "보여줄까"]
        loop_count = sum(
            1
            for text in recent_texts
            if any(p in text for p in confirm_patterns)
        )
        if loop_count >= 2:
            issues.append("CONVERSATION_LOOP: Multiple turns asking for confirmation without providing content")

    # 7) Empty content — card exists but no useful data
    if turn.a2ui_count > 0 and not turn.data_model_keys:
        issues.append("EMPTY_DATA_MODEL: Card has no data model")

    return issues


# ── WebSocket client ──

async def run_scenario(scenario_name: str, initial_prompt: str) -> ScenarioResult:
    """Run a single scenario with multi-turn interaction."""
    result = ScenarioResult(name=scenario_name, initial_prompt=initial_prompt, turns=[])

    try:
        # Use unique session_id per scenario to avoid memory pollution
        session_id = f"a2ui_test_{scenario_name}_{uuid.uuid4().hex[:8]}"
        ws_uri = f"{WS_BASE}?session_id={session_id}"
        async with websockets.connect(ws_uri, ping_interval=None, close_timeout=5) as ws:
            next_msg = json.dumps({"type": "message", "content": initial_prompt})
            all_turns = []

            for turn_num in range(1, MAX_TURNS + 1):
                start = time.time()
                await ws.send(next_msg)

                a2ui_msgs = []
                full_response = ""

                while True:
                    try:
                        resp = await asyncio.wait_for(ws.recv(), timeout=TIMEOUT)
                        data = json.loads(resp)
                        t = data.get("type", "")

                        if t == "a2ui":
                            a2ui_msgs = data.get("messages", [])
                        elif t == "done":
                            full_response = data.get("full_response", "")
                            break
                        elif t == "error":
                            result.issues.append(f"SERVER_ERROR: {data}")
                            result.passed = False
                            return result
                    except asyncio.TimeoutError:
                        result.issues.append(f"TIMEOUT on turn {turn_num}")
                        result.passed = False
                        return result

                elapsed = (time.time() - start) * 1000

                # Parse components and buttons
                components = []
                buttons = []
                dm_keys = []
                surface_id = None

                for msg in a2ui_msgs:
                    if msg.get("createSurface"):
                        surface_id = msg["createSurface"]["surfaceId"]
                    if msg.get("updateComponents"):
                        for c in msg["updateComponents"]["components"]:
                            components.append(c.get("component", "?"))
                            if c.get("component") == "Button":
                                buttons.append(c)
                    if msg.get("updateDataModel"):
                        val = msg["updateDataModel"].get("value", {})
                        if isinstance(val, dict):
                            dm_keys = list(val.keys())

                turn = TurnResult(
                    turn=turn_num,
                    sent=next_msg[:200],
                    text=full_response,
                    a2ui_count=len(a2ui_msgs),
                    components=components,
                    buttons=buttons,
                    data_model_keys=dm_keys,
                    elapsed_ms=elapsed,
                )

                all_turns.append(turn)
                turn.issues = detect_issues(turn, all_turns)
                result.turns.append(turn)

                if turn.issues:
                    result.issues.extend(
                        [f"Turn {turn_num}: {i}" for i in turn.issues]
                    )

                # Decide next action
                if not a2ui_msgs or not buttons:
                    break

                # Find first clickable button
                first_event_btn = None
                first_url_btn = None
                for btn in buttons:
                    fc = btn.get("action", {}).get("functionCall", {})
                    if fc.get("call") == "openUrl":
                        first_url_btn = btn
                        continue
                    ev = btn.get("action", {}).get("event")
                    if ev and not first_event_btn:
                        # Check for URL in context
                        ctx = ev.get("context", {})
                        urls = [
                            v
                            for v in ctx.values()
                            if isinstance(v, str) and v.startswith("http")
                        ]
                        if urls:
                            first_url_btn = btn
                        else:
                            first_event_btn = btn

                if first_url_btn:
                    # URL button found — this is a terminal action (browser opens)
                    fc = first_url_btn.get("action", {}).get("functionCall", {})
                    ev = first_url_btn.get("action", {}).get("event", {})
                    url = fc.get("args", {}).get("url", "")
                    if not url:
                        ctx = ev.get("context", {})
                        url = next(
                            (v for v in ctx.values() if isinstance(v, str) and v.startswith("http")),
                            "",
                        )
                    break  # Client would open URL, no more server turns needed

                if first_event_btn and surface_id:
                    ev = first_event_btn["action"]["event"]
                    ctx = dict(ev.get("context", {}))
                    # If TextField exists, inject a sample value
                    text_fields = [c for c in components if c == "TextField"]
                    if text_fields and not any(ctx.values()):
                        ctx["value"] = "10"  # sample input for TextField scenarios
                    payload = {
                        "surfaceId": surface_id,
                        "name": ev.get("name", ""),
                        "sourceComponentId": first_event_btn["id"],
                        "context": ctx,
                    }
                    next_msg = json.dumps({"type": "a2ui_action", "payload": payload})
                else:
                    break

    except Exception as e:
        result.issues.append(f"CONNECTION_ERROR: {e}")
        result.passed = False

    # Final assessment
    if any("HALLUCINATION" in i for i in result.issues):
        result.passed = False
    if any("CONVERSATION_LOOP" in i for i in result.issues):
        result.passed = False
    if any("NO_CARD_ON_FIRST_TURN" in i for i in result.issues):
        result.passed = False

    return result


# ── Test Scenarios ──

SCENARIOS = [
    ("restaurant_recommendation", "강서구 맛집 추천해줘"),
    ("weather_card", "서울 날씨 알려줘"),
    ("quiz_geography", "세계 수도 퀴즈 내줘"),
    ("todo_checklist", "오늘 할일 체크리스트 만들어줘"),
    ("comparison_table", "아이폰 16 vs 갤럭시 S25 비교해줘"),
    ("recipe_card", "김치찌개 레시피 카드로 보여줘"),
    ("schedule_weekly", "이번 주 운동 계획 세워줘"),
    ("game_menu", "간단한 게임 하나 만들어줘"),
    ("survey_form", "만족도 설문조사 카드 만들어줘"),
    ("travel_itinerary", "제주도 2박3일 여행 계획 카드로 만들어줘"),
    ("calculator", "간단한 계산기 카드 만들어줘"),
    ("music_playlist", "집중할 때 듣기 좋은 플레이리스트 추천해줘"),
]


async def main():
    print("=" * 70)
    print("A2UI Multi-Turn E2E Test Suite")
    print(f"Scenarios: {len(SCENARIOS)} | Max turns: {MAX_TURNS} | Timeout: {TIMEOUT}s")
    print("=" * 70)

    results = []
    passed = 0
    failed = 0

    for i, (name, prompt) in enumerate(SCENARIOS):
        print(f"\n{'─' * 60}")
        print(f"[{i + 1}/{len(SCENARIOS)}] {name}: {prompt}")
        print(f"{'─' * 60}")

        result = await run_scenario(name, prompt)
        results.append(result)

        for turn in result.turns:
            status = "✓" if not turn.issues else "✗"
            comp_summary = ", ".join(set(turn.components))[:60] if turn.components else "none"
            print(
                f"  Turn {turn.turn}: {status} "
                f"| a2ui={turn.a2ui_count} "
                f"| btns={len(turn.buttons)} "
                f"| {turn.elapsed_ms:.0f}ms "
                f"| [{comp_summary}]"
            )
            if turn.issues:
                for issue in turn.issues:
                    print(f"    ⚠ {issue}")
            # Show text snippet
            text_preview = turn.text[:120].replace("\n", " ")
            print(f"    > {text_preview}")

        if result.passed:
            passed += 1
            print(f"  → PASSED")
        else:
            failed += 1
            print(f"  → FAILED")
            for issue in result.issues:
                print(f"    ✗ {issue}")

    # Summary
    print(f"\n{'=' * 70}")
    print(f"SUMMARY: {passed}/{len(SCENARIOS)} passed, {failed} failed")
    print(f"{'=' * 70}")

    # Aggregate issues
    all_issues = {}
    for r in results:
        for issue in r.issues:
            # Extract issue type
            itype = issue.split(":")[0].split(" ")[-1] if ":" in issue else issue
            all_issues[itype] = all_issues.get(itype, 0) + 1

    if all_issues:
        print("\nIssue frequency:")
        for itype, count in sorted(all_issues.items(), key=lambda x: -x[1]):
            print(f"  {count}x {itype}")

    # Save report
    report = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "summary": {"total": len(SCENARIOS), "passed": passed, "failed": failed},
        "issue_frequency": all_issues,
        "scenarios": [
            {
                "name": r.name,
                "prompt": r.initial_prompt,
                "passed": r.passed,
                "turns": len(r.turns),
                "issues": r.issues,
                "turn_details": [
                    {
                        "turn": t.turn,
                        "text": t.text[:300],
                        "a2ui_count": t.a2ui_count,
                        "components": list(set(t.components)),
                        "button_count": len(t.buttons),
                        "data_model_keys": t.data_model_keys,
                        "elapsed_ms": round(t.elapsed_ms),
                        "issues": t.issues,
                    }
                    for t in r.turns
                ],
            }
            for r in results
        ],
    }

    import os
    script_dir = os.path.dirname(os.path.abspath(__file__))
    report_path = os.path.join(script_dir, "..", "reports", "multi_turn_report.json")
    os.makedirs(os.path.dirname(report_path), exist_ok=True)
    with open(report_path, "w") as f:
        json.dump(report, f, ensure_ascii=False, indent=2)
    print(f"\nReport saved to: {report_path}")

    return 1 if failed > 0 else 0


if __name__ == "__main__":
    exit_code = asyncio.run(main())
    sys.exit(exit_code)
