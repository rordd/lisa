/**
 * A2UI v0.9 Renderer — Lit-based custom elements.
 *
 * Renders A2UI v0.9 components directly from JSON,
 * following the official v0.9 specification.
 * No dependency on @a2ui/lit (v0.8 only).
 */

import { LitElement, html, css, nothing, TemplateResult } from 'lit';
import { customElement, property } from 'lit/decorators.js';

// ── Types ──

export interface A2UIComponent {
  id: string;
  component?: string;
  type?: string;
  child?: string;
  children?: string[] | { componentId?: string; path?: string };
  text?: string | { path: string };
  variant?: string;
  align?: string;
  justify?: string;
  direction?: string;
  action?: {
    event?: { name?: string; context?: Record<string, unknown> };
    functionCall?: { call: string; args?: Record<string, unknown>; returnType?: string };
  };
  primary?: boolean;
  label?: string | { path: string };
  value?: unknown;
  min?: number;
  max?: number;
  url?: string | { path: string };
  fit?: string;
  name?: string | { path: string };
  axis?: string;
  tabs?: Array<{ label?: string | { path: string }; title?: string | { path: string }; child: string }>;
  tabItems?: Array<{ title?: string | { path: string }; child: string }>;
  trigger?: string;
  content?: string;
  entryPointChild?: string;
  contentChild?: string;
  options?: Array<{ label: string | { path: string }; value: string }>;
  selections?: unknown;
  description?: string | { path: string };
  enableDate?: boolean;
  enableTime?: boolean;
  textFieldType?: string;
  validationRegexp?: string;
  [key: string]: unknown;
}

export interface A2UISurface {
  surfaceId: string;
  components: Map<string, A2UIComponent>;
  dataModel: Record<string, unknown>;
  rootId: string;
}

// ── Surface state builder ──

export function buildSurface(messages: any[]): A2UISurface | null {
  let surfaceId = '@default';
  const components = new Map<string, A2UIComponent>();
  let dataModel: Record<string, unknown> = {};
  let rootId = 'root';

  for (const msg of messages) {
    // Support both v0.9 key names: createSurface / beginRendering
    const surface = msg.createSurface || msg.beginRendering;
    if (surface) {
      surfaceId = surface.surfaceId || '@default';
      if (surface.root) rootId = surface.root;
    }
    // Support both v0.9 key names: updateComponents / surfaceUpdate
    const update = msg.updateComponents || msg.surfaceUpdate;
    if (update) {
      for (const c of update.components) {
        // Normalize v0.9 nested component format:
        // {"component": {"Column": {"children": {"explicitList": [...]}}}, "id": "..."}
        // → flatten to {"component": "Column", "children": [...], "id": "..."}
        const normalized: any = { id: c.id };
        if (c.component && typeof c.component === 'object') {
          const keys = Object.keys(c.component);
          if (keys.length === 1) {
            const typeName = keys[0];
            normalized.component = typeName;
            const props = c.component[typeName];
            if (props && typeof props === 'object') {
              for (const [k, v] of Object.entries(props)) {
                if (k === 'children' && v && typeof v === 'object' && (v as any).explicitList) {
                  normalized.children = (v as any).explicitList;
                } else if (k === 'text' && v && typeof v === 'object' && (v as any).literalString != null) {
                  normalized.text = (v as any).literalString;
                } else if (k === 'label' && v && typeof v === 'object' && (v as any).literalString != null) {
                  normalized.label = (v as any).literalString;
                } else if (k === 'name' && v && typeof v === 'object' && (v as any).literalString != null) {
                  normalized.name = (v as any).literalString;
                } else if (k === 'description' && v && typeof v === 'object' && (v as any).literalString != null) {
                  normalized.description = (v as any).literalString;
                } else if (k === 'url' && v && typeof v === 'object' && (v as any).literalString != null) {
                  normalized.url = (v as any).literalString;
                } else {
                  normalized[k] = v;
                }
              }
            }
          }
        } else {
          Object.assign(normalized, c);
        }
        components.set(normalized.id, normalized);
        if (normalized.id === 'root') rootId = 'root';
      }
    }
    // Support both v0.9 key names: updateDataModel / dataModelUpdate
    const dm = msg.updateDataModel || msg.dataModelUpdate;
    if (dm) {
      if (!dm.path || dm.path === '/') {
        dataModel = { ...dataModel, ...dm.value };
      }
    }
  }

  if (components.size === 0) return null;
  return { surfaceId, components, dataModel, rootId };
}

