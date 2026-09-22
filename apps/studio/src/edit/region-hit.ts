/**
 * Preview region hit-test — map click coords (relative to painted image)
 * to SailInspect node ids via SailRender region boxes.
 */

import type { RenderRegion } from './types'

export function parseRegions(raw: unknown[]): RenderRegion[] {
  if (!Array.isArray(raw)) return []
  const out: RenderRegion[] = []
  for (const r of raw) {
    if (!r || typeof r !== 'object') continue
    const o = r as Record<string, unknown>
    const id = String(o.nodeId ?? o.id ?? o.node_id ?? '')
    if (!id) continue
    out.push({
      nodeId: id,
      x: Number(o.x ?? 0),
      y: Number(o.y ?? 0),
      w: Number(o.w ?? o.width ?? 0),
      h: Number(o.h ?? o.height ?? 0),
      ...(o.tag != null ? { tag: String(o.tag) } : {}),
      ...(o.label != null ? { label: String(o.label) } : {}),
    })
  }
  return out
}

/**
 * Pick the smallest-area region containing (x, y).
 * Coordinates are in the same space as SailRender regions (viewport px).
 */
export function hitTestRegion(
  regions: RenderRegion[],
  x: number,
  y: number,
): RenderRegion | null {
  let best: RenderRegion | null = null
  let bestArea = Infinity
  for (const r of regions) {
    if (x < r.x || y < r.y || x > r.x + r.w || y > r.y + r.h) continue
    const area = Math.max(1, r.w) * Math.max(1, r.h)
    if (area < bestArea) {
      bestArea = area
      best = r
    }
  }
  return best
}

/** Scale click from displayed image size → render viewport coords. */
export function scaleClick(
  clientX: number,
  clientY: number,
  imgRect: { left: number; top: number; width: number; height: number },
  viewport: { width: number; height: number },
): { x: number; y: number } {
  const relX = clientX - imgRect.left
  const relY = clientY - imgRect.top
  const sx = imgRect.width > 0 ? viewport.width / imgRect.width : 1
  const sy = imgRect.height > 0 ? viewport.height / imgRect.height : 1
  return { x: relX * sx, y: relY * sy }
}
