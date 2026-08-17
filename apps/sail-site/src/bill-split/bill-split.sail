<!--
  <bill-split> — split a bill between people; per-person share tracks headcount.
  Used by the "Building a component" guide as the growing example.
-->
<scope>
  props: {
    label = 'Dinner'
    total = 96
  }

  style card = { size(measure-sm) }
  style title = { type(label) }
  style totalText = { fg(muted).type(mono) }
  style peopleCount = { type(mono-sm) }
  style shareText = { type(mono) }
</scope>

<controller>
const money = (n: number): string => '$' + n.toFixed(2)

export default controller((ctx: { total: () => number }) => {
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
