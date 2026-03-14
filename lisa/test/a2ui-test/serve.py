#!/usr/bin/env python3
"""A2UI Test App — HTTP + WebSocket proxy server.

Serves index.html on HTTP and proxies /ws to the zeroclaw gateway,
so only one port needs to be open externally.

Usage:
    python3 serve.py [--port 8321] [--gateway ws://127.0.0.1:42617/ws/chat]
"""

import asyncio
import argparse
import pathlib
import mimetypes

import websockets
from websockets.asyncio.server import serve as ws_serve

HERE = pathlib.Path(__file__).parent
STATIC_DIR = HERE / "dist"  # Vite build output
GATEWAY = "ws://127.0.0.1:42617/ws/chat"


async def proxy_ws(client_ws):
    """Proxy WebSocket messages between browser client and zeroclaw gateway."""
    async with websockets.connect(GATEWAY, ping_interval=None, ping_timeout=None, close_timeout=None) as gw_ws:
        async def client_to_gw():
            try:
                async for msg in client_ws:
                    await gw_ws.send(msg)
            except websockets.ConnectionClosed:
                pass

        async def gw_to_client():
            try:
                async for msg in gw_ws:
                    await client_ws.send(msg)
            except websockets.ConnectionClosed:
                pass

        # Run both directions concurrently; stop when either side closes
        done, pending = await asyncio.wait(
            [asyncio.ensure_future(client_to_gw()),
             asyncio.ensure_future(gw_to_client())],
            return_when=asyncio.FIRST_COMPLETED,
        )
        for task in pending:
            task.cancel()


async def handle_request(connection, request):
    """Handle HTTP requests (serve static files) and upgrade WebSocket."""
    path = request.path

    # WebSocket upgrade for /ws
    if path == "/ws":
        return None  # let websockets handle the upgrade

    # Serve static files
    if path == "/" or path == "":
        file_path = STATIC_DIR / "index.html"
    else:
        file_path = STATIC_DIR / path.lstrip("/")

    if file_path.is_file():
        content_type, _ = mimetypes.guess_type(str(file_path))
        content_type = content_type or "application/octet-stream"
        body = file_path.read_bytes()
        return websockets.http11.Response(
            200,
            "OK",
            websockets.datastructures.Headers({
                "Content-Type": content_type,
                "Cache-Control": "no-cache",
            }),
            body,
        )

    return websockets.http11.Response(
        404, "Not Found",
        websockets.datastructures.Headers({"Content-Type": "text/plain"}),
        b"Not Found",
    )


async def handler(websocket):
    """Handle upgraded WebSocket connections (proxy to gateway)."""
    await proxy_ws(websocket)


async def main(port: int, gateway: str):
    global GATEWAY
    GATEWAY = gateway

    async with ws_serve(
        handler,
        "0.0.0.0",
        port,
        process_request=handle_request,
        ping_interval=None,
        ping_timeout=None,
        close_timeout=None,
    ) as server:
        print(f"A2UI Test App running on http://0.0.0.0:{port}")
        print(f"  Gateway proxy: {gateway}")
        print(f"  Open in browser and test A2UI cards")
        await asyncio.get_event_loop().create_future()  # run forever


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="A2UI Test App Server")
    parser.add_argument("--port", type=int, default=8321)
    parser.add_argument("--gateway", default="ws://127.0.0.1:42617/ws/chat")
    args = parser.parse_args()
    asyncio.run(main(args.port, args.gateway))
