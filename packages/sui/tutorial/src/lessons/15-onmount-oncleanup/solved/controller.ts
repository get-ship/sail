import { controller, signal, effect, onCleanup } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const elapsed = signal(0)
    const paused = signal(false)
    const start = performance.now()

    console.log('[clock] mounted')

    // Recreate the interval when `paused` flips — the effect's
    // cleanup tears down the old interval before the next run.
    effect(() => {
      if (paused()) return
      const id = window.setInterval(() => elapsed.set(elapsed() + 1), 1000)
      return () => window.clearInterval(id)
    })

    // Cleanups fire in reverse-registration order: "second" then "first".
    onCleanup(() => console.log('[clock] first cleanup'))
    onCleanup(() => console.log('[clock] second cleanup'))
    onCleanup(() => console.log(`[clock] unmounted after ${Math.round(performance.now() - start)}ms`))

    return { elapsed, paused }
  },
})
