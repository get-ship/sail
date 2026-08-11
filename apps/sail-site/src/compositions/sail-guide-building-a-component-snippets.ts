// Snippets for /docs/guide/building-a-component. Each one is a STEP of the
// same file — apps/sail-site/src/bill-split/bill-split.sail — so the guide's
// last snippet and the component running at the bottom of the page are the
// same source. Every SAIL sample here is compiled by the build
// (crates/sail/compiler/tests/site_snippets_compile.rs).
import { signal, derived } from '../vendor/runtime'
export const code = {
  // Step 1 — markup only. No props, no controller: a .sail file with nothing
  // but a template is a valid component.
  markup: `<card style({ box: pad(4).w(full).maxW(panel-max-w-sm) })>
  <column gap="3">
    <row align="center" justify="between" nowrap>
      <strong style({ text: textCase(uppercase).tracking(wide).fontSize(sm) })>
        Dinner</strong>
      <span style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(sm) })>
        $96.00</span>
    </row>
  </column>
</card>`,

  // Step 2 — the two hard-coded strings become props.
  scope: `<scope>
  props: {
    label = 'Dinner'
    total = 96
  }
</scope>

<card style({ box: pad(4).w(full).maxW(panel-max-w-sm) })>
  <column gap="3">
    <row align="center" justify="between" nowrap>
      <strong style({ text: textCase(uppercase).tracking(wide).fontSize(sm) })>
        {label()}</strong>
      <span style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(sm) })>
        {total()}</span>
    </row>
  </column>
</card>`,

  usageProps: `<bill-split label="Tapas" total="120"></bill-split>`,

  // Step 3 — state and derived values arrive in the controller.
  // controller/signal/derived are ambient — no import.
  controller: `<controller>
const money = (n: number): string => '$' + n.toFixed(2)

export default controller((ctx) => {
  const people = signal(2)
  const add = (): void => people.set((n) => Math.min(n + 1, 12))
  const remove = (): void => people.set((n) => Math.max(n - 1, 1))

  const share = derived(() => money(ctx.total() / people()))
  const totalLabel = derived(() => money(ctx.total()))

  return { people, add, remove, share, totalLabel }
})
</controller>`,

  // Step 3, template half — the row that spends what the controller returned.
  counterRow: `<row align="center" justify="between" nowrap gap="3">
  <row align="center" gap="2" nowrap>
    <button size="sm" variant="outline" on:click={() => remove()} label="One fewer person">−</button>
    <span style({ box: w(8), text: fontFamily(mono).fontSize(sm).nums(tabular).textAlign(center) })>
      {people()}</span>
    <button size="sm" variant="outline" on:click={() => add()} label="One more person">+</button>
  </row>
  <span style({ text: fontFamily(mono).fontSize(2xl).nums(tabular) })>
    {share()}</span>
</row>`,

  // Step 5 — a slot: the hole the caller fills.
  slotDef: `<column gap="3">
  ...the header and the counter row...
  <slot name="note"></slot>
</column>`,

  // Step 6 — using the finished component from a page.
  usage: `<bill-split label="Dinner" total="96">
  <p slot="note" style({ paint: fg(muted-foreground), text: fontSize(xs) })>
    Tip included. Arguments not.
  </p>
</bill-split>`,
}
