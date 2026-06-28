import { controller, signal, effect } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

const KEY = 'demo:count'

function readInitial(): number {
  const raw = typeof localStorage !== 'undefined' ? localStorage.getItem(KEY) : null
  const n = raw != null ? Number(raw) : 0
  return Number.isFinite(n) ? n : 0
}

export default controller<Scope>({
  scope: () => {
    const count = signal(readInitial())
    const elapsed = signal(0)

    effect(() => {
      document.title = `count = ${count()}`
    })

    effect(() => {
      localStorage.setItem(KEY, String(count()))
    })

    effect(() => {
      const id = window.setInterval(() => elapsed.set(elapsed() + 1), 1000)
      return () => window.clearInterval(id)
    })

    return {
      count, elapsed,
      bump: (n: number): void => count.set(count() + n),
      reset: (): void => { count.set(0); elapsed.set(0) },
    }
  },
})
