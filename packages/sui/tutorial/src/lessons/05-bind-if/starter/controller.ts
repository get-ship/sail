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
        setTimeout(() => state.set('done'), 700)
      },
      // TODO: add a `reset` action that flips state back to 'idle'.
      // TODO: also wire an error branch — simulate a failure with
      //       Math.random() < 0.3 inside start().
    }
  },
})
