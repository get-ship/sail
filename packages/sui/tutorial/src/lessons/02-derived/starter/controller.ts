import { controller, signal, derived } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const count = signal(0)
    const name = signal('friend')
    const doubled = derived(() => count() * 2)
    const greeting = derived(() => `hello, ${name()}`)

    return {
      count, name, doubled, greeting,
      bump: (n: number): void => count.set(count() + n),
      setName: (v: string): void => name.set(v),
    }
  },
})
