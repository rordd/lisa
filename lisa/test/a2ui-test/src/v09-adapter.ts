/**
 * A2UI v0.9 → v0.8 message adapter.
 *
 * Server emits v0.9 messages (createSurface, updateComponents, updateDataModel).
 * The official @a2ui/lit renderer (v0.8) expects v0.8 messages
 * (beginRendering, surfaceUpdate, dataModelUpdate).
 *
 * This adapter bridges the gap until @a2ui/lit ships v0.9 support.
 */

// ── v0.9 types (server side) ──

interface V09Message {
  version?: string;
  createSurface?: { surfaceId: string; catalogId?: string };
  updateComponents?: { surfaceId?: string; components: V09Component[] };
  updateDataModel?: { surfaceId?: string; path?: string; value: unknown };
  deleteSurface?: { surfaceId: string };
}

interface V09Component {
  id: string;
  component?: string;   // e.g. "Text", "Card", "Button"
  type?: string;         // alternative to component
  child?: string;
  children?: string[];
  text?: string | { path: string };
  variant?: string;
  align?: string;
  justify?: string;
  action?: {
    event?: {
      name?: string;
      context?: Record<string, unknown>;
    };
  };
  [key: string]: unknown;
}

// ── v0.8 types (renderer side) ──

interface V08Message {
  beginRendering?: { surfaceId: string; root: string; styles?: Record<string, string> };
  surfaceUpdate?: { surfaceId: string; components: V08ComponentInstance[] };
  dataModelUpdate?: { surfaceId: string; path?: string; contents: V08ValueMap[] };
  deleteSurface?: { surfaceId: string };
}

interface V08ComponentInstance {
  id: string;
  weight?: number;
  component?: Record<string, unknown>;  // { "Text": { text: { literalString: "..." } } }
}

interface V08ValueMap {
  key: string;
  valueString?: string;
  valueNumber?: number;
  valueBoolean?: boolean;
  valueMap?: V08ValueMap[];
}

let lastSurfaceId = '@default';

/**
 * Wrap a string-or-path value into v0.8 StringValue format.
 * v0.9: "hello" or { path: "/x" }
 * v0.8: { literalString: "hello" } or { path: "/x" }
 */
function toStringValue(val: unknown): Record<string, unknown> | undefined {
  if (val == null) return undefined;
  if (typeof val === 'string') {
    return { literalString: val };
  }
  if (typeof val === 'object' && (val as any).path) {
    return { path: (val as any).path };
  }
  return { literalString: String(val) };
}

/**
 * Convert v0.9 children array to v0.8 children format.
 * v0.9: ["id1", "id2"]
 * v0.8: { explicitList: ["id1", "id2"] }
 */
function toChildren(arr: string[]): Record<string, unknown> {
  return { explicitList: arr };
}

/**
 * Convert v0.9 action to v0.8 action format.
 * v0.9: { event: { name: "x", context: { key: "val" } } }
 * v0.8: { name: "x", context: [{ key: "key", value: { literalString: "val" } }] }
 */
function toAction(action: V09Component['action']): Record<string, unknown> | undefined {
  if (!action) return undefined;
  const event = action.event;
  if (!event) return undefined;

  const result: Record<string, unknown> = {};
  if (event.name) result.name = event.name;

  if (event.context && typeof event.context === 'object') {
    result.context = Object.entries(event.context).map(([key, val]) => {
      const entry: Record<string, unknown> = { key };
      if (typeof val === 'string') {
        entry.value = { literalString: val };
      } else if (typeof val === 'number') {
        entry.value = { literalNumber: val };
      } else if (typeof val === 'boolean') {
        entry.value = { literalBoolean: val };
      } else {
        entry.value = { literalString: JSON.stringify(val) };
      }
      return entry;
    });
  }

  return result;
}

/**
 * Map v0.9 variant to v0.8 usageHint.
 */
function toUsageHint(variant: string | undefined): string | undefined {
  if (!variant) return undefined;
  const map: Record<string, string> = {
    h1: 'h1', h2: 'h2', h3: 'h3', h4: 'h4', h5: 'h5',
    caption: 'caption', body: 'body',
    subtitle: 'h3',  // closest mapping
    title: 'h2',
  };
  return map[variant] || 'body';
}

