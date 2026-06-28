import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

type State = 'idle' | 'loading' | 'done' | 'error'

export default controller<Scope>({
  scope: () => {
    const state = signal<State>('idle')

    return {
      state,
      start: (): void => {
        state.set('loading')
        setTimeout(() => state.set(Math.random() < 0.3 ? 'error' : 'done'), 700)
      },
      reset: (): void => state.set('idle'),
    }
  },
})
