<!-- Query match with else arm (state kind) against api.demo.clock. -->

<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style title = { fg(foreground).type({ size: 2xl, weight: bold }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
  style error = { fg(destructive).type({ size: sm }) }
  style state = { fg(muted-foreground).type({ size: sm, family: mono }) }
</scope>

<controller>
import { controller, query } from '@ship/sail-runtime'
import { api } from '../../src/compositions/demo-clients'

export default controller(() => ({
  t: query(() => api.demo.clock()),
}))
</controller>

<column gap="3" style(page)>
  <h1 style(title)>Live Feed</h1>
  <match {t()}>
    {loading => <p style(muted)>Loading…</p>}
    {error => <p style(error)>Error</p>}
    {else => <p style(state)>state: {t().kind}</p>}
  </match>
</column>