// ── Resolve text values ──

/** Resolve a path like "/label" or "/items/0/title" against a data model.
 *  Also handles LLM-generated template placeholders like "/items/{index}/title"
 *  by substituting {index} with the _index value from the data model.
 */
function resolvePath(path: string, dataModel: Record<string, unknown>): unknown {
  const parts = path.replace(/^\//, '').split('/');
  let val: unknown = dataModel;
  for (const p of parts) {
    if (val == null || typeof val !== 'object') return undefined;
    let key = p;
    // Handle {varName} template placeholders (e.g., {index}, {idx}, {i})
    // Substitute with _varName or varName from the data model root
    if (/^\{.+\}$/.test(p)) {
      const varName = p.slice(1, -1);
      const idx = (dataModel as any)['_' + varName] ?? (dataModel as any)[varName];
      if (idx != null) {
        key = String(idx);
      }
    }
    val = (val as Record<string, unknown>)[key];
  }
  return val;
}

function resolveText(
  text: string | { path: string } | undefined,
  dataModel: Record<string, unknown>,
): string {
  if (text == null) return '';
  if (typeof text === 'string') {
    // Handle "${/path}" string interpolation pattern (LLM sometimes uses this instead of {path: "/..."})
    // Supports: "${/label}", "Temperature: ${/temp}°C", pure "${/title}"
    if (text.includes('${/')) {
      return text.replace(/\$\{(\/[^}]+)\}/g, (_match, path) => {
        const val = resolvePath(path, dataModel);
        return val != null ? String(val) : '';
      });
    }
    return text;
  }
  if (text.path) {
    const val = resolvePath(text.path, dataModel);
    return val != null ? String(val) : '';
  }
  return '';
}

// ── Resolve arbitrary values (boolean, number, etc.) ──

function resolveValue(
  value: unknown,
  dataModel: Record<string, unknown>,
): unknown {
  if (value == null) return undefined;
  // {path: "/checked"} object binding
  if (typeof value === 'object' && value !== null && 'path' in value) {
    return resolvePath((value as { path: string }).path, dataModel);
  }
  // "${/path}" string binding
  if (typeof value === 'string' && value.includes('${/')) {
    const match = value.match(/^\$\{(\/[^}]+)\}$/);
    if (match) {
      return resolvePath(match[1], dataModel);
    }
  }
  return value;
}

// ── Main Surface Element ──

@customElement('a2ui-surface-v09')
export class A2UISurfaceElement extends LitElement {
  @property({ type: Object }) surface: A2UISurface | null = null;

  /** Track user input values from TextFields, CheckBoxes, Sliders, ChoicePickers */
  private _inputValues = new Map<string, unknown>();

  override updated(changedProps: Map<string, unknown>) {
    if (changedProps.has('surface')) {
      this._inputValues.clear();
    }
  }

