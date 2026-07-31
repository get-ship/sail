<controller>
import type { ControllerScope } from '@ship/sail-runtime'
export default controller((_ctx: ControllerScope<any>) => {
  // Non-literal signal init → SSR tier I (interactive / hydrate)
  const seed = signal(Math.random())
  const label = signal('n=' + String(seed()))
  return { seed, label }
})
</controller>
<column gap="2">
  <span>{seed()}</span>
  <p>{label()}</p>
</column>
