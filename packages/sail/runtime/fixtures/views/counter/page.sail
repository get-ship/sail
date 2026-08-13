<controller>
import { controller, signal } from '@ship/sail-runtime'

export default controller(() => ({ count: signal(0) }))
</controller>

<scope>
  style page = { box: space({ pad: 8, mx: auto }).size({ max-w: breakpoint-mobile }), text: type({ align: center }) }
  style value = { text: type({ family: mono, size: 2xl }) }
  style caption = { box: space({ mt: 6 }), paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
</scope>

<column gap="4" style(page)>
  <h1>Counter</h1>
  <p>Count: <span data-test="value" style(value)>{count()}</span></p>
  <row gap="2" justify="center">
    <button data-test="dec" variant="outline" on:click={() => count.set(count() - 1)}>-</button>
    <button data-test="inc" variant="solid" tone="primary" on:click={() => count.set(count() + 1)}>+</button>
    <button data-test="reset" variant="ghost" on:click={() => count.set(0)}>reset</button>
  </row>
  <p style(caption)>Tier 1: signal state, declarative template, the simplest possible Sail view.</p>
</column>
