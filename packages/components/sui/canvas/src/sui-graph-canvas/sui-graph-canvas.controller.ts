import { controller, bindHtml } from '@ship/sui-runtime'
import type { Scope } from './sui-graph-canvas.sui'

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

function esc(s: string): string {
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function coerce<T>(v: unknown, fallback: T[]): T[] {
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

function computePositions(
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

function bezierPath(
  a: { x: number; y: number },
  b: { x: number; y: number },
): string {
  const dx = Math.max(40, Math.abs(b.x - a.x) * 0.4)
  return `M ${a.x} ${a.y} C ${a.x + dx} ${a.y}, ${b.x - dx} ${b.y}, ${b.x} ${b.y}`
}

function kindTemplateFor(host: HTMLElement, kind: string): HTMLTemplateElement | null {
  const tpls = host.querySelectorAll('template[slot="node"]')
  for (const t of Array.from(tpls)) {
    if ((t as HTMLElement).dataset.kind === kind) return t as HTMLTemplateElement
  }
  return null
}

function nodeBodyHtml(host: HTMLElement, n: GraphNode): string {
  const tpl = kindTemplateFor(host, n.kind)
  if (!tpl) {
    const sub = n.sub ? `<div class="node-sub">${esc(n.sub)}</div>` : ''
    return `<div class="node-h"><span class="tag">${esc(n.kind)}</span></div><div class="node-title">${esc(n.title ?? n.id)}</div>${sub}`
  }
  return tpl.innerHTML
    .replace(/\$\{title\}/g, esc(n.title ?? ''))
    .replace(/\$\{sub\}/g, esc(n.sub ?? ''))
    .replace(/\$\{id\}/g, esc(n.id))
}

function renderGraphHtml(
  host: HTMLElement,
  nodesRaw: string,
  edgesRaw: string,
  width: number,
  height: number,
  layout: string,
  selectedId: string,
): string {
  const nodes = coerce<GraphNode>(nodesRaw, [])
  const edges = coerce<GraphEdge>(edgesRaw, [])
  const pos = computePositions(nodes, width, height, layout)

  const edgePaths = edges
    .map((e) => {
      const a = pos.get(e.from)
      const b = pos.get(e.to)
      if (!a || !b) return ''
      const d = esc(bezierPath(a, b))
      const dashed = e.dashed ? ' data-dashed' : ''
      return `<path d="${d}" data-kind="${esc(e.kind ?? 'default')}"${dashed}></path>`
    })
    .join('')

  const nodeDivs = nodes
    .map((n) => {
      const p = pos.get(n.id)
      if (!p) return ''
      const left = (p.x / width) * 100
      const top = (p.y / height) * 100
      const sel = selectedId && selectedId === n.id ? ' data-selected' : ''
      const wStyle = n.w ? `--node-w: ${n.w}px;` : ''
      const label = esc(`${n.kind}: ${n.title ?? n.id}`)
      const body = nodeBodyHtml(host, n)
      return `<div class="node" part="node" data-kind="${esc(n.kind)}" data-id="${esc(n.id)}"${sel} tabindex="0" role="button" aria-label="${label}" style="left: ${left}%; top: ${top}%;${wStyle}">${body}</div>`
    })
    .join('')

  return (
    `<svg class="edges" part="edges" viewBox="0 0 ${width} ${height}" preserveAspectRatio="none" aria-hidden="true">${edgePaths}</svg>` +
    nodeDivs
  )
}

export default controller<Scope>({
  mount(state, host) {
    const canvas = state.canvasEl as HTMLElement | null
    if (!canvas) return

    const inner = canvas.ownerDocument.createElement('div')
    canvas.appendChild(inner)

    const stopRender = bindHtml(inner, () => {
      const nodesRaw = state.nodes as string
      const edgesRaw = state.edges as string
      const width = (state.width as number) || 1000
      const height = (state.height as number) || 600
      const layout = (state.layout as string) || 'positioned'
      const selectedId = state.selectedId as string
      return renderGraphHtml(host, nodesRaw, edgesRaw, width, height, layout, selectedId)
    })

    let lastHoveredId = ''

    const onClick = (e: MouseEvent) => {
      const node = (e.target as HTMLElement).closest<HTMLElement>('.node[data-id]')
      if (!node) return
      const id = node.getAttribute('data-id') ?? ''
      const kind = node.getAttribute('data-kind') ?? ''
      host.setAttribute('selected-id', id)
      host.dispatchEvent(
        new CustomEvent('sui-node-select', {
          detail: { id, kind },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== 'Enter' && e.key !== ' ') return
      const node = (e.target as HTMLElement).closest<HTMLElement>('.node[data-id]')
      if (!node) return
      e.preventDefault()
      const id = node.getAttribute('data-id') ?? ''
      const kind = node.getAttribute('data-kind') ?? ''
      host.setAttribute('selected-id', id)
      host.dispatchEvent(
        new CustomEvent('sui-node-select', {
          detail: { id, kind },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const onMouseOver = (e: MouseEvent) => {
      const node = (e.target as HTMLElement).closest<HTMLElement>('.node[data-id]')
      if (!node) {
        lastHoveredId = ''
        return
      }
      const id = node.getAttribute('data-id') ?? ''
      if (id === lastHoveredId) return
      lastHoveredId = id
      const kind = node.getAttribute('data-kind') ?? ''
      host.dispatchEvent(
        new CustomEvent('sui-node-hover', {
          detail: { id, kind },
          bubbles: true,
          composed: true,
        }),
      )
    }

    canvas.addEventListener('click', onClick)
    canvas.addEventListener('keydown', onKeyDown)
    canvas.addEventListener('mouseover', onMouseOver)

    return () => {
      stopRender()
      canvas.removeEventListener('click', onClick)
      canvas.removeEventListener('keydown', onKeyDown)
      canvas.removeEventListener('mouseover', onMouseOver)
    }
  },
})
