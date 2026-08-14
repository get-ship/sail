<!-- Error-boundary proof: query that always fails via ship.demo.boom. -->

<controller>
import { controller, query } from '@ship/sail-runtime'
import { ship } from '../../src/compositions/demo-clients'

export default controller(() => ({
  result: query(() => ship.demo.boom()),
}))
</controller>

<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style title = { fg(foreground).type({ size: 2xl, weight: bold }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
  style mono = { fg(foreground).type({ family: mono }) }
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