/**
 * Convert a single v0.9 component to v0.8 ComponentInstance format.
 *
 * v0.9: { id, component: "Text", text: "hello", variant: "h1" }
 * v0.8: { id, component: { "Text": { text: { literalString: "hello" }, usageHint: "h1" } } }
 */
function convertComponent(c: V09Component): V08ComponentInstance {
  const typeName = c.component || c.type || 'Unknown';
  const props: Record<string, unknown> = {};

  // Handle properties based on component type
  switch (typeName) {
    case 'Text': {
      const textVal = toStringValue(c.text);
      if (textVal) props.text = textVal;
      const hint = toUsageHint(c.variant as string);
      if (hint) props.usageHint = hint;
      break;
    }

    case 'Card': {
      if (c.child) props.child = c.child;
      break;
    }

    case 'Button': {
      if (c.child) props.child = c.child;
      if (c.action) props.action = toAction(c.action);
      if (c.primary != null) props.primary = c.primary;
      break;
    }

    case 'Row':
    case 'Column': {
      if (c.children) props.children = toChildren(c.children);
      // v0.9 uses justify/align, v0.8 uses distribution/alignment
      if (c.justify) props.distribution = c.justify;
      if (c.align) props.alignment = c.align;
      break;
    }

    case 'Image': {
      const url = toStringValue(c.url as unknown);
      if (url) props.url = url;
      if (c.fit) props.fit = c.fit;
      if (c.usageHint) props.usageHint = c.usageHint;
      break;
    }

    case 'Icon': {
      const name = toStringValue(c.name as unknown);
      if (name) props.name = name;
      break;
    }

    case 'Divider': {
      if (c.axis) props.axis = c.axis;
      break;
    }

    case 'List': {
      if (c.children) props.children = toChildren(c.children);
      break;
    }

    default: {
      // Pass through unknown props (excluding meta fields)
      const skip = new Set(['id', 'component', 'type', 'version']);
      for (const [k, v] of Object.entries(c)) {
        if (!skip.has(k) && v !== undefined) {
          props[k] = v;
        }
      }
      break;
    }
  }

  return {
    id: c.id,
    component: { [typeName]: props },
  };
}

/**
 * Convert a v0.9 A2UI message to v0.8 format.
 */
export function convertV09ToV08(msg: V09Message): V08Message | null {
  if (msg.createSurface) {
    const sid = msg.createSurface.surfaceId || '@default';
    lastSurfaceId = sid;
    return {
      beginRendering: {
        surfaceId: sid,
        root: 'root',
      },
    };
  }

  if (msg.updateComponents) {
    const sid = msg.updateComponents.surfaceId || lastSurfaceId;
    const components = msg.updateComponents.components.map(convertComponent);
    return {
      surfaceUpdate: { surfaceId: sid, components },
    };
  }

  if (msg.updateDataModel) {
    const sid = msg.updateDataModel.surfaceId || lastSurfaceId;
    const path = msg.updateDataModel.path || '/';
    const contents = valueToContents(msg.updateDataModel.value);
    return {
      dataModelUpdate: { surfaceId: sid, path, contents },
    };
  }

  if (msg.deleteSurface) {
    return { deleteSurface: msg.deleteSurface };
  }

  return null;
}

/**
 * Convert a v0.9 value (arbitrary JSON object) to v0.8 ValueMap[] format.
 */
function valueToContents(value: unknown): V08ValueMap[] {
  if (value == null) return [];
  if (typeof value !== 'object' || Array.isArray(value)) {
    return [{ key: '', valueString: JSON.stringify(value) }];
  }
  const obj = value as Record<string, unknown>;
  return Object.entries(obj).map(([key, val]) => {
    const entry: V08ValueMap = { key };
    if (typeof val === 'string') {
      entry.valueString = val;
    } else if (typeof val === 'number') {
      entry.valueNumber = val;
    } else if (typeof val === 'boolean') {
      entry.valueBoolean = val;
    } else if (typeof val === 'object' && val !== null && !Array.isArray(val)) {
      entry.valueMap = valueToContents(val);
    } else {
      entry.valueString = JSON.stringify(val);
    }
    return entry;
  });
}

/**
 * Convert an array of v0.9 messages to v0.8 format.
 */
export function convertMessages(v09Messages: V09Message[]): V08Message[] {
  return v09Messages
    .map(convertV09ToV08)
    .filter((m): m is V08Message => m !== null);
}
