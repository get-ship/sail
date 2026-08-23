<!-- Fixture: SSR tier C (client) — interval-driven signal. -->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
export default controller((_ctx: ControllerScope<any>) => {
  const ticks = signal(0)
  effect(() => {
    const t = setInterval(() => ticks.set((n) => n + 1), 1000)
    return () => clearInterval(t)
  })
  return { ticks }
})
</controller>
<column gap="2">
  <p>{ticks()}</p>
  <p>live {ticks()}</p>
</column>
