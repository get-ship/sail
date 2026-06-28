import { controller, signal, effect } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const endpoint = signal('/api/users')
    const requestId = signal(0)
    const status = signal<'idle' | 'loading' | 'done' | 'aborted'>('idle')
    const lastEndpoint = signal('')

    // Re-run on every endpoint change. Cleanup aborts the in-flight
    // simulated request so stale responses never land.
    effect(() => {
      const url = endpoint()
      const id = requestId() + 1
      requestId.set(id)
      status.set('loading')

      let cancelled = false
      const timer = window.setTimeout(() => {
        if (cancelled) return
        status.set('done')
        lastEndpoint.set(url)
      }, 1200)

      return () => {
        cancelled = true
        window.clearTimeout(timer)
        // Only log "aborted" if we haven't completed yet.
        if (status() === 'loading') status.set('aborted')
      }
    })

    return { endpoint, requestId, status, lastEndpoint }
  },
})
