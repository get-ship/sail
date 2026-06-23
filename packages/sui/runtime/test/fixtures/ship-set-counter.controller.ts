import { controller, signal, derived } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    const count = signal(0)
    const doubled = derived(() => count() * 2)
    return { count, doubled }
  },
})