  private _fireAction(name: string, componentId: string, context: Record<string, unknown>) {
    // Resolve data binding paths ({path: "/options/B"} → actual value) before sending
    const resolvedContext: Record<string, unknown> = {};
    const dataModel = this.surface?.dataModel ?? {};
    for (const [key, val] of Object.entries(context)) {
      resolvedContext[key] = resolveValue(val, dataModel);
    }
    // Merge collected form input values into the event context
    const mergedContext: Record<string, unknown> = { ...resolvedContext };
    for (const [compId, value] of this._inputValues) {
      // Use component id as key if not already in context
      if (!(compId in mergedContext)) {
        mergedContext[compId] = value;
      }
    }
    // Also add a flat "formData" object for easy server-side access
    if (this._inputValues.size > 0) {
      const formData: Record<string, unknown> = {};
      for (const [compId, value] of this._inputValues) {
        formData[compId] = value;
      }
      mergedContext['_formData'] = formData;
    }
    this.dispatchEvent(new CustomEvent('a2ui-action', {
      bubbles: true,
      composed: true,
      detail: { name, sourceComponentId: componentId, context: mergedContext },
    }));
  }

  static styles = css`
    :host { display: block; }

    .card {
      background: #fff;
      border: 1px solid #dadce0;
      border-radius: 12px;
      padding: 16px;
      box-shadow: 0 1px 3px rgba(0,0,0,.08);
    }

    .column { display: flex; flex-direction: column; gap: 6px; }
    .column[data-align="center"] { align-items: center; }
    .column[data-align="start"] { align-items: flex-start; }
    .column[data-align="end"] { align-items: flex-end; }

    .row { display: flex; flex-direction: row; gap: 8px; align-items: center; flex-wrap: wrap; }
    .row[data-justify="center"] { justify-content: center; }
    .row[data-justify="spaceAround"] { justify-content: space-around; }
    .row[data-justify="spaceBetween"] { justify-content: space-between; }
    .row[data-justify="end"] { justify-content: flex-end; }

    .text-h1 { font-size: 28px; font-weight: 500; }
    .text-h2 { font-size: 20px; font-weight: 500; }
    .text-h3 { font-size: 16px; font-weight: 500; }
    .text-subtitle { font-size: 15px; color: #5f6368; }
    .text-body { font-size: 14px; color: #3c4043; }
    .text-caption { font-size: 12px; color: #9aa0a6; }

    .btn {
      display: inline-flex; align-items: center; justify-content: center;
      padding: 8px 20px;
      border: 1px solid #dadce0;
      border-radius: 20px;
      background: #fff;
      color: #1a73e8;
      font-size: 14px; font-weight: 500;
      cursor: pointer;
      transition: background .15s, box-shadow .15s;
      font-family: inherit;
    }
    .btn:hover { background: #f0f4ff; box-shadow: 0 1px 4px rgba(26,115,232,.2); }
    .btn:active { background: #e0eaff; }
    .btn.primary, .btn.filled {
      background: #1a73e8; color: #fff; border-color: #1a73e8;
    }
    .btn.primary:hover, .btn.filled:hover { background: #1669d0; }
    .btn.outlined { background: #fff; color: #1a73e8; border-color: #1a73e8; }
    .btn.text { background: transparent; border: none; color: #1a73e8; }

    .divider { border: none; border-top: 1px solid #e0e0e0; margin: 8px 0; }
    .divider.vertical { border-top: none; border-left: 1px solid #e0e0e0; height: 100%; margin: 0 8px; }

    /* CheckBox */
    .checkbox-wrapper {
      display: flex; align-items: center; gap: 8px; cursor: pointer;
      padding: 4px 0; font-size: 14px; color: #3c4043;
    }
    .checkbox-wrapper input[type="checkbox"] {
      width: 18px; height: 18px; accent-color: #1a73e8; cursor: pointer;
    }

    /* Slider */
    .slider-wrapper { display: flex; flex-direction: column; gap: 4px; width: 100%; }
    .slider-wrapper label { font-size: 12px; color: #5f6368; }
    .slider-wrapper input[type="range"] {
      width: 100%; accent-color: #1a73e8; cursor: pointer;
    }
    .slider-value { font-size: 12px; color: #9aa0a6; text-align: right; }

    /* TextField */
    .textfield-wrapper { display: flex; flex-direction: column; gap: 4px; width: 100%; }
    .textfield-wrapper label { font-size: 12px; color: #5f6368; }
    .textfield-wrapper input, .textfield-wrapper textarea {
      padding: 8px 12px; border: 1px solid #dadce0; border-radius: 8px;
      font-size: 14px; font-family: inherit; outline: none;
      transition: border-color .15s;
    }
    .textfield-wrapper input:focus, .textfield-wrapper textarea:focus {
      border-color: #1a73e8;
    }

    /* Image */
    .a2ui-image { max-width: 100%; border-radius: 8px; }
    .a2ui-image.icon { width: 24px; height: 24px; border-radius: 0; }
    .a2ui-image.avatar { width: 40px; height: 40px; border-radius: 50%; object-fit: cover; }
    .a2ui-image.thumbnail { width: 80px; height: 80px; object-fit: cover; }
    .a2ui-image.banner { width: 100%; max-height: 200px; object-fit: cover; }
    .a2ui-image.smallFeature { width: 64px; height: 64px; object-fit: contain; }
    .a2ui-image.largeFeature { width: 100%; max-height: 280px; object-fit: contain; }

    /* Icon */
    .a2ui-icon { font-family: 'Material Symbols Outlined', sans-serif; font-size: 24px; color: #5f6368; }
    .a2ui-icon-emoji { font-size: 20px; }

    /* Tabs */
    .tabs-header { display: flex; border-bottom: 2px solid #e0e0e0; gap: 0; }
    .tab-btn {
      padding: 8px 16px; border: none; background: transparent;
      font-size: 14px; font-weight: 500; color: #5f6368;
      cursor: pointer; border-bottom: 2px solid transparent;
      margin-bottom: -2px; font-family: inherit;
    }
    .tab-btn.active { color: #1a73e8; border-bottom-color: #1a73e8; }
    .tab-btn:hover { background: #f0f4ff; }
    .tab-content { padding: 12px 0; }

    /* ChoicePicker */
    .choice-picker { display: flex; flex-direction: column; gap: 6px; }
    .choice-picker label.group-label { font-size: 12px; color: #5f6368; }
    .choice-option { display: flex; align-items: center; gap: 8px; font-size: 14px; color: #3c4043; cursor: pointer; }
    .choice-option input { accent-color: #1a73e8; }

    /* DateTimeInput */
    .datetime-wrapper { display: flex; flex-direction: column; gap: 4px; }
    .datetime-wrapper label { font-size: 12px; color: #5f6368; }
    .datetime-wrapper input {
      padding: 8px 12px; border: 1px solid #dadce0; border-radius: 8px;
      font-size: 14px; font-family: inherit;
    }

    /* Modal */
    .modal-overlay {
      position: fixed; top: 0; left: 0; right: 0; bottom: 0;
      background: rgba(0,0,0,.4); display: flex; align-items: center; justify-content: center;
      z-index: 1000;
    }
    .modal-content {
      background: #fff; border-radius: 12px; padding: 24px;
      max-width: 480px; width: 90%; box-shadow: 0 4px 24px rgba(0,0,0,.2);
    }

    /* List */
    .list-vertical { display: flex; flex-direction: column; gap: 4px; }
    .list-horizontal { display: flex; flex-direction: row; gap: 8px; flex-wrap: wrap; }

    /* AudioPlayer */
    .audio-wrapper { display: flex; flex-direction: column; gap: 4px; }
    .audio-wrapper .audio-desc { font-size: 12px; color: #5f6368; }
    .audio-wrapper audio { width: 100%; }

    /* Video */
    .a2ui-video { width: 100%; max-height: 360px; border-radius: 8px; }
  `;

