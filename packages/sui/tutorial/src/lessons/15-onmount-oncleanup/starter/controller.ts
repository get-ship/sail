import { controller, signal, effect, onCleanup } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const elapsed = signal(0)

    // Mount work runs here — no onMount needed.
    const start = performance.now()
    console.log('[clock] mounted')

    // Effect with returned cleanup — fires per tick, cleanup on unmount.
    effect(() => {
      const id = window.setInterval(() => elapsed.set(elapsed() + 1), 1000)
      return () => window.clearInterval(id)
    })

    // Explicit unmount work.
    onCleanup(() => {
      console.log(`[clock] unmounted after ${Math.round(performance.now() - start)}ms`)
    })

    return { elapsed }
  },
})
