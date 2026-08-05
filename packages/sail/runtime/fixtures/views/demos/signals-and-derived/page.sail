<controller>
import { controller, signal, derived } from '@ship/sail-runtime'

export default controller(() => {
    const count = signal(0)
    const history = signal<Array<{ k: number; value: number }>>([])
    const double = derived(() => count() * 2)
    let nextKey = 1

    const push = (next: number) => {
      count.set(next)
      history.set([{ k: nextKey++, value: next }, ...history()].slice(0, 10))
    }

    return {
      count,
      double,
      history,
      inc: () => push(count() + 1),
      dec: () => push(count() - 1),
      reset: () => {
        count.set(0)
        history.set([])
      },
    }
  })
</controller>

<div data-demo="true">
  <p><a href="/demos">← demos</a></p>
  <h1>signals + derived</h1>
  <p>
    <code>scope:</code> declares signals; <code>derived:</code> computes from
    them. Read with <code>count()</code>; write with <code>count.set(…)</code>.
  </p>

  <section>
    <span>count = <strong data-test="count">{count()}</strong></span>
    <span>double = <strong data-test="double">{double()}</strong></span>
  </section>

  <div>
    <button data-test="dec" on:click={dec}>−1</button>
    <button data-test="inc" on:click={inc}>+1</button>
    <button data-test="reset" on:click={reset}>reset</button>
  </div>

  <section>
    <h2>history</h2>
    <if {history().length === 0}>
      <p data-muted="true" data-test="empty">No history yet.</p>
    <else>
      <ol data-test="history">
        <for {history()} as="v" key={v.k}>
          <li>{v.value}</li>
        </for>
      </ol>
    </else></if>
  </section>
</div>