  protected render() {
    if (!this.surface) return nothing;
    return this._renderComponent(this.surface.rootId);
  }

  private _renderComponent(id: string): TemplateResult | typeof nothing {
    const s = this.surface!;
    const comp = s.components.get(id);
    if (!comp) return nothing;

    const typeName = comp.component || comp.type || '';

    switch (typeName) {
      case 'Card':
        return this._renderCard(comp);
      case 'Column':
        return this._renderColumn(comp);
      case 'Row':
        return this._renderRow(comp);
      case 'Text':
        return this._renderText(comp);
      case 'Button':
        return this._renderButton(comp);
      case 'Divider':
        return html`<hr class="divider ${comp.axis === 'vertical' ? 'vertical' : ''}" />`;
      case 'CheckBox':
        return this._renderCheckBox(comp);
      case 'Slider':
        return this._renderSlider(comp);
      case 'TextField':
        return this._renderTextField(comp);
      case 'Image':
        return this._renderImage(comp);
      case 'Icon':
        return this._renderIcon(comp);
      case 'Tabs':
        return this._renderTabs(comp);
      case 'List':
        return this._renderList(comp);
      case 'Modal':
        return this._renderModal(comp);
      case 'ChoicePicker':
      case 'MultipleChoice':
        return this._renderChoicePicker(comp);
      case 'DateTimeInput':
        return this._renderDateTimeInput(comp);
      case 'Video':
        return this._renderVideo(comp);
      case 'AudioPlayer':
        return this._renderAudioPlayer(comp);
      default:
        // Try to render children if available
        if (comp.children) {
          return html`<div>${comp.children.map(c => this._renderComponent(c))}</div>`;
        }
        if (comp.child) {
          return this._renderComponent(comp.child);
        }
        return nothing;
    }
  }

