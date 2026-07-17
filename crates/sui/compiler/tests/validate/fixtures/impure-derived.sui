<controller>
import { controller, derived } from '@ship/sail-runtime'

export default controller((ctx) => {
  const x = derived(() => Date.now())
  return { x }
})
</controller>

<p>{x()}</p>
