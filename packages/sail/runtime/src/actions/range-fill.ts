import type { Action } from '../action'

/**
 * `use:rangeFill` — tick labels only. WebKit track fill is compiled
 * emission / platform CSS, not `--rng-pct` from JS.
 */
export const rangeFill: Action<void> = (node) => {
  const el = node as HTMLInputElement
  let tickRow: HTMLElement | null = null

  const syncTicks = (): void => {
    const labels = (el.getAttribute('ticks') ?? '')
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean)
    if (!labels.length) {
      tickRow?.remove()
      tickRow = null
      return
    }
    if (!tickRow || !tickRow.isConnected) {
      tickRow = document.createElement('div')
      tickRow.setAttribute('data-slider-ticks', '')
      tickRow.setAttribute('aria-hidden', 'true')
      el.after(tickRow)
    }
    tickRow.replaceChildren(
      ...labels.map((label) => {
        const span = document.createElement('span')
        span.textContent = label
        return span
      }),
    )
  }

  syncTicks()
  const obs = new MutationObserver(syncTicks)
  obs.observe(el, {
    attributes: true,
    attributeFilter: ['ticks', 'size'],
  })

  return {
    destroy: () => {
      obs.disconnect()
      tickRow?.remove()
      tickRow = null
    },
  }
}
