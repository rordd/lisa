#!/usr/bin/env python3
"""E2E test for Lisa A2UI quiz flow via WebSocket.

Tests whether:
1. Button action context includes "choiceText" (actual option text, not just letter)
2. The LLM response after selecting an option references the actual option text

Protocol:
1. Send quiz request message
2. Receive a2ui (card data) + done (text) messages
3. Extract surface ID, button actions, data model from A2UI
4. Send a2ui_action selecting one option
5. Check if LLM response references the actual option text
"""

import asyncio
import json
import sys
import time

import websockets

WS_URL = "ws://127.0.0.1:42617/ws/chat"
TIMEOUT_INITIAL = 120  # seconds to wait for first response (LLM can be slow)
TIMEOUT_ACTION = 120   # seconds to wait for action response


def log(label: str, data):
    """Pretty-print a labeled message."""
    print(f"\n{'='*70}")
    print(f"  {label}")
    print(f"{'='*70}")
    if isinstance(data, (dict, list)):
        print(json.dumps(data, indent=2, ensure_ascii=False))
    else:
        print(data)
    print()


async def receive_all_messages(ws, timeout: float) -> list[dict]:
    """Receive messages until 'done' or 'error', or timeout."""
    messages = []
    deadline = time.monotonic() + timeout
    while True:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            print(f"[TIMEOUT] No 'done' message received within {timeout}s")
            break
        try:
            raw = await asyncio.wait_for(ws.recv(), timeout=remaining)
            msg = json.loads(raw)
            msg_type = msg.get("type", "unknown")
            print(f"  <- recv: type={msg_type}")
            messages.append(msg)

            if msg_type in ("done", "error"):
                break
        except asyncio.TimeoutError:
            print(f"[TIMEOUT] No more messages within {timeout}s")
            break
        except websockets.ConnectionClosed as e:
            print(f"[CLOSED] Connection closed: {e}")
            break
    return messages


def extract_a2ui_info(messages: list[dict]) -> dict | None:
    """Extract surface ID, buttons (with full action details), and data model from A2UI messages."""
    for msg in messages:
        if msg.get("type") != "a2ui":
            continue

        a2ui_messages = msg.get("messages", [])
        info = {
            "surface_id": None,
            "components": [],
            "data_model": None,
            "buttons": [],
            "raw_a2ui": a2ui_messages,
        }

        for a2ui_msg in a2ui_messages:
            # createSurface
            if "createSurface" in a2ui_msg:
                info["surface_id"] = a2ui_msg["createSurface"].get("surfaceId")

            # updateComponents - look for buttons
            if "updateComponents" in a2ui_msg:
                sid = a2ui_msg["updateComponents"].get("surfaceId")
                if info["surface_id"] is None:
                    info["surface_id"] = sid
                components = a2ui_msg["updateComponents"].get("components", [])
                info["components"] = components
                for comp in components:
                    comp_type = comp.get("component", "")
                    is_button = False
                    if isinstance(comp_type, dict):
                        if "Button" in comp_type:
                            is_button = True
                    elif comp_type in ("Button", "button"):
                        is_button = True
                    if is_button:
                        info["buttons"].append(comp)

            # updateDataModel - option mappings
            if "updateDataModel" in a2ui_msg:
                info["data_model"] = a2ui_msg["updateDataModel"].get("value")
                if info["surface_id"] is None:
                    info["surface_id"] = a2ui_msg["updateDataModel"].get("surfaceId")

        return info if info["surface_id"] else None
    return None


def extract_button_action(btn: dict) -> dict | None:
    """Extract action details from a button component.

    Handles both:
    - v0.9 style: {"component": "Button", "action": {"event": {"name": "...", "context": {...}}}}
    - Dict style: {"component": {"Button": {"action": {"event": {...}}}}}
    """
    action = None
    comp = btn.get("component", "")

    if isinstance(comp, dict) and "Button" in comp:
        action = comp["Button"].get("action", {})
    else:
        action = btn.get("action", {})

    if not action:
        return None

    # action can be {"event": {"name": ..., "context": ...}} or directly {"name": ..., "context": ...}
    if "event" in action:
        event = action["event"]
        return {
            "name": event.get("name", "unknown"),
            "context": event.get("context", {}),
        }
    elif "name" in action:
        return {
            "name": action.get("name", "unknown"),
            "context": action.get("context", {}),
        }
    return None