  private _renderCard(comp: A2UIComponent) {
    return html`
      <div class="card">
        ${comp.child ? this._renderComponent(comp.child) : nothing}
      </div>
    `;
  }

  private _renderColumn(comp: A2UIComponent) {
    // Collect child IDs that are consumed as button.child references
    const consumed = this._getConsumedChildIds();
    const children = (comp.children || []).filter(id => !consumed.has(id));
    return html`
      <div class="column" data-align="${comp.align || ''}">
        ${children.map(id => this._renderComponent(id))}
      </div>
    `;
  }

  private _renderRow(comp: A2UIComponent) {
    const consumed = this._getConsumedChildIds();
    const children = (comp.children || []).filter(id => !consumed.has(id));
    return html`
      <div class="row" data-justify="${comp.justify || ''}">
        ${children.map(id => this._renderComponent(id))}
      </div>
    `;
  }

  private _renderText(comp: A2UIComponent) {
    const text = resolveText(comp.text as any, this.surface!.dataModel);
    const variant = comp.variant || 'body';
    return html`<span class="text-${variant}">${text}</span>`;
  }

  private _renderButton(comp: A2UIComponent) {
    // Resolve button label: child text component, or fallback
    let label = '';
    if (comp.child) {
      const childComp = this.surface!.components.get(comp.child);
      if (childComp) {
        label = resolveText(childComp.text as any, this.surface!.dataModel);
      }
    }
    if (!label) label = (comp.label as string) || (comp.text as string) || comp.id;

    const variant = (comp.variant as string) || '';
    const isPrimary = comp.primary === true || variant === 'filled';
    const btnClass = isPrimary ? 'primary' : variant || '';
    const onClick = () => {
      // 1) Explicit functionCall — handle locally per A2UI standard
      const fn = comp.action?.functionCall;
      if (fn) {
        this._handleFunctionCall(fn);
        return;
      }
      // 2) Event — but detect URLs in context and handle client-side
      //    LLMs often use event when they should use functionCall for URLs.
      //    Rather than relying on prompt engineering, the client handles it.
      const event = comp.action?.event;
      if (event) {
        const url = this._extractUrlFromEvent(event);
        if (url) {
          window.open(url, '_blank', 'noopener');
          return;
        }
        this._fireAction(
          event.name || 'unknown',
          comp.id,
          event.context || {},
        );
      }
    };

    return html`
      <button class="btn ${btnClass}" @click=${onClick}>
        ${label}
      </button>
    `;
  }

