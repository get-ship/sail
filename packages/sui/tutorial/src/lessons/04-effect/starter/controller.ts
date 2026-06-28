import { controller, signal, effect } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const count = signal(0)

    // Mirror count into the document title.
    effect(() => {
      document.title = `count = ${count()}`
    })

    // TODO 1: persist count to localStorage on every change.
    // TODO 2: hydrate count from localStorage on first run.

    return {
      count,
      bump: (n: number): void => count.set(count() + n),
      reset: (): void => count.set(0),
    }
  },
})
