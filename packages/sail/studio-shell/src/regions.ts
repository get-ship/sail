
import type { BridgeRect } from './channel'

export type RegionNode = {
  /** Span-keyed or provisional locate id */
  nodeId: string
  label: string
  selector?: string
}

export type ResolvedRegion = {
  rect: BridgeRect
  nodes: RegionNode[]
  /** true when SailRender regions metadata was used */
  fromRender: boolean
  imagePng?: string
}

const STUDIO_SKIP = new Set(['sail-studio-root', 'sail-studio-fab'])

function isStudioChrome(el: Element | null): boolean {
  if (!el) return true
  if (el.id && STUDIO_SKIP.has(el.id)) return true
  if (el.closest?.('#sail-studio-root, #sail-studio-fab, [data-sail-studio]')) return true
  return false
}

function cssPath(el: Element): string {
  if (el.id && !STUDIO_SKIP.has(el.id)) return `#${CSS.escape(el.id)}`
  const tag = el.tagName.toLowerCase()
  const p = el.parentElement
  if (!p || p === document.documentElement) return tag
  const sibs = Array.from(p.children).filter((c) => c.tagName === el.tagName)
  if (sibs.length === 1) return `${cssPath(p)} > ${tag}`
  return `${cssPath(p)} > ${tag}:nth-of-type(${sibs.indexOf(el) + 1})`
}

function labelFor(el: Element): string {
  const tag = el.tagName.toLowerCase()
  const name =
    el.getAttribute('data-sail-host') ||
    el.getAttribute('name') ||
    el.id ||
    el.classList?.[0] ||
    ''
  return name ? `<${tag}> ${name}` : `<${tag}>`
}

/**
 * Locate-live: sample points inside the rect and collect unique elements.
 * Does not measure layout for assessment — only identity for selection join.
 */
export function resolveRectToNodes(rect: BridgeRect): RegionNode[] {
  if (typeof document === 'undefined') return []
  const nodes: RegionNode[] = []
  const seen = new Set<Element>()
  const steps = 3
  for (let iy = 0; iy < steps; iy++) {
    for (let ix = 0; ix < steps; ix++) {
      const x = rect.left + ((ix + 0.5) / steps) * rect.width
      const y = rect.top + ((iy + 0.5) / steps) * rect.height
      const el = document.elementFromPoint(x, y)
      if (!el || isStudioChrome(el) || seen.has(el)) continue
      // Prefer nearest element with a stable-ish identity under app root.
      let cur: Element | null = el
      while (cur && isStudioChrome(cur)) cur = cur.parentElement
      if (!cur || seen.has(cur)) continue
      seen.add(cur)
      const selector = cssPath(cur)
      nodes.push({
        nodeId: selector,
        label: labelFor(cur),
        selector,
      })
    }
  }
  return nodes
}

/**
 * Join box against SailRender regions metadata (static assess).
 * Region entries are expected as `{ id, x, y, w, h }` or `{ nodeId, box }`.
 */
export function joinRectToRenderRegions(
  rect: BridgeRect,
  regions: unknown[],
): RegionNode[] {
  const out: RegionNode[] = []
  for (const raw of regions) {
    if (!raw || typeof raw !== 'object') continue
    const r = raw as Record<string, unknown>
    const id = String(r.id ?? r.nodeId ?? r.node_id ?? '')
    if (!id) continue
    const box = (r.box ?? r.rect ?? r) as Record<string, unknown>
    const x = Number(box.x ?? box.left ?? 0)
    const y = Number(box.y ?? box.top ?? 0)
    const w = Number(box.w ?? box.width ?? 0)
    const h = Number(box.h ?? box.height ?? 0)
    // Intersection test in viewport space (render coords may be 1:1 for default vp).
    const overlap =
      rect.left < x + w &&
      rect.left + rect.width > x &&
      rect.top < y + h &&
      rect.top + rect.height > y
    if (!overlap) continue
    const selector = typeof r.selector === 'string' ? r.selector : undefined
    out.push({
      nodeId: id,
      label: String(r.label ?? r.tag ?? id),
      ...(selector !== undefined ? { selector } : {}),
    })
  }
  return out
}

export type RenderShot = {
  imagePng: string
  regions: unknown[]
}

/**
 * Optional SailRender call — only when `path` is known.
 * Fail-soft: returns null so locate-live still works offline.
 */
export async function sailRenderRegion(
  path: string,
  viewport: { width: number; height: number },
  theme = 'sail',
  mode = 'dark',
): Promise<RenderShot | null> {
  try {
    const { sailRenderScreenshot } = await import('@ship/sail-studio-client')
    const ret = await sailRenderScreenshot({
      path,
      theme,
      mode,
      viewport: `${viewport.width}x${viewport.height}`,
    })
    return {
      imagePng: ret.imagePng,
      regions: ret.regions ?? [],
    }
  } catch {
    return null
  }
}

export async function resolveRegion(
  rect: BridgeRect,
  opts?: {
    sailPath?: string | null
    viewport?: { width: number; height: number }
    theme?: string
    mode?: string
  },
): Promise<ResolvedRegion> {
  const live = resolveRectToNodes(rect)
  if (opts?.sailPath) {
    const shot = await sailRenderRegion(
      opts.sailPath,
      opts.viewport ?? { width: 1440, height: 900 },
      opts.theme ?? 'sail',
      opts.mode ?? 'dark',
    )
    if (shot) {
      const joined = joinRectToRenderRegions(rect, shot.regions)
      return {
        rect,
        nodes: joined.length ? joined : live,
        fromRender: joined.length > 0,
        imagePng: shot.imagePng,
      }
    }
  }
  return { rect, nodes: live, fromRender: false }
}
