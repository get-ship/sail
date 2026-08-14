<controller>
import { controller, signal, derived, batch } from '@ship/sail-runtime'

export default controller(() => {
  const count = signal(0)
  const history = signal<Array<{ k: number; value: number }>>([])
  const double = derived(() => count() * 2)
  let nextKey = 1

  const push = (next: number) => {
    batch(() => {
      count.set(next)
      history.set([{ k: nextKey++, value: next }, ...history()].slice(0, 10))
    })
  }

  return {
    count,
    double,
    history,
    inc: () => push(count() + 1),
    dec: () => push(count() - 1),
    reset: () => {
      batch(() => {
        count.set(0)
        history.set([])
      })
    },
  }
})
</controller>

<scope>
  style page = { pad(6).margin({ x: auto }).size({ max-w: breakpoint-tablet }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
  style mono = { type({ family: mono, size: sm }) }
</scope>

<column gap="4" style(page)>
  <p><a href="/demos">← demos</a></p>
  <column gap="1">
    <h1>signals + derived</h1>
    <p style(muted)>
      signal holds writeable state; derived recomputes from it.
      Read with count(); write with count.set(...).
    </p>
  </column>

  <row gap="4" align="center">
    <span style(mono)>count = <strong data-test="count">{count()}</strong></span>
    <span style(mono)>double = <strong data-test="double">{double()}</strong></span>
  </row>

  <row gap="2">
    <button data-test="dec" variant="outline" on:click={dec}>-1</button>
    <button data-test="inc" variant="solid" tone="primary" on:click={inc}>+1</button>
    <button data-test="reset" variant="ghost" on:click={reset}>reset</button>
  </row>

  <column gap="2">
    <h2>history</h2>
    <if {history().length === 0}>
      <p data-test="empty" style(muted)>No history yet.</p>
    {else =>
      <ol data-test="history" style(mono)>
        <for {history() as v} key={v.k}>
          <li>{v.value}</li>
        </for>
      </ol>
    }</if>
  </column>
</column>
