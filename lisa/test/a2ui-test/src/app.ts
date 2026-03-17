/**
 * Lisa A2UI Test Client
 *
 * Renders A2UI v0.9 cards directly using custom Lit elements.
 * Connects to Lisa gateway via WebSocket proxy.
 */

import { buildSurface, A2UISurface } from './a2ui-renderer.js';
import './a2ui-renderer.js';

// ── State ──
let ws: WebSocket | null = null;
let requestStartTime: number | null = null;
let thinkingTimer: number | null = null;
let thinkingEl: HTMLElement | null = null;
let currentA2UIMessages: unknown[] = [];

// ── DOM helpers ──
const $ = (id: string) => document.getElementById(id)!;

function setStatus(connected: boolean) {
  const el = $('status');
  const txt = $('status-text');
  el.className = connected ? 'connected' : '';
  txt.textContent = connected ? 'Connected' : 'Disconnected';
  ($('chat-input') as HTMLInputElement).disabled = !connected;
  ($('send-btn') as HTMLButtonElement).disabled = !connected;
  $('connect-btn').textContent = connected ? 'Disconnect' : 'Connect';
}

function scrollBottom() {
  const main = $('messages');
  main.scrollTop = main.scrollHeight;
}

function addMessage(type: string, content: string, elapsedSec?: number | null) {
  const main = $('messages');
  const div = document.createElement('div');
  div.className = `msg ${type}`;
  div.textContent = content;
  if (elapsedSec != null && type === 'assistant') {
    const badge = document.createElement('span');
    badge.className = 'elapsed';
    badge.textContent = `${elapsedSec.toFixed(1)}s`;
    div.appendChild(badge);
  }
  main.appendChild(div);
  scrollBottom();
}

// ── Thinking indicator ──
function showThinking() {
  removeThinking();
  requestStartTime = performance.now();
  const main = $('messages');
  thinkingEl = document.createElement('div');
  thinkingEl.className = 'thinking';
  thinkingEl.innerHTML = `
    <div class="spinner"></div>
    <span>Thinking...</span>
    <span class="timer">0.0s</span>
  `;
  main.appendChild(thinkingEl);
  scrollBottom();
  thinkingTimer = window.setInterval(() => {
    if (!thinkingEl || !requestStartTime) return;
    const elapsed = ((performance.now() - requestStartTime) / 1000).toFixed(1);
    const timerSpan = thinkingEl.querySelector('.timer');
    if (timerSpan) timerSpan.textContent = `${elapsed}s`;
  }, 100);
}

function removeThinking() {
  if (thinkingTimer) { clearInterval(thinkingTimer); thinkingTimer = null; }
  if (thinkingEl) { thinkingEl.remove(); thinkingEl = null; }
}

function getElapsed(): number | null {
  if (!requestStartTime) return null;
  return (performance.now() - requestStartTime) / 1000;
}

// ── A2UI rendering (v0.9 direct) ──
function renderA2UISurface(elapsedSec: number | null) {
  const main = $('messages');
  const container = document.createElement('div');
  container.className = 'a2ui-surface-container';

  const surface = buildSurface(currentA2UIMessages);
  if (surface) {
    const el = document.createElement('a2ui-surface-v09') as any;
    el.surface = surface;
    el.addEventListener('a2ui-action', (e: CustomEvent) => {
      handleA2UIAction(e.detail, surface.surfaceId);
    });
    container.appendChild(el);
  }

  // Raw JSON inspector
  if (currentA2UIMessages.length > 0) {
    const details = document.createElement('details');
    details.className = 'inspector';
    const summary = document.createElement('summary');
    summary.textContent = `Raw A2UI JSON (${currentA2UIMessages.length} messages)`;
    const pre = document.createElement('pre');
    pre.textContent = JSON.stringify(currentA2UIMessages, null, 2);
    details.appendChild(summary);
    details.appendChild(pre);
    container.appendChild(details);
  }

  // Elapsed badge
  if (elapsedSec != null) {
    const badge = document.createElement('div');
    badge.style.cssText = 'text-align:right;font-size:11px;color:#9aa0a6;margin-top:4px';
    badge.textContent = `${elapsedSec.toFixed(1)}s`;
    container.appendChild(badge);
  }

  main.appendChild(container);
  scrollBottom();
}

function handleA2UIAction(detail: any, surfaceId: string) {
  console.log('A2UI action:', detail);
  if (ws && ws.readyState === WebSocket.OPEN) {
    currentA2UIMessages = [];  // reset for next response
    ws.send(JSON.stringify({
      type: 'a2ui_action',
      payload: {
        surfaceId,
        name: detail?.name || 'unknown',
        sourceComponentId: detail?.sourceComponentId || 'unknown',
        context: detail?.context || {},
      },
    }));
    showThinking();
  }
}

// ── WebSocket handling ──
function handleWSMessage(data: any) {
  console.log('[WS]', data.type, data);
  switch (data.type) {
    case 'history':
      if (data.messages?.length) {
        addMessage('system', `History: ${data.messages.length} messages`);
      }
      break;

    case 'a2ui':
      if (data.messages) {
        console.log('[A2UI] received', data.messages.length, 'messages');
        currentA2UIMessages = data.messages;
      }
      break;

    case 'done': {
      const elapsed = getElapsed();
      removeThinking();
      console.log('[DONE] a2ui msgs:', currentA2UIMessages.length, 'full_response:', !!data.full_response);
      if (currentA2UIMessages.length > 0) {
        renderA2UISurface(elapsed);
        currentA2UIMessages = [];  // reset after rendering
      }
      if (data.full_response) {
        addMessage('assistant', data.full_response,
          currentA2UIMessages.length > 0 ? null : elapsed);
      }
      requestStartTime = null;
      break;
    }

    case 'chunk':
      break;

    case 'error':
      removeThinking();
      addMessage('system', `Error: ${data.message}`);
      requestStartTime = null;
      break;

    default:
      console.log('Unknown WS message:', data);
  }
}

// ── Public API (called from HTML) ──
(window as any).toggleConnection = function () {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.close();
    return;
  }
  const url = ($('ws-url') as HTMLInputElement).value.trim();
  if (!url) return;

  addMessage('system', `Connecting to ${url}...`);
  ws = new WebSocket(url);
  ws.onopen = () => { setStatus(true); addMessage('system', 'Connected'); };
  ws.onclose = () => { setStatus(false); removeThinking(); addMessage('system', 'Disconnected'); };
  ws.onerror = () => { addMessage('system', 'Connection error'); };
  ws.onmessage = (event) => {
    try { handleWSMessage(JSON.parse(event.data)); }
    catch (e) { console.error('Parse error:', e); }
  };
};

(window as any).sendMessage = function () {
  const input = $('chat-input') as HTMLInputElement;
  const text = input.value.trim();
  if (!text || !ws || ws.readyState !== WebSocket.OPEN) return;

  addMessage('user', text);
  currentA2UIMessages = [];
  showThinking();
  ws.send(JSON.stringify({ type: 'message', content: text }));
  input.value = '';
  input.focus();
};

// ── Init ──
window.addEventListener('load', () => {
  ($('chat-input') as HTMLInputElement).focus();
  const wsInput = $('ws-url') as HTMLInputElement;
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
  wsInput.value = `${proto}//${location.host}/ws`;
  (window as any).toggleConnection();
});
