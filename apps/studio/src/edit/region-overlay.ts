/** Imperative region highlight layer over the static preview image. */
export type RegionBoxUi = {
  id: string
  left: string
  top: string
  width: string
  height: string
  selected: boolean
  key: string
}

export function paintRegionOverlay(boxes: RegionBoxUi[]): void {
  if (typeof document === 'undefined') return
  const host = document.querySelector('[data-region-overlay]') as HTMLElement | null
  if (!host) return
  let layer = host.querySelector('[data-region-layer]') as HTMLElement | null
  if (!layer) {
    layer = document.createElement('div')
    layer.setAttribute('data-region-layer', '')
    layer.style.cssText = 'position:absolute;inset:0;pointer-events:none;z-index:1'
    host.appendChild(layer)
  }
  layer.replaceChildren()
  for (const box of boxes) {
    const el = document.createElement('div')
    el.dataset.regionBox = ''
    el.dataset.nodeId = box.id
    el.dataset.selected = box.selected ? 'true' : 'false'
    el.style.cssText = `position:absolute;left:${box.left};top:${box.top};width:${box.width};height:${box.height};border:1px solid var(--primary);border-radius:4px;box-sizing:border-box;${box.selected ? 'background:color-mix(in oklab,var(--primary) 20%,transparent)' : ''}`
    layer.appendChild(el)
  }
}
