<!--
  <bill-split> — the component the "Building a component" guide grows, one
  step at a time: markup, then props, then state, then styling, then a slot.
  Split a bill between people; the per-person share follows the headcount.
  The guide's snippets are slices of THIS file.
-->
<scope>
  props: {
    label = 'Dinner'
    total = 96
  }
</scope>

<controller>
import { controller, signal, derived } from '@ship/sail-runtime'

const money = (n: number): string => '$' + n.toFixed(2)

export default controller((ctx) => {
  const people = signal(2)
  const add = (): void => people.set((n) => Math.min(n + 1, 12))
  const remove = (): void => people.set((n) => Math.max(n - 1, 1))

  const share = derived(() => money(ctx.total() / people()))
  const totalLabel = derived(() => money(ctx.total()))

  return { people, add, remove, share, totalLabel }
})
</controller>

<card sx="p-4 w-full max-w-panel-max-w-sm">
  <column gap="3">
    <row align="center" justify="between" nowrap>
      <strong sx="uppercase tracking-wide text-sm">{label()}</strong>
      <span sx="font-mono text-sm text-muted-foreground">{totalLabel()}</span>
    </row>

    <row align="center" justify="between" nowrap gap="3">
      <row align="center" gap="2" nowrap>
        <button size="sm" variant="outline" on:click={() => remove()} aria-label="One fewer person">−</button>
        <span sx="font-mono text-sm tabular-nums w-8 text-center">{people()}</span>
        <button size="sm" variant="outline" on:click={() => add()} aria-label="One more person">+</button>
      </row>
      <span sx="font-mono text-2xl tabular-nums">{share()}</span>
    </row>

    <slot name="note"></slot>
  </column>
</card>