  // ── Client-side function calls (A2UI standard) ──
  private _handleFunctionCall(fn: { call: string; args?: Record<string, unknown> }) {
    switch (fn.call) {
      case 'openUrl': {
        const url = fn.args?.url as string;
        if (url) window.open(url, '_blank', 'noopener');
        break;
      }
      default:
        console.warn(`[A2UI] Unhandled client functionCall: ${fn.call}`, fn.args);
    }
  }

  /**
   * Detect URLs buried in event context — LLMs often put URLs in event
   * when they should use functionCall.openUrl. Client handles it anyway.
   */
  private _extractUrlFromEvent(
    event: { name?: string; context?: Record<string, unknown> },
  ): string | null {
    const ctx = event.context;
    if (!ctx) return null;
    // Check common patterns: mapUrl, url, link, href, etc.
    for (const val of Object.values(ctx)) {
      if (typeof val === 'string' && /^https?:\/\/.+/i.test(val)) {
        return val;
      }
    }
    return null;
  }

  // ── CheckBox ──
  private _renderCheckBox(comp: A2UIComponent) {
    const label = resolveText(comp.label as any, this.surface!.dataModel);
    const checked = resolveValue(comp.value, this.surface!.dataModel);
    const onChange = (e: Event) => {
      const target = e.target as HTMLInputElement;
      this._inputValues.set(comp.id, target.checked);
    };
    return html`
      <label class="checkbox-wrapper">
        <input type="checkbox" .checked=${!!checked} @change=${onChange} />
        ${label}
      </label>
    `;
  }

  // ── Slider ──
  private _renderSlider(comp: A2UIComponent) {
    const label = resolveText(comp.label as any, this.surface!.dataModel);
    const val = resolveValue(comp.value, this.surface!.dataModel);
    const min = comp.min ?? 0;
    const max = comp.max ?? 100;
    const onInput = (e: Event) => {
      const target = e.target as HTMLInputElement;
      this._inputValues.set(comp.id, Number(target.value));
    };
    return html`
      <div class="slider-wrapper">
        ${label ? html`<label>${label}</label>` : nothing}
        <input type="range" min=${min} max=${max} .value=${String(val ?? min)} @input=${onInput} />
        <span class="slider-value">${val ?? min} / ${max}</span>
      </div>
    `;
  }

  // ── TextField ──
  private _renderTextField(comp: A2UIComponent) {
    const label = resolveText(comp.label as any, this.surface!.dataModel);
    const val = resolveText(comp.text as any ?? comp.value as any, this.surface!.dataModel);
    const fieldType = comp.textFieldType || 'shortText';
    const onInput = (e: Event) => {
      const target = e.target as HTMLInputElement | HTMLTextAreaElement;
      this._inputValues.set(comp.id, target.value);
    };
    if (fieldType === 'longText' || fieldType === 'multiline') {
      return html`
        <div class="textfield-wrapper">
          ${label ? html`<label>${label}</label>` : nothing}
          <textarea rows="3" .value=${val} placeholder=${label} @input=${onInput}></textarea>
        </div>
      `;
    }
    const inputType = fieldType === 'obscured' || fieldType === 'password' ? 'password'
      : fieldType === 'number' ? 'number'
      : fieldType === 'date' ? 'date'
      : fieldType === 'email' ? 'email' : 'text';
    return html`
      <div class="textfield-wrapper">
        ${label ? html`<label>${label}</label>` : nothing}
        <input type=${inputType} .value=${val} placeholder=${label} @input=${onInput} />
      </div>
    `;
  }

  // ── Image ──
  private _renderImage(comp: A2UIComponent) {
    const url = resolveText(comp.url as any, this.surface!.dataModel);
    const hint = (comp.variant || comp.usageHint || '') as string;
    const fitStyle = comp.fit ? `object-fit: ${comp.fit}` : '';
    return html`<img class="a2ui-image ${hint}" src=${url} style=${fitStyle} alt="" />`;
  }

