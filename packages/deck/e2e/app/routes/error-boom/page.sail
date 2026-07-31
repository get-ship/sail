<controller>
import { controller, effect } from '@ship/sail-runtime'

// Deliberately throws from an effect on mount, with no
// <sail-error-boundary> anywhere in this route — the no-boundary path that
// used to reach only console.error. Proves the deck dev overlay renders.
export default controller(() => {
    effect(() => {
      throw new Error('deliberate boom for overlay e2e')
    })
    return {}
  })
</controller>

<div data-error-boom="true">
  <h1>error boom</h1>
</div>
