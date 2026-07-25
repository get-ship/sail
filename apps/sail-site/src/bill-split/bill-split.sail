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

  const card = style.pad(4).w(full).maxW(panel-max-w-sm)
  const title = style.textCase(uppercase).tracking(wide).fontSize(sm)
  const totalText = style.fontFamily(mono).fontSize(sm).fg(muted-foreground)
  const peopleCount = style.fontFamily(mono).fontSize(sm).nums(tabular).w(8).textAlign(center)
  const shareText = style.fontFamily(mono).fontSize(2xl).nums(tabular)
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

<card sx={card}>
  <column gap="3">
    <row align="center" justify="between" nowrap>
      <strong sx={title}>{label()}</strong>
      <span sx={totalText}>{totalLabel()}</span>
    </row>

    <row align="center" justify="between" nowrap gap="3">
      <row align="center" gap="2" nowrap>
        <button size="sm" variant="outline" on:click={() => remove()} aria-label="One fewer person">−</button>
        <span sx={peopleCount}>{people()}</span>
        <button size="sm" variant="outline" on:click={() => add()} aria-label="One more person">+</button>
      </row>
      <span sx={shareText}>{share()}</span>
    </row>

    <slot name="note"></slot>
  </column>
</card>
