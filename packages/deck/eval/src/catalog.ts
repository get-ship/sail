// Primitive catalog — the typed shape of the data the prompt feeds the
// model. Today we ship a hand-written mock; once the Phase-4 compiler
// emits a custom-elements manifest, `loadCatalog` will read it instead.

export type PropType = 'string' | 'bool' | 'number' | 'union'

export interface PropDef {
  name: string
  type: PropType
  values?: readonly string[]
  default?: string | number | boolean
  bindable?: boolean
}

export interface SlotDef {
  name: string
  summary?: string
}

export interface EventDef {
  name: string
  summary?: string
}

export interface PrimitiveDef {
  tag: string
  summary?: string
  props: readonly PropDef[]
  slots: readonly SlotDef[]
  events: readonly EventDef[]
}

export interface PrimitiveCatalog {
  version: string
  primitives: readonly PrimitiveDef[]
}

/**
 * Load the active primitive catalog. The Phase-4 compiler will swap
 * the implementation for a CEM reader; for now we return the mock.
 */
export async function loadCatalog(): Promise<PrimitiveCatalog> {
  const { mockCatalog } = await import('./mock-catalog.ts')
  return mockCatalog
}

/**
 * Validate a JSON-shaped object against the catalog schema. Returns
 * the typed catalog or throws with a precise field path.
 */
export function parseCatalog(raw: unknown): PrimitiveCatalog {
  if (!isObject(raw)) throw new Error('catalog: expected object at root')
  if (typeof raw['version'] !== 'string') {
    throw new Error('catalog: version must be a string')
  }
  if (!Array.isArray(raw['primitives'])) {
    throw new Error('catalog: primitives must be an array')
  }
  const primitives = (raw['primitives'] as unknown[]).map((p, i) =>
    parsePrimitive(p, `primitives[${i}]`),
  )
  return { version: raw['version'], primitives }
}

function parsePrimitive(raw: unknown, path: string): PrimitiveDef {
  if (!isObject(raw)) throw new Error(`${path}: expected object`)
  if (typeof raw['tag'] !== 'string') {
    throw new Error(`${path}.tag: must be a string`)
  }
  const props = Array.isArray(raw['props'])
    ? raw['props'].map((p, i) => parseProp(p, `${path}.props[${i}]`))
    : []
  const slots = Array.isArray(raw['slots'])
    ? raw['slots'].map((s, i) => parseSlot(s, `${path}.slots[${i}]`))
    : []
  const events = Array.isArray(raw['events'])
    ? raw['events'].map((e, i) => parseEvent(e, `${path}.events[${i}]`))
    : []
  const def: PrimitiveDef = { tag: raw['tag'], props, slots, events }
  if (typeof raw['summary'] === 'string') {
    return { ...def, summary: raw['summary'] }
  }
  return def
}

function parseProp(raw: unknown, path: string): PropDef {
  if (!isObject(raw)) throw new Error(`${path}: expected object`)
  if (typeof raw['name'] !== 'string') {
    throw new Error(`${path}.name: must be a string`)
  }
  const type = raw['type']
  if (
    type !== 'string' &&
    type !== 'bool' &&
    type !== 'number' &&
    type !== 'union'
  ) {
    throw new Error(`${path}.type: must be string|bool|number|union`)
  }
  const out: PropDef = { name: raw['name'], type }
  if (type === 'union') {
    if (!Array.isArray(raw['values']) || raw['values'].length === 0) {
      throw new Error(`${path}.values: union requires non-empty values[]`)
    }
    out.values = raw['values'].map(String)
  }
  if (raw['default'] !== undefined) {
    out.default = raw['default'] as string | number | boolean
  }
  if (raw['bindable'] === true) out.bindable = true
  return out
}

function parseSlot(raw: unknown, path: string): SlotDef {
  if (!isObject(raw)) throw new Error(`${path}: expected object`)
  if (typeof raw['name'] !== 'string') {
    throw new Error(`${path}.name: must be a string`)
  }
  const out: SlotDef = { name: raw['name'] }
  if (typeof raw['summary'] === 'string') out.summary = raw['summary']
  return out
}

function parseEvent(raw: unknown, path: string): EventDef {
  if (!isObject(raw)) throw new Error(`${path}: expected object`)
  if (typeof raw['name'] !== 'string') {
    throw new Error(`${path}.name: must be a string`)
  }
  const out: EventDef = { name: raw['name'] }
  if (typeof raw['summary'] === 'string') out.summary = raw['summary']
  return out
}

function isObject(v: unknown): v is Record<string, unknown> {
  return typeof v === 'object' && v !== null && !Array.isArray(v)
}
