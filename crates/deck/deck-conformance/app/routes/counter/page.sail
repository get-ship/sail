<!-- Client signal counter; initial value is JS-only (empty SSR slot until hydrate). -->

<controller>
import { controller, signal } from '@ship/sail-runtime'

export default controller(() => ({ count: signal(0) }))
</controller>

<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style title = { paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) }
  style muted = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mono = { paint: fg(foreground), text: fontFamily(mono) }
</scope>

<column gap="3" style(page)>
  <h1 style(title)>Counter</h1>
  <p style(muted)>
    Count: <strong style(mono)>{count()}</strong>
  </p>
  <button on:click={() => count.set(count() + 1)}>increment</button>
</column>
