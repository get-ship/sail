<!-- Live query proof: api.demo.clock tick via match loading/error/ok. -->

<controller>
import { controller, query } from '@ship/sail-runtime'
import { api } from '../../src/compositions/demo-clients'

export default controller(() => ({
  t: query(() => api.demo.clock()),
}))
</controller>

<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style title = { fg(foreground).type({ size: 2xl, weight: bold }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
  style error = { fg(destructive).type({ size: sm }) }
  style mono = { fg(foreground).type({ family: mono }) }
</scope>

<column gap="3" style(page)>
  <h1 style(title)>Live Clock</h1>
  <match {t()}>
    {loading => <p style(muted)>Loading…</p>}
    {error as err => <p style(error)>Error: {err.message}</p>}
    {ok as value =>
      <p style(muted)>
        Tick: <strong style(mono)>{value.now}</strong>
      </p>
    }
  </match>
</column>
