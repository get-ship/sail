<!-- Fixture: SSR tier I — non-literal signal init (interactive / hydrate). -->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
export default controller((_ctx: ControllerScope<any>) => {
  const seed = signal(Math.random())
  const label = signal('n=' + String(seed()))
  return { seed, label }
})
</controller>
<column gap="2">
  <p>{seed()}</p>
  <p>{label()}</p>
</column>
