<!-- fixture for CM tokenizer + explorer -->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
export default controller((_ctx: ControllerScope<any>) => {
  const ticks = signal(0)
  return { ticks }
})
</controller>
<column gap="4">
  <card tone="primary" sx={style.pad(6).radius(lg)}>
    <h3>Static, by default</h3>
    <p>Every route renders without JavaScript.</p>
    <row gap="2">
      <button type="button" tone="primary">Get started</button>
      <button type="button" tone="secondary">Read the docs</button>
    </row>
    {ticks().toFixed(0)}
  </card>
</column>
