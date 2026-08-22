/** Pure graph-canvas helpers. */
export interface GraphNode {
  id: string
  kind: string
  title?: string
  sub?: string
  x?: number
  y?: number
  w?: number
  data?: Record<string, unknown>
}
export interface GraphEdge {
  from: string
  to: string
  kind?: string
  dashed?: boolean
}
export type GraphHost = {
  querySelectorAll(s: string): ArrayLike<GraphHost>
  dataset?: Record<string, string | undefined>
  innerHTML: string
  [k: string]: unknown
}
export const EDGE_STROKE: Record<string, string> = {
  default: 'var(--border)',
  success: 'var(--success)',
  warning: 'var(--warning)',
  danger: 'var(--danger)',
  info: 'var(--info)',
}
export const EDGE_DEFAULT = 'var(--border)'

export function esc(s: string): string {
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

export function coerce<T>(v: unknown, fallback: T[]): T[] {
  if (Array.isArray(v)) return v as T[]
  if (typeof v === 'string' && v.trim()) {
    try {
      const parsed = JSON.parse(v)
      return Array.isArray(parsed) ? (parsed as T[]) : fallback
    } catch {
      return fallback
    }
  }
  return fallback
}

export function computePositions(
  nodes: GraphNode[],
  width: number,
  height: number,
  layout: string,
): Map<string, { x: number; y: number }> {
  const map = new Map<string, { x: number; y: number }>()
  const missing = nodes.some((n) => typeof n.x !== 'number' || typeof n.y !== 'number')
  if (layout === 'positioned' && !missing) {
    for (const n of nodes) map.set(n.id, { x: n.x as number, y: n.y as number })
    return map
  }
  const cx = width * 0.22
  const cy = height * 0.5
  const centers = nodes.filter((n) => n.kind === 'agent')
  const satellites = nodes.filter((n) => n.kind !== 'agent')
  centers.forEach((n, i) => {
    const span = Math.max(1, centers.length)
    const y = cy + (i - (span - 1) / 2) * 80
    map.set(n.id, { x: cx, y })
  })
  const byKind = new Map<string, GraphNode[]>()
  for (const n of satellites) {
    const list = byKind.get(n.kind) ?? []
    list.push(n)
    byKind.set(n.kind, list)
  }
  const kinds = Array.from(byKind.keys())
  kinds.forEach((kind, bandIdx) => {
    const list = byKind.get(kind)!
    const bandX = width * (0.5 + ((bandIdx + 1) / (kinds.length + 1)) * 0.45)
    list.forEach((n, i) => {
      const y = (height * (i + 1)) / (list.length + 1)
      map.set(n.id, { x: bandX, y })
    })
  })
  return map
}

export function bezierPath(
  a: { x: number; y: number },
  b: { x: number; y: number },
): string {
  const dx = Math.max(40, Math.abs(b.x - a.x) * 0.4)
  return `M ${a.x} ${a.y} C ${a.x + dx} ${a.y}, ${b.x - dx} ${b.y}, ${b.x} ${b.y}`
}

// The per-kind edge palette. Each kind reads its host knob first, so a consumer
// can restyle one relation type without touching the rest.


export function kindTemplateFor(host: GraphHost, kind: string): GraphHost | null {
  const tpls = host.querySelectorAll('template[slot="node"]')
  const list = Array.from(tpls as ArrayLike<GraphHost>)
  for (let i = 0; i < list.length; i++) {
    const t = list[i]
    if (!t) continue
    if (t.dataset?.kind === kind) return t
  }
  return null
}

/** A per-kind <template> is consumer-authored markup with ${title}/${sub}/${id}
 *  placeholders. '' when the kind has no template — the card renders the
 *  default chrome the template block declares. */
export function nodeBodyHtml(host: GraphHost | null, n: GraphNode): string {
  if (!host) return ''
  const tpl = kindTemplateFor(host, n.kind)
  if (!tpl) return ''
  return tpl.innerHTML
    .replace(/\$\{title\}/g, esc(n.title ?? ''))
    .replace(/\$\{sub\}/g, esc(n.sub ?? ''))
    .replace(/\$\{id\}/g, esc(n.id))
}

