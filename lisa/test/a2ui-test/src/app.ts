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

// ── Session Management ──
const SESSIONS_KEY = 'lisa-sessions';

function getSessions(): string[] {
  const raw = localStorage.getItem(SESSIONS_KEY);
  return raw ? JSON.parse(raw) : ['lisa-test'];
}

function saveSessions(sessions: string[]) {
  localStorage.setItem(SESSIONS_KEY, JSON.stringify(sessions));
}

function renderSessionList() {
  const sel = document.getElementById('session-select') as HTMLSelectElement;
  const sessions = getSessions();
  const cur = sel.value || sessions[0];
  sel.innerHTML = sessions.map(s =>
    `<option value="${s}"${s === cur ? ' selected' : ''}>${s}</option>`
  ).join('');
}

function getSessionId(): string {
  const sel = document.getElementById('session-select') as HTMLSelectElement;
  return sel?.value || 'lisa-test';
}

(window as any).newSession = function () {
  const name = prompt('세션 이름:');
  if (!name || !name.trim()) return;
  const sessions = getSessions();
  if (!sessions.includes(name.trim())) {
    sessions.push(name.trim());
    saveSessions(sessions);
  }
  renderSessionList();
  (document.getElementById('session-select') as HTMLSelectElement).value = name.trim();
};

(window as any).deleteSession = function () {
  const id = getSessionId();
  if (id === 'lisa-test') { alert('기본 세션은 삭제할 수 없습니다'); return; }
  if (!confirm(`"${id}" 세션을 삭제하시겠습니까?`)) return;
  const sessions = getSessions().filter(s => s !== id);
  saveSessions(sessions);
  renderSessionList();
};

(window as any).toggleMenu = function () {
  const menu = document.getElementById('session-menu')!;
  menu.style.display = menu.style.display === 'none' ? 'block' : 'none';
};

(window as any).switchSession = function () {
  document.getElementById('session-menu')!.style.display = 'none';
  const label = document.getElementById('current-session-label');
  if (label) label.textContent = getSessionId();
  // Reconnect with new session
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.close();
  }
  setTimeout(() => (window as any).toggleConnection(), 300);
};

