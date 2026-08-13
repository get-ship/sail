<!-- sail-graph-canvas Pure controller -->
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
import { bindHtml, emit } from '@ship/sail-runtime/internal'
import type { Action } from '@ship/sail-runtime'
import type { Scope } from './sail-graph-canvas.sail'
import {
  coerce, computePositions, bezierPath, nodeBodyHtml,
  EDGE_STROKE, EDGE_DEFAULT,
  type GraphNode, type GraphEdge,
} from './graph-helpers'

interface NodeView {
  id: string; kind: string; title: string; sub: string; label: string
  left: string; top: string; width: string | null; selected: boolean; body: string
}
interface EdgeView {
  key: string; d: string; kind: string; stroke: string; dash: string | null
}

export default controller(({ nodes, edges, width, height, layout, selectedId }: Scope) => {
    const hostEl = signal<HostBag | null>(null)
    mount((el) => { hostEl.set(el) })

    const graph = derived(() => {
      const w = width() || 1000
      const h = height() || 600
      const ns = coerce(nodes(), []) as GraphNode[]
      return { w, h, ns, pos: computePositions(ns, w, h, layout() || 'positioned') }
    })

    const nodeViews = derived(() => {
      const { w, h, ns, pos } = graph()
      const sel = selectedId() || ''
      const host = hostEl()
      const out: NodeView[] = []
      for (const n of ns) {
        const p = pos.get(n.id)
        if (!p) continue
        out.push({
          id: n.id, kind: n.kind, title: n.title ?? n.id, sub: n.sub ?? '',
          label: `${n.kind}: ${n.title ?? n.id}`,
          left: `${(p.x / w) * 100}%`, top: `${(p.y / h) * 100}%`,
          width: n.w ? `${n.w}px` : null,
          selected: sel !== '' && sel === n.id,
          body: nodeBodyHtml(host, n),
        })
      }
      return out
    })

    const edgeViews = derived(() => {
      const { pos } = graph()
      const out: EdgeView[] = []
      const edgeList = coerce(edges(), []) as GraphEdge[]
      for (let i = 0; i < edgeList.length; i++) {
        const e = edgeList[i]!
        const a = pos.get(e.from)
        const b = pos.get(e.to)
        if (!a || !b) continue
        const kind = e.kind ?? 'default'
        out.push({
          key: `${e.from}${e.to}${i}`,
          d: bezierPath(a, b),
          kind,
          stroke: EDGE_STROKE[kind] ?? EDGE_DEFAULT,
          dash: e.dashed ? '4 3' : null,
        })
      }
      return out
    })

    const nodeBody: Action<string> = (el, html) => {
      const src = signal(html)
      const stop = bindHtml(el, () => src())
      return { update: (next: string) => src.set(next), destroy: stop }
    }

    const emitNode = (name: string, id: string, kind: string): void => {
      const h = hostEl()
      if (h) emit(h, name, { id, kind })
    }
    const select = (n: NodeView): void => {
      hostSetAttr(hostEl(), 'selected-id', n.id)
      emitNode('sail-node-select', n.id, n.kind)
    }
    const activate = (n: NodeView, e: DomEv): void => {
      if (e.key !== 'Enter' && e.key !== ' ') return
      e.preventDefault()
      select(n)
    }
    let lastHoveredId = ''
    const hover = (n: NodeView): void => {
      if (n.id === lastHoveredId) return
      lastHoveredId = n.id
      emitNode('sail-node-hover', n.id, n.kind)
    }

    return { hostEl, nodeViews, edgeViews, nodeBody, select, activate, hover }
  })
</controller>

<div style({ box: place({ position: relative }).size({ w: full }).overflow({ all: hidden }), paint: fill({ color: surface-1 }).border({ color: border, width: 1 }).radius(md) })>
  <svg
    viewBox={`0 0 ${width() || 1000} ${height() || 600}`}
    preserveAspectRatio="none"
    aria-hidden="true" style({ box: flow({ display: block }).size({ w: full, h: auto }), paint: pointerEvents(none) })>
    <for {edgeViews() as e} key={e.key}>
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

  <for {nodeViews() as n} key={n.id}>
    <div use:rect={{left: n.left, top: n.top}} style({ box: place({ position: absolute, top: bind(boxTop), left: bind(boxLeft) }).size({ w: 0, h: 0 }) })>
      <center style({ box: size({ h: 0 }) })>
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

         style({ box: size({ w: bind(boxW) }).space({ pad-x: 3, pad-y: 2 }), paint: fill({ color: surface-1 }).ink({ color: foreground }).border({ color: border, width: 1 }).radius(md).elevation(xs).cursor(pointer), when: { hover: { paint: elevation(sm) }, focus: { paint: border({ kind: outline, color: ring }) }, current: { paint: border({ color: ring, width: 1 }).elevation(sm) }, stateAgent: { paint: border({ edge: l, color: primary, width: 1 }) }, stateSkill: { paint: border({ edge: l, color: success, width: 1 }) }, stateMcp: { paint: border({ edge: l, color: info, width: 1 }) }, stateRule: { paint: border({ edge: l, color: warning, width: 1 }) }, statePerm: { paint: border({ edge: l, color: destructive, width: 1 }) }, stateHarness: { paint: border({ edge: l, color: accent, width: 1 }) } } })>

          <if {n.body !== ''}>
            <div use:nodeBody={n.body}></div>
          {else => <row gap="2" align="center" style({ box: space({ mb: 1 }) })>
              <span style({ paint: ink({ color: muted-foreground }), text: type({ family: body, size: xs, weight: semibold, tracking: wide, case: uppercase }) })>{n.kind}</span>
            </row>
            <div style({ paint: ink({ color: foreground }) })>              <span style({ box: flow({ display: contents }), text: type({ family: body, size: sm, weight: semibold }) })>                {n.title}
              </span>
</div>
            <if {n.sub !== ''}>
              <div style({ paint: ink({ color: muted-foreground }) })>                <span style({ box: flow({ display: contents }), text: type({ family: body, size: xs }) })>                  {n.sub}
                </span>
</div>
            </if>}</if>
        </column>
      </center>
    </div>
  </for>
</div>
<slot name="node" hidden></slot>
