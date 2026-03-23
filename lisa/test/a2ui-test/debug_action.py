#!/usr/bin/env python3
"""Debug: Send a quiz request, then a button action, and print ALL WS messages."""
import asyncio
import json
import sys
import websockets

WS_URL = "ws://127.0.0.1:42617/app"

async def main():
    async with websockets.connect(WS_URL, ping_interval=None, ping_timeout=None) as ws:
        # Collect initial messages
        print("=== Connected, waiting for history ===")
        try:
            msg = await asyncio.wait_for(ws.recv(), timeout=5)
            data = json.loads(msg)
            print(f"[{data['type']}] {len(data.get('messages', []))} messages")
        except asyncio.TimeoutError:
            pass

        # Step 1: Send quiz request
        print("\n=== Step 1: Requesting quiz ===")
        await ws.send(json.dumps({"type": "message", "content": "퀴즈! 4지선다 상식 문제 하나"}))

        a2ui_data = None
        done_text = None
        while True:
            try:
                msg = await asyncio.wait_for(ws.recv(), timeout=120)
                data = json.loads(msg)
                print(f"[{data['type']}]", end=" ")
                if data['type'] == 'a2ui':
                    a2ui_data = data['messages']
                    # Extract data model
                    for m in a2ui_data:
                        if 'updateDataModel' in m:
                            dm = m['updateDataModel'].get('value', {})
                            print(f"\n  DataModel: {json.dumps(dm, ensure_ascii=False)}")
                        if 'updateComponents' in m:
                            # Find buttons with actions
                            for comp in m['updateComponents'].get('components', []):
                                if comp.get('component') == 'Button' and 'action' in comp:
                                    print(f"\n  Button: id={comp['id']} action={json.dumps(comp['action'], ensure_ascii=False)}")
                elif data['type'] == 'done':
                    done_text = data.get('full_response', '')
                    print(f"\n  Response: {done_text[:200]}")
                    break
                elif data['type'] == 'chunk':
                    pass
                else:
                    print(json.dumps(data, ensure_ascii=False)[:100])
            except asyncio.TimeoutError:
                print("TIMEOUT")
                return

        if not a2ui_data:
            print("No A2UI data received!")
            return

        # Find surface ID and first button
        surface_id = None
        button = None
        for m in a2ui_data:
            if 'createSurface' in m:
                surface_id = m['createSurface']['surfaceId']
            if 'updateComponents' in m:
                for comp in m['updateComponents'].get('components', []):
                    if comp.get('component') == 'Button' and 'action' in comp and not button:
                        button = comp

        if not surface_id or not button:
            print("Could not find surface/button!")
            return

        # Step 2: Click the button (second option = B typically)
        # Find the second button instead
        buttons = []
        for m in a2ui_data:
            if 'updateComponents' in m:
                for comp in m['updateComponents'].get('components', []):
                    if comp.get('component') == 'Button' and 'action' in comp:
                        buttons.append(comp)

        if len(buttons) >= 2:
            button = buttons[1]  # Pick option B

        action_payload = {
            "surfaceId": surface_id,
            "name": button['action']['event']['name'],
            "sourceComponentId": button['id'],
            "context": button['action']['event'].get('context', {})
        }

        print(f"\n=== Step 2: Sending action ===")
        print(f"  Payload: {json.dumps(action_payload, ensure_ascii=False)}")

        await ws.send(json.dumps({"type": "a2ui_action", "payload": action_payload}))

        # Collect response
        while True:
            try:
                msg = await asyncio.wait_for(ws.recv(), timeout=120)
                data = json.loads(msg)
                print(f"[{data['type']}]", end=" ")
                if data['type'] == 'a2ui':
                    for m in data['messages']:
                        if 'updateDataModel' in m:
                            dm = m['updateDataModel'].get('value', {})
                            print(f"\n  Response DataModel: {json.dumps(dm, ensure_ascii=False)}")
                elif data['type'] == 'done':
                    resp = data.get('full_response', '')
                    print(f"\n  === LLM RESPONSE ===")
                    print(f"  {resp}")

                    # Check if it mentions the actual option text
                    # Extract option text from quiz data model
                    option_texts = []
                    for m in a2ui_data:
                        if 'updateDataModel' in m:
                            dm = m['updateDataModel'].get('value', {})
                            if 'options' in dm:
                                for k, v in dm['options'].items():
                                    option_texts.append(v)
                            for k, v in dm.items():
                                if k.startswith('option') and isinstance(v, str):
                                    option_texts.append(v)

                    choice = action_payload['context'].get('choice', '')
                    print(f"\n  === VERDICT ===")
                    print(f"  Choice sent: {choice}")
                    print(f"  Option texts: {option_texts}")

                    found = False
                    for t in option_texts:
                        # Check if any option text appears in response
                        # Strip the letter prefix if present (e.g., "B. 대서양" -> "대서양")
                        clean = t.split('. ', 1)[-1] if '. ' in t else t
                        if clean in resp:
                            print(f"  FOUND option text '{clean}' in response!")
                            found = True

                    if found:
                        print("  RESULT: PASS ✓")
                    else:
                        print("  RESULT: FAIL ✗ — LLM did not mention actual option text")
                    break
                elif data['type'] == 'chunk':
                    pass
            except asyncio.TimeoutError:
                print("TIMEOUT waiting for action response")
                return

asyncio.run(main())
