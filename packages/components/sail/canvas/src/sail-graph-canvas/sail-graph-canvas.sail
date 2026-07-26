<!--
  <sail-graph-canvas> — SAIL port of <ship-graph-canvas>.
  Generic positioned-graph primitive. Renders an SVG edge layer with bezier
  paths plus absolutely-positioned node cards.

  Prop / event parity vs <ship-graph-canvas>
  ─────────────────────────────────────────────────────────────────────────────
  Lit                           SAIL                          Notes
  ─────────────────────────────────────────────────────────────────────────────
  nodes (array|string, [])      nodes (string, '')            ✓ JSON-encoded
  edges (array|string, [])      edges (string, '')            ✓ JSON-encoded
  width (number, 1000, reflect) width (number, 1000, reflect) ✓
  height (number, 600, reflect) height (number, 600, reflect) ✓
  layout ('positioned'|'radial' layout (union, 'positioned',  ✓
    , 'positioned', reflect)      reflect)
  selectedId (string, '',       selected-id (string, '',      ✓
    reflect, attribute:           reflect)
    'selected-id')
  ─────────────────────────────────────────────────────────────────────────────
  ship-node-select {id, kind}   sail-node-select {id, kind}    renamed
  ship-node-hover  {id, kind}   sail-node-hover  {id, kind}    renamed
  ─────────────────────────────────────────────────────────────────────────────
  Slots: node (hidden; per-kind <template slot="node" data-kind="...">)
  ─────────────────────────────────────────────────────────────────────────────
  Anatomy: the surface holds one <svg> edge layer and one 0×0 anchor per node;
  each anchor centers a role=button card carrying data-id (identity), data-state
  (kind) and aria-current (selection).
  ─────────────────────────────────────────────────────────────────────────────

  GEOMETRY
  The `width`/`height` props are the graph's coordinate space. The edge <svg>
  carries them as its viewBox and sits IN FLOW, so its intrinsic ratio gives
  the surface its height and node percentages resolve against the same box.
  Node cards are HTML (they must not scale with the surface), positioned by
  percentage through the style: channel — runtime geometry, not a skin.
  Each card hangs in a 0×0 anchor + <center>, which lands the card's midpoint
  on its (x, y) — the point the layout actually computed.

  Edge stroke colors stay consumer-overridable knobs (`--edge-agent` … on the
  host); the paths read them with a token fallback.
-->

<scope>
  props: {
    nodes = ''
    edges = ''
    width = 1000
    height = 600
    layout: 'positioned' | 'radial' = 'positioned'
    'selected-id' = ''
  }
</scope>

<controller>
import { controller, signal, derived } from '@ship/sail-runtime'
import { bindHtml } from '@ship/sail-runtime/internal'
import type { Action } from '@ship/sail-runtime'
import type { Scope } from './sail-graph-canvas.sail'

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

/** One node card, resolved to what the template paints. */
interface NodeView {
  id: string
  kind: string
  title: string
  sub: string
  label: string
  left: string
  top: string
  /** null = the card keeps its default width (the skin's --panel-min-w-sm). */
  width: string | null
  selected: boolean
  /** Consumer-authored body HTML from a per-kind <template>; '' = default chrome. */
  body: string
}