// Close menu on outside click
document.addEventListener('click', (e) => {
  const menu = document.getElementById('session-menu');
  const btn = document.getElementById('menu-btn');
  if (menu && btn && !menu.contains(e.target as Node) && !btn.contains(e.target as Node)) {
    menu.style.display = 'none';
  }
});
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
      handleA2UIAction(e.detail, surface);
    });
    container.appendChild(el);
  }

  // Raw JSON inspector with copy button
  const surfaceId = surface?.surfaceId;
  if (surfaceId) {
    container.dataset.surfaceId = surfaceId;
  }
  if (currentA2UIMessages.length > 0) {
    const inspectorWrap = document.createElement('div');
    inspectorWrap.className = 'inspector-wrap';
    const jsonText = JSON.stringify(currentA2UIMessages, null, 2);
    const copyBtn = document.createElement('button');
    copyBtn.className = 'copy-btn';
    copyBtn.textContent = '📋 Copy';
    copyBtn.addEventListener('click', () => {
      const doCopy = () => {
        if (navigator.clipboard?.writeText) {
          return navigator.clipboard.writeText(jsonText);
        }
        const ta = document.createElement('textarea');
        ta.value = jsonText;
        ta.style.cssText = 'position:fixed;left:-9999px;top:0';
        document.body.appendChild(ta);
        ta.select();
        const ok = document.execCommand('copy');
        document.body.removeChild(ta);
        return ok ? Promise.resolve() : Promise.reject();
      };
      doCopy().then(() => {
        copyBtn.textContent = '✅ Copied!';
        setTimeout(() => { copyBtn.textContent = '📋 Copy'; }, 1500);
      }).catch(() => {
        copyBtn.textContent = '❌ Failed';
        setTimeout(() => { copyBtn.textContent = '📋 Copy'; }, 1500);
      });
    });
    inspectorWrap.appendChild(copyBtn);
    const details = document.createElement('details');
    details.className = 'inspector';
    const summary = document.createElement('summary');
    summary.textContent = `Raw A2UI JSON (${currentA2UIMessages.length} messages)`;
    const pre = document.createElement('pre');
    pre.textContent = jsonText;
    details.appendChild(summary);
    details.appendChild(pre);
    inspectorWrap.appendChild(details);
    container.appendChild(inspectorWrap);
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

// ── A2Web iframe rendering ──
function rewriteA2WebUrl(url: string): string {
  // Rewrite 127.0.0.1 / localhost to the actual server hostname
  // so external clients can access a2web pages
  try {
    const parsed = new URL(url);
    if (parsed.hostname === '127.0.0.1' || parsed.hostname === 'localhost') {
      parsed.hostname = location.hostname;
    }
    return parsed.toString();
  } catch {
    return url;
  }
}

function renderA2WebFrame(data: { url?: string; title?: string; id?: string }) {
  const elapsed = getElapsed();
  removeThinking();
  const main = $('messages');
  const container = document.createElement('div');
  container.className = 'a2web-container';

  const pageUrl = rewriteA2WebUrl(data.url || '');
  const header = document.createElement('div');
  header.className = 'a2web-header';
  header.innerHTML = `
    <span class="a2web-label">a2web</span>
    <span class="a2web-title">${data.title || 'Web Page'}</span>
    <a href="${pageUrl}" target="_blank" rel="noopener" class="a2web-open">새 탭에서 열기 ↗</a>
  `;
  container.appendChild(header);

  const iframe = document.createElement('iframe');
  iframe.src = pageUrl;
  iframe.className = 'a2web-iframe';
  iframe.setAttribute('sandbox', 'allow-scripts allow-same-origin allow-forms allow-popups');
  container.appendChild(iframe);

  if (elapsed != null) {
    const badge = document.createElement('div');
    badge.style.cssText = 'text-align:right;font-size:11px;color:#9aa0a6;margin-top:4px';
    badge.textContent = `${elapsed.toFixed(1)}s`;
    container.appendChild(badge);
  }

  main.appendChild(container);
  scrollBottom();
  requestStartTime = null;
}

function handleA2UIAction(detail: any, surface: any) {
  console.log('A2UI action:', detail);
  if (ws && ws.readyState === WebSocket.OPEN) {
    currentA2UIMessages = [];  // reset for next response
    const payload: any = {
      surfaceId: surface.surfaceId,
      name: detail?.name || 'unknown',
      sourceComponentId: detail?.sourceComponentId || 'unknown',
      context: detail?.context || {},
    };
    // v0.9 standard: include dataModel when sendDataModel is enabled
    if (surface.sendDataModel && surface.dataModel) {
      payload.dataModel = surface.dataModel;
    }
    ws.send(JSON.stringify({ type: 'a2ui_action', payload }));
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
        // Handle deleteSurface — remove matching card from DOM
        for (const msg of data.messages as any[]) {
          if (msg.deleteSurface?.surfaceId) {
            const sid = msg.deleteSurface.surfaceId;
            const el = document.querySelector(`[data-surface-id="${sid}"]`);
            if (el) el.remove();
            console.log('[A2UI] deleteSurface:', sid);
          }
        }
        currentA2UIMessages = data.messages;
      }
      break;

    case 'a2web':
      renderA2WebFrame(data);
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
  let url = ($('ws-url') as HTMLInputElement).value.trim();
  if (!url) return;

  // Append session_id for persistent sessions
  const sep = url.includes('?') ? '&' : '?';
  if (!url.includes('session_id=')) {
    url += `${sep}session_id=${encodeURIComponent(getSessionId())}`;
  }

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
  renderSessionList();
  ($('chat-input') as HTMLInputElement).focus();
  const wsInput = $('ws-url') as HTMLInputElement;
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
  // Use gateway port (42617) and the /app Lisa channel endpoint
  const gwHost = location.hostname + ':42617';
  wsInput.value = `${proto}//${gwHost}/app`;
  (window as any).toggleConnection();
});