  // ── Icon ──
  private _renderIcon(comp: A2UIComponent) {
    const name = resolveText(comp.name as any, this.surface!.dataModel);
    // Convert camelCase to snake_case for Material Symbols
    const iconName = name.replace(/([A-Z])/g, '_$1').toLowerCase().replace(/^_/, '');
    // Emoji fallback map for common weather/UI icons
    const emojiMap: Record<string, string> = {
      'cloud': '☁️', 'sunny': '☀️', 'clear': '☀️', 'sun': '☀️',
      'umbrella': '☂️', 'rain': '🌧️', 'rainy': '🌧️', 'snow': '❄️',
      'thunderstorm': '⛈️', 'fog': '🌫️', 'wind': '💨', 'partly_cloudy': '⛅',
      'partly_cloudy_day': '⛅', 'partly_cloudy_night': '⛅',
      'check': '✅', 'close': '❌', 'star': '⭐', 'favorite': '❤️',
      'home': '🏠', 'settings': '⚙️', 'search': '🔍', 'info': 'ℹ️',
      'warning': '⚠️', 'error': '❗', 'calendar': '📅', 'schedule': '📅',
      'location': '📍', 'place': '📍', 'restaurant': '🍽️', 'music': '🎵',
      'play': '▶️', 'pause': '⏸️', 'stop': '⏹️',
    };
    const emoji = emojiMap[iconName];
    if (emoji) {
      return html`<span class="a2ui-icon-emoji">${emoji}</span>`;
    }
    return html`<span class="material-symbols-outlined a2ui-icon">${iconName}</span>`;
  }

  // ── Tabs ──
  private _renderTabs(comp: A2UIComponent) {
    const items = comp.tabItems || comp.tabs || [];
    if (items.length === 0) return nothing;

    // Use first tab as active by default (no state management in this simple renderer)
    const activeIdx = 0;
    return html`
      <div>
        <div class="tabs-header">
          ${items.map((tab, i) => {
            const label = resolveText((tab.title || (tab as any).label) as any, this.surface!.dataModel);
            return html`<button class="tab-btn ${i === activeIdx ? 'active' : ''}">${label}</button>`;
          })}
        </div>
        <div class="tab-content">
          ${this._renderComponent(items[activeIdx].child)}
        </div>
      </div>
    `;
  }

  // ── List ──
  private _renderList(comp: A2UIComponent) {
    const dir = comp.direction === 'horizontal' ? 'horizontal' : 'vertical';
    const children = comp.children || [];

    // Data-bound template: {"componentId": "...", "path": "/items"}
    if (!Array.isArray(children) && typeof children === 'object') {
      const binding = children as { componentId?: string; path?: string };
      if (binding.componentId && binding.path) {
        const templateComp = this.surface!.components.get(binding.componentId);
        const items = resolveValue({ path: binding.path }, this.surface!.dataModel);
        if (templateComp && Array.isArray(items)) {
          return html`
            <div class="list-${dir}">
              ${items.map((item, idx) => this._renderTemplateInstance(templateComp, item, idx))}
            </div>
          `;
        }
      }
      return nothing;
    }

    if (Array.isArray(children)) {
      return html`
        <div class="list-${dir}">
          ${children.map(id => this._renderComponent(id))}
        </div>
      `;
    }
    return nothing;
  }

  /** Render a template component instance with item-scoped data model. */
  private _renderTemplateInstance(
    templateComp: A2UIComponent,
    itemData: Record<string, unknown>,
    index: number,
  ): TemplateResult | typeof nothing {
    // Create a temporary sub-surface with item data as the data model
    // and render the template's children against it
    const s = this.surface!;
    const savedDataModel = s.dataModel;
    // Merge item data so both {path: "/title"} and {path: "/current/title"} resolve correctly.
    // "/current" is a common LLM convention for referencing the current iteration item.
    s.dataModel = { ...savedDataModel, ...itemData, current: itemData, _index: index };
    try {
      // Render the template component itself (Card, Column, Row, etc.)
      // This handles both child and children patterns correctly.
      return this._renderComponent(templateComp.id);
    } finally {
      s.dataModel = savedDataModel;
    }
  }

