// Shared types and helpers for sail-annotation-* components.
// Types mirror runtime::events::annotation::AnnotationEvent (exact JSON field names).

export type AnnotationKind = 'point' | 'region'

export interface AnnotationTarget {
  view_id?: string
  selector?: string
  rect?: [number, number, number, number]
}

export interface AnnotationEvent {
  id: string
  kind: AnnotationKind
  target: AnnotationTarget
  note?: string
  created_at: string
  author_kind: 'human' | 'agent'
}

export interface BuildAnnotationInput {
  viewId?: string | undefined
  kind: AnnotationKind
  selector?: string | undefined
  rect?: [number, number, number, number] | undefined
  note?: string | undefined
}

// ─── ULID ──────────────────────────────────────────────────────────────────

const CROCKFORD = '0123456789ABCDEFGHJKMNPQRSTVWXYZ'

export function newUlid(now: number = Date.now()): string {
  let ts = now
  const time = new Array<string>(10)
  for (let i = 9; i >= 0; i--) {
    time[i] = CROCKFORD[ts % 32]!
    ts = Math.floor(ts / 32)
  }
  const rand = new Array<string>(16)
  const cryptoObj = typeof globalThis !== 'undefined'
    ? (globalThis as { crypto?: any }).crypto
    : undefined
  if (cryptoObj?.getRandomValues) {
    const buf = new Uint8Array(16)
    cryptoObj.getRandomValues(buf)
    for (let i = 0; i < 16; i++) rand[i] = CROCKFORD[buf[i]! % 32]!
  } else {
    for (let i = 0; i < 16; i++) rand[i] = CROCKFORD[Math.floor(Math.random() * 32)]!
  }
  return time.join('') + rand.join('')
}

// ─── Event builder ──────────────────────────────────────────────────────────

/** HTTP POST for a staged annotation. Lives here so the controller never names `fetch`. */
export async function postAnnotationEvent(
  endpoint: string,
  ev: AnnotationEvent,
): Promise<{ ok: boolean; status: number }> {
  const resp = await fetch(endpoint, {
    method: 'POST',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(ev),
  })
  return { ok: resp.ok, status: resp.status }
}

export function buildAnnotationEvent(input: BuildAnnotationInput): AnnotationEvent {
  const target: AnnotationTarget = {}
  if (input.viewId) target.view_id = input.viewId
  if (input.selector) target.selector = input.selector
  if (input.rect) target.rect = input.rect
  return {
    id: newUlid(),
    kind: input.kind,
    target,
    ...(input.note !== undefined ? { note: input.note } : {}),
    created_at: new Date().toISOString(),
    author_kind: 'human',
  }
}

// ─── CSS selector ──────────────────────────────────────────────────────────

export function getCSSSelector(el: any): string {
  if (el.id) return `#${el.id}`
  const tag = el.tagName.toLowerCase()
  if (el.className && typeof el.className === 'string') {
    const classes = el.className.trim().split(/\s+/).slice(0, 3).join('.')
    if (classes) return `${tag}.${classes}`
  }
  return tag
}
