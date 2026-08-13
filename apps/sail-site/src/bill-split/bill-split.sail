<!--
  <bill-split> — split a bill between people; per-person share tracks headcount.
  Used by the "Building a component" guide as the growing example.
-->
<scope>
  props: {
    label = 'Dinner'
    total = 96
  }

  style card = { box: space({ pad: 4 }).size({ w: full, max-w: panel-max-w-sm }) }
  style title = { text: type({ size: sm, tracking: wide, case: uppercase }) }
  style totalText = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: sm }) }
  style peopleCount = { box: size({ w: 8 }), text: type({family: mono, size: sm, align: center, nums: tabular}) }
  style shareText = { text: type({family: mono, size: 2xl, nums: tabular}) }
</scope>

<controller>
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

<card style(card)>
  <column gap="3">
    <row align="center" justify="between" nowrap>
      <strong style(title)>{label()}</strong>
      <span style(totalText)>{totalLabel()}</span>
    </row>

    <row align="center" justify="between" nowrap gap="3">
      <row align="center" gap="2" nowrap>
        <button size="sm" variant="outline" on:click={() => remove()} label="One fewer person">−</button>
        <span style(peopleCount)>{people()}</span>
        <button size="sm" variant="outline" on:click={() => add()} label="One more person">+</button>
      </row>
      <span style(shareText)>{share()}</span>
    </row>

    <slot name="note"></slot>
  </column>
</card>
