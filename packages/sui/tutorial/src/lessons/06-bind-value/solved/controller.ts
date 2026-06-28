import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const name = signal('')
    const quantity = signal('1')
    const size = signal<'S' | 'M' | 'L'>('M')
    const agreed = signal(false)

    return { name, quantity, size, agreed }
  },
})
