<controller>
import type { ControllerScope } from '@ship/sail-runtime'
export default controller((_ctx: ControllerScope<any>) => {
  const pure = signal(0)
  return { pure }
})
</controller>
<column gap="2">
  <span>{pure()}</span>
</column>