interface EdgeView {
  key: string
  d: string
  kind: string
  stroke: string
  dash: string | null
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

// The per-kind edge palette. Each kind reads its host knob first, so a consumer
// can restyle one relation type without touching the rest.
const EDGE_STROKE: Record<string, string> = {
  agent: 'var(--edge-agent, var(--primary))',
  skill: 'var(--edge-skill, var(--success))',
  mcp: 'var(--edge-mcp, var(--info))',
  rule: 'var(--edge-rule, var(--warning))',
  perm: 'var(--edge-perm, var(--destructive))',
  harness: 'var(--edge-harness, var(--accent))',
}
const EDGE_DEFAULT = 'var(--edge-default, var(--border))'

function kindTemplateFor(host: HTMLElement, kind: string): HTMLTemplateElement | null {
  const tpls = host.querySelectorAll('template[slot="node"]')
  for (const t of Array.from(tpls)) {
    if ((t as HTMLElement).dataset.kind === kind) return t as HTMLTemplateElement
  }
  return null
}

/** A per-kind <template> is consumer-authored markup with ${title}/${sub}/${id}
 *  placeholders. '' when the kind has no template — the card renders the
 *  default chrome the template block declares. */
function nodeBodyHtml(host: HTMLElement | null, n: GraphNode): string {
  if (!host) return ''
  const tpl = kindTemplateFor(host, n.kind)
  if (!tpl) return ''
  return tpl.innerHTML
    .replace(/\$\{title\}/g, esc(n.title ?? ''))
    .replace(/\$\{sub\}/g, esc(n.sub ?? ''))
    .replace(/\$\{id\}/g, esc(n.id))
}

export default controller(({ nodes, edges, width, height, layout, selectedId }: Scope, host?: Element) => {
    // The per-kind <template>s are the host's own light-DOM children, and a
    // node card's body is fixed at FIRST render (a keyed row re-binds its own
    // attributes, not a nested branch), so the host must be read here — the
    // resolver passes it as the setup function's 2nd arg. `mount()` re-seeds
    // it once the instance actually connects, for the same reason `mount:`
    // used to: `host` can be undefined at setup time.
    const hostEl = signal<HTMLElement | null>((host as HTMLElement) ?? null)
    mount((el) => { hostEl.set(el) })

    const graph = derived(() => {
      const w = width() || 1000
      const h = height() || 600
      const ns = coerce<GraphNode>(nodes(), [])
      return { w, h, ns, pos: computePositions(ns, w, h, layout() || 'positioned') }
    })

    const nodeViews = derived<NodeView[]>(() => {
      const { w, h, ns, pos } = graph()
      const host = hostEl()
      const sel = selectedId()
      const out: NodeView[] = []
      for (const n of ns) {
        const p = pos.get(n.id)
        if (!p) continue
        out.push({
          id: n.id,
          kind: n.kind,
          title: n.title ?? n.id,
          sub: n.sub ?? '',
          label: `${n.kind}: ${n.title ?? n.id}`,
          left: `${(p.x / w) * 100}%`,
          top: `${(p.y / h) * 100}%`,
          width: n.w ? `${n.w}px` : null,
          selected: sel !== '' && sel === n.id,
          body: nodeBodyHtml(host, n),
        })
      }
      return out
    })

    const edgeViews = derived<EdgeView[]>(() => {
      const { pos } = graph()
      const out: EdgeView[] = []
      coerce<GraphEdge>(edges(), []).forEach((e, i) => {
        const a = pos.get(e.from)
        const b = pos.get(e.to)
        if (!a || !b) return
        const kind = e.kind ?? 'default'
        out.push({
          key: `${e.from}${e.to}${i}`,
          d: bezierPath(a, b),
          kind,
          stroke: EDGE_STROKE[kind] ?? EDGE_DEFAULT,
          dash: e.dashed ? '4 3' : null,
        })
      })
      return out
    })

    // Consumer-authored node markup — foreign HTML, so it goes through the one
    // sanctioned reactive-innerHTML path (morph-based, identity-safe).
    const nodeBody: Action<string> = (el, html) => {
      const src = signal(html)
      const stop = bindHtml(el, () => src())
      return { update: (next: string) => src.set(next), destroy: stop }
    }

    const emit = (name: string, id: string, kind: string): void => {
      hostEl()?.dispatchEvent(
        new CustomEvent(name, { detail: { id, kind }, bubbles: true, composed: true }),
      )
    }

    const select = (n: NodeView): void => {
      hostEl()?.setAttribute('selected-id', n.id)
      emit('sail-node-select', n.id, n.kind)
    }

    const activate = (n: NodeView, e: KeyboardEvent): void => {
      if (e.key !== 'Enter' && e.key !== ' ') return
      e.preventDefault()
      select(n)
    }

    // Hover fires on ENTERING a node — a mouseover from a child element inside
    // the same card is the same hover, not a new one.
    let lastHoveredId = ''
    const hover = (n: NodeView): void => {
      if (n.id === lastHoveredId) return
      lastHoveredId = n.id
      emit('sail-node-hover', n.id, n.kind)
    }

    return { hostEl, nodeViews, edgeViews, nodeBody, select, activate, hover }
})

</controller>
<div sx={style.position(relative).w(full).bg(card).borderWidth(1).border(border).radius(md).overflow(hidden)}>
  <svg
    viewBox={`0 0 ${width() || 1000} ${height() || 600}`}
    preserveAspectRatio="none"
    aria-hidden="true"
    sx={style.display(block).w(full).h(auto).pointerEvents(none)}
  >
    <for as="e" of={edgeViews()} key={e.key}>
      <path
        d={e.d}
        data-kind={e.kind}
        data-dashed={e.dash ? true : null}
        fill="none"
        stroke={e.stroke}
        stroke-width="1.5"
        stroke-dasharray={e.dash}
        opacity="0.85"
      ></path>
    </for>
  </svg>

  <for as="n" of={nodeViews()} key={n.id}>
    <div use:rect={{left: n.left, top: n.top}} sx={style.position(absolute).w(0).h(0).left(bind(boxLeft)).top(bind(boxTop))}>
      <center sx={style.h(0)}>
        <column
          gap="0"
          role="button"
          tabindex="0"
          data-id={n.id}
          data-state={n.kind}
          aria-label={n.label}
          aria-current={n.selected ? 'true' : 'false'}
          use:rect={{width: n.width}}
          on:click={() => select(n)}
          on:keydown={(e) => activate(n, e)}
          on:mouseover={() => hover(n)}
          sx={style.w(bind(boxW)).padY(2).padX(3).bg(card).fg(foreground).fontFamily(body).fontSize(sm).borderWidth(1).border(border).radius(md).shadow(xs).cursor(pointer).when({ hover: style.shadow(sm), focus: style.outline(ring), current: style.borderWidth(1).border(ring).shadow(sm), stateAgent: style.borderLWidth(1).borderL(primary), stateSkill: style.borderLWidth(1).borderL(success), stateMcp: style.borderLWidth(1).borderL(info), stateRule: style.borderLWidth(1).borderL(warning), statePerm: style.borderLWidth(1).borderL(destructive), stateHarness: style.borderLWidth(1).borderL(accent) })}
        >
          <if cond={n.body !== ''}>
            <div use:nodeBody={n.body}></div>
          <else>
            <row gap="2" align="center" sx={style.mb(1)}>
              <span sx={style.fontSize(xs).fontWeight(semibold).tracking(wide).textCase(uppercase).fg(muted-foreground)}>{n.kind}</span>
            </row>
            <div sx={style.fontWeight(semibold).fg(foreground)}>{n.title}</div>
            <if cond={n.sub !== ''}>
              <div sx={style.fontSize(xs).fg(muted-foreground)}>{n.sub}</div>
            </if>
          </else></if>
        </column>
      </center>
    </div>
  </for>
</div>
<slot name="node" hidden></slot>
