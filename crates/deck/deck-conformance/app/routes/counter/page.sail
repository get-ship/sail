<!-- Client signal counter; initial value is JS-only (empty SSR slot until hydrate). -->

<controller>
import { controller, signal } from '@ship/sail-runtime'

export default controller(() => ({ count: signal(0) }))
</controller>

<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style title = { fg(foreground).type({ size: 2xl, weight: bold }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
  style mono = { fg(foreground).type({ family: mono }) }
</scope>

<column gap="3" style(page)>
  <h1 style(title)>Counter</h1>
  <p style(muted)>
    Count: <strong style(mono)>{count()}</strong>
  </p>
  <button on:click={() => count.set(count() + 1)}>increment</button>
</column>
