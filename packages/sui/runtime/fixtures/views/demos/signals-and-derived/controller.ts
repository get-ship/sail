import { controller, signal, derived } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => {
    const count = signal(0)
    const history = signal<Array<{ k: number; value: number }>>([])
    const double = derived(() => count() * 2)
    let nextKey = 1

    const push = (next: number) => {
      count.set(next)
      history.set([{ k: nextKey++, value: next }, ...history()].slice(0, 10))
    }

    return {
      count,
      double,
      history,
      inc: () => push(count() + 1),
      dec: () => push(count() - 1),
      reset: () => {
        count.set(0)
        history.set([])
      },
    }
  },
})
