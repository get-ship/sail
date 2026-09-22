<controller>
import { controller, signal } from '@ship/sail-runtime'

export default controller((ctx) => ({ count: signal(0) }))
</controller>

<div>
  <p>{coun()}</p>
</div>
