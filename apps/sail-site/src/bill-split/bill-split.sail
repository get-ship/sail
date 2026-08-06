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

  const card = style({ box: pad(4).w(full).maxW(panel-max-w-sm) })
  const title = style({ text: textCase(uppercase).tracking(wide).fontSize(sm) })
  const totalText = style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(sm) })
  const peopleCount = style({ box: w(8), text: fontFamily(mono).fontSize(sm).nums(tabular).textAlign(center) })
  const shareText = style({ text: fontFamily(mono).fontSize(2xl).nums(tabular) })
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

<card>
  style(card)

  <column gap="3">
    <row align="center" justify="between" nowrap>
      <strong>
        style(title)
{label()}</strong>
      <span>
        style(totalText)
{totalLabel()}</span>
    </row>

    <row align="center" justify="between" nowrap gap="3">
      <row align="center" gap="2" nowrap>
        <button size="sm" variant="outline" on:click={() => remove()} label="One fewer person">−</button>
        <span>
          style(peopleCount)
{people()}</span>
        <button size="sm" variant="outline" on:click={() => add()} label="One more person">+</button>
      </row>
      <span>
        style(shareText)
{share()}</span>
    </row>

    <slot name="note"></slot>
  </column>
</card>