  // ── Modal ──
  private _renderModal(comp: A2UIComponent) {
    const triggerId = comp.entryPointChild || comp.trigger || '';
    const contentId = comp.contentChild || comp.content || '';
    // Render trigger inline; modal content is hidden until interaction (simplified: always show trigger)
    return html`
      <div>
        ${triggerId ? this._renderComponent(triggerId) : nothing}
      </div>
    `;
    // Full modal would need state — omitted for simplicity, trigger click is enough for testing
  }

  // ── ChoicePicker / MultipleChoice ──
  private _renderChoicePicker(comp: A2UIComponent) {
    const label = resolveText(comp.label as any, this.surface!.dataModel);
    const options = (comp.options || []) as Array<{ label: string | { path: string }; value: string }>;
    const variant = (comp.variant || 'radio') as string;
    const isMulti = variant === 'multipleSelection' || variant === 'chip' || comp.component === 'MultipleChoice';
    const inputType = isMulti ? 'checkbox' : 'radio';
    const groupName = `choice-${comp.id}`;

    const onChange = (e: Event) => {
      const target = e.target as HTMLInputElement;
      if (isMulti) {
        const current = (this._inputValues.get(comp.id) as string[]) || [];
        if (target.checked) {
          this._inputValues.set(comp.id, [...current, target.value]);
        } else {
          this._inputValues.set(comp.id, current.filter(v => v !== target.value));
        }
      } else {
        this._inputValues.set(comp.id, target.value);
      }
    };

    return html`
      <div class="choice-picker">
        ${label ? html`<label class="group-label">${label}</label>` : nothing}
        ${options.map(opt => {
          const optLabel = resolveText(opt.label as any, this.surface!.dataModel);
          return html`
            <label class="choice-option">
              <input type=${inputType} name=${groupName} value=${opt.value} @change=${onChange} />
              ${optLabel}
            </label>
          `;
        })}
      </div>
    `;
  }

  // ── DateTimeInput ──
  private _renderDateTimeInput(comp: A2UIComponent) {
    const label = resolveText(comp.label as any, this.surface!.dataModel);
    const val = resolveText(comp.value as any, this.surface!.dataModel);
    const enableDate = comp.enableDate !== false;
    const enableTime = comp.enableTime === true;
    const inputType = enableDate && enableTime ? 'datetime-local'
      : enableTime ? 'time' : 'date';
    return html`
      <div class="datetime-wrapper">
        ${label ? html`<label>${label}</label>` : nothing}
        <input type=${inputType} .value=${val} />
      </div>
    `;
  }

  // ── Video ──
  private _renderVideo(comp: A2UIComponent) {
    const url = resolveText(comp.url as any, this.surface!.dataModel);
    return html`<video class="a2ui-video" src=${url} controls></video>`;
  }

  // ── AudioPlayer ──
  private _renderAudioPlayer(comp: A2UIComponent) {
    const url = resolveText(comp.url as any, this.surface!.dataModel);
    const desc = resolveText(comp.description as any, this.surface!.dataModel);
    return html`
      <div class="audio-wrapper">
        ${desc ? html`<span class="audio-desc">${desc}</span>` : nothing}
        <audio src=${url} controls></audio>
      </div>
    `;
  }

  /** Collect IDs referenced by Button.child so they don't render twice. */
  private _getConsumedChildIds(): Set<string> {
    const consumed = new Set<string>();
    for (const comp of this.surface!.components.values()) {
      const typeName = comp.component || comp.type || '';
      if (typeName === 'Button' && comp.child) {
        consumed.add(comp.child);
      }
    }
    return consumed;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'a2ui-surface-v09': A2UISurfaceElement;
  }
}
