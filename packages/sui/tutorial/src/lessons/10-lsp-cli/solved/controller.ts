import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const on = signal(false)
    const label = () => (on() ? 'on' : 'off')
    return {
      label,
      toggle: (): void => on.set(!on()),
    }
  },
})
