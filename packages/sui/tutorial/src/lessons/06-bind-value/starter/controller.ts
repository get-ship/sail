import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const name = signal('')
    const quantity = signal('1')
    // TODO: add agreed = signal(false), size = signal<'S'|'M'|'L'>('M')

    return { name, quantity }
  },
})