async def run_test():
    print(f"Connecting to {WS_URL} ...")
    try:
        ws = await websockets.connect(
            WS_URL,
            ping_interval=None,
            ping_timeout=None,
            close_timeout=10,
        )
    except Exception as e:
        print(f"[ERROR] Failed to connect: {e}")
        sys.exit(1)

    print("Connected!")

    # --- Step 0: Receive initial history message ---
    try:
        raw = await asyncio.wait_for(ws.recv(), timeout=5)
        init_msg = json.loads(raw)
        log("INITIAL MESSAGE (history)", {"type": init_msg.get("type"), "num_messages": len(init_msg.get("messages", []))})
    except asyncio.TimeoutError:
        print("[WARN] No initial history message received")

    # --- Step 1: Send quiz request ---
    quiz_request = {
        "type": "message",
        "content": "퀴즈 하자! 4지선다 상식 퀴즈 문제 하나만",
    }
    log("SENDING QUIZ REQUEST", quiz_request)
    await ws.send(json.dumps(quiz_request))

    # --- Step 2: Receive quiz response ---
    print("\nWaiting for quiz response (this may take 30+ seconds)...")
    quiz_messages = await receive_all_messages(ws, TIMEOUT_INITIAL)

    for msg in quiz_messages:
        msg_type = msg.get("type", "unknown")
        if msg_type == "a2ui":
            log("A2UI MESSAGE", msg)
        elif msg_type == "done":
            full_resp = msg.get("full_response", "")
            log("DONE MESSAGE (FULL TEXT)", {
                "type": "done",
                "full_response": full_resp,
            })
        elif msg_type == "error":
            log("ERROR MESSAGE", msg)
            print("[FAIL] Got error instead of quiz response. Exiting.")
            await ws.close()
            sys.exit(1)
        elif msg_type == "chunk":
            pass  # skip chunk logging
        else:
            log(f"OTHER MESSAGE (type={msg_type})", msg)

    # --- Step 3: Extract A2UI info ---
    a2ui_info = extract_a2ui_info(quiz_messages)
    if a2ui_info is None:
        print("\n[FAIL] No A2UI data found in quiz response.")
        for m in quiz_messages:
            print(f"  type={m.get('type')}")
        for m in quiz_messages:
            if m.get("type") == "done":
                log("FULL RESPONSE (no A2UI found)", m.get("full_response", ""))
        await ws.close()
        sys.exit(1)

    log("EXTRACTED A2UI INFO", {
        "surface_id": a2ui_info["surface_id"],
        "num_components": len(a2ui_info["components"]),
        "num_buttons": len(a2ui_info["buttons"]),
        "data_model": a2ui_info["data_model"],
    })

    # --- Step 4: Analyze button actions ---
    print("\n" + "="*70)
    print("  BUTTON ACTION ANALYSIS")
    print("="*70)

    button_actions = []
    for i, btn in enumerate(a2ui_info["buttons"]):
        btn_id = btn.get("id", f"unknown-{i}")
        action_info = extract_button_action(btn)
        button_actions.append({
            "id": btn_id,
            "action": action_info,
        })
        print(f"\n  Button [{btn_id}]:")
        if action_info:
            print(f"    action name: {action_info['name']}")
            print(f"    context:     {json.dumps(action_info['context'], ensure_ascii=False)}")
            has_choice_text = "choiceText" in action_info.get("context", {})
            print(f"    has choiceText: {has_choice_text}")
        else:
            print(f"    (no action found)")
            print(f"    raw button: {json.dumps(btn, ensure_ascii=False)}")

    if not button_actions:
        print("\n[WARN] No buttons found. Trying generic action with data model.")

    # --- Step 5: Pick an option and send button action ---
    # Try to pick the second button (B) if available, else first
    chosen_idx = min(1, len(button_actions) - 1) if button_actions else 0
    chosen_btn = button_actions[chosen_idx] if button_actions else None

    if chosen_btn and chosen_btn["action"]:
        action_name = chosen_btn["action"]["name"]
        action_context = chosen_btn["action"]["context"]
        btn_id = chosen_btn["id"]
    else:
        # Fallback: construct a generic action
        action_name = "submit"
        action_context = {"choice": "B"}
        btn_id = "btn-b"

    action_payload = {
        "type": "a2ui_action",
        "payload": {
            "surfaceId": a2ui_info["surface_id"],
            "name": action_name,
            "sourceComponentId": btn_id,
            "context": action_context,
        },
    }

    log("SENDING BUTTON ACTION", action_payload)
    await ws.send(json.dumps(action_payload))

    # --- Step 6: Receive action response ---
    print("\nWaiting for action response...")
    action_messages = await receive_all_messages(ws, TIMEOUT_ACTION)

    response_text = ""
    for msg in action_messages:
        msg_type = msg.get("type", "unknown")
        if msg_type == "a2ui":
            log("ACTION RESPONSE - A2UI", msg)
        elif msg_type == "done":
            response_text = msg.get("full_response", "")
            log("ACTION RESPONSE - DONE (FULL TEXT)", {
                "type": "done",
                "full_response": response_text,
            })
        elif msg_type == "error":
            log("ACTION RESPONSE - ERROR", msg)
        elif msg_type == "chunk":
            pass
        else:
            log(f"ACTION RESPONSE - OTHER (type={msg_type})", msg)

    # --- Step 7: Final Verdict ---
    print("\n" + "="*70)
    print("  TEST RESULTS")
    print("="*70)

    # Check 1: Does button context include choiceText?
    check1_pass = False
    choice_text_value = None
    for ba in button_actions:
        if ba["action"] and "choiceText" in ba["action"].get("context", {}):
            check1_pass = True
            choice_text_value = ba["action"]["context"]["choiceText"]
            break

    print(f"\n  CHECK 1: Button context includes 'choiceText'")
    if check1_pass:
        print(f"    PASS - choiceText found: '{choice_text_value}'")
    else:
        print(f"    FAIL - No 'choiceText' found in any button context")
        print(f"    Button contexts:")
        for ba in button_actions:
            if ba["action"]:
                print(f"      [{ba['id']}] context = {json.dumps(ba['action']['context'], ensure_ascii=False)}")

    # Check 2: Does the LLM response mention the actual option text?
    check2_pass = False
    selected_text = None

    # Get the selected option's text
    if chosen_btn and chosen_btn["action"]:
        ctx = chosen_btn["action"].get("context", {})
        selected_text = ctx.get("choiceText")
        if not selected_text:
            # Try to resolve from data model
            choice_key = ctx.get("choice", "")
            dm = a2ui_info.get("data_model", {})
            if dm:
                options = dm.get("options", dm.get("choices", dm.get("answers", {})))
                if isinstance(options, dict):
                    selected_text = options.get(choice_key, options.get(choice_key.lower()))
                elif isinstance(options, list):
                    idx = ord(choice_key) - ord('A') if choice_key else -1
                    if 0 <= idx < len(options):
                        selected_text = str(options[idx])

    print(f"\n  CHECK 2: LLM response references actual option text")
    if not response_text:
        print(f"    FAIL - No response text received")
    elif not selected_text:
        print(f"    SKIP - Could not determine selected option text")
        print(f"    Response: {response_text}")
    else:
        if selected_text in response_text:
            check2_pass = True
            print(f"    PASS - Response contains '{selected_text}'")
        else:
            # Check if response mentions any of the option texts (LLM might paraphrase)
            print(f"    FAIL - Response does NOT contain '{selected_text}'")
            print(f"    Response text: {response_text}")
            # Try partial match
            if any(word in response_text for word in selected_text.split() if len(word) > 1):
                print(f"    NOTE - Partial match found (some words from '{selected_text}' appear)")

    # Check 3: Data model present?
    check3_pass = a2ui_info.get("data_model") is not None
    print(f"\n  CHECK 3: Data model present in A2UI")
    if check3_pass:
        print(f"    PASS - Data model: {json.dumps(a2ui_info['data_model'], ensure_ascii=False)}")
    else:
        print(f"    FAIL - No data model found")

    # Overall verdict
    print(f"\n{'='*70}")
    all_pass = check1_pass and check2_pass and check3_pass
    if all_pass:
        print("  VERDICT: PASS - All checks passed")
    else:
        failed = []
        if not check1_pass:
            failed.append("choiceText in button context")
        if not check2_pass:
            failed.append("LLM references option text")
        if not check3_pass:
            failed.append("data model present")
        print(f"  VERDICT: FAIL - Failed checks: {', '.join(failed)}")
    print(f"{'='*70}")

    await ws.close()
    print("\nTest complete.")
    sys.exit(0 if all_pass else 1)


if __name__ == "__main__":
    asyncio.run(run_test())
