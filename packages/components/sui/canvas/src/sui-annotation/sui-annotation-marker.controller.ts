import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-annotation-marker.sui'

export default controller<Scope>({
  mount(state, _host) {

    const stop = effect(() => {
      const pinEl = state.$signals.pinEl() as HTMLElement | null
      const boxEl = state.$signals.boxEl() as HTMLElement | null
      const x = state.$signals.x() as number
      const y = state.$signals.y() as number
      const idx = state.$signals.index() as number
      const rect = state.$signals.rect() as string

      if (pinEl) {
        pinEl.style.left = `${x}px`
        pinEl.style.top = `${y}px`
        pinEl.textContent = String(idx + 1)
      }

      if (boxEl && rect) {
        const parts = rect.split(',').map(Number)
        if (parts.length === 4 && parts.every((n) => Number.isFinite(n))) {
          const [rx, ry, rw, rh] = parts as [number, number, number, number]
          boxEl.style.cssText = `left:${rx}px;top:${ry}px;width:${rw}px;height:${rh}px`
        }
      }
    })

    return stop
  },
})
