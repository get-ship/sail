<!-- Error-boundary proof: query that always fails via ship.demo.boom. -->

<controller>
import { controller, query } from '@ship/sail-runtime'
import { ship } from '../../src/compositions/demo-clients'

export default controller(() => ({
  result: query(() => ship.demo.boom()),
}))
</controller>

<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style title = { paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) }
  style muted = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mono = { paint: fg(foreground), text: fontFamily(mono) }
</scope>

<column gap="3" style(page)>
  <h1 style(title)>Error Boundary</h1>
  <match {result()}>
    {loading => <p style(muted)>Loading…</p>}
    {error as err => <p style(muted)>Something went wrong</p>}
    {ok as value =>
      <p style(muted)>
        Value: <strong style(mono)>{value.data}</strong>
      </p>
    }
  </match>
</column>
