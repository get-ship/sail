<!-- Query match with else arm (state kind) against api.demo.clock. -->

<controller>
import { controller, query } from '@ship/sail-runtime'
import { api } from '../../src/compositions/demo-clients'

export default controller(() => ({
  t: query(() => api.demo.clock()),
}))
</controller>

<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style title = { paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) }
  style muted = { paint: fg(muted-foreground), text: fontSize(sm) }
  style error = { paint: fg(destructive), text: fontSize(sm) }
  style state = { paint: fg(muted-foreground), text: fontSize(sm).fontFamily(mono) }
</scope>

<column gap="3" style(page)>
  <h1 style(title)>Live Feed</h1>
  <match {t()}>
    {loading => <p style(muted)>Loading…</p>}
    {error as err => <p style(error)>Error: {err.message}</p>}
    {else => <p style(state)>state: {t().kind}</p>}
  </match>
</column>
