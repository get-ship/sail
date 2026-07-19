// Snippets for /sail/guide/building-a-component. Each one is a STEP of the
// same file — apps/sail-site/src/bill-split/bill-split.sail — so the guide's
// last snippet and the component running at the bottom of the page are the
// same source. Every SAIL sample here is compiled by the build
// (crates/sail/compiler/tests/site_snippets_compile.rs).
export const code = {
  // Step 1 — markup only. No props, no controller: a .sail file with nothing
  // but a template is a valid component.
  markup: `<card sx="p-4 w-full max-w-panel-max-w-sm">
  <column gap="3">
    <row align="center" justify="between" nowrap>
      <strong sx="uppercase tracking-wide text-sm">Dinner</strong>
      <span sx="font-mono text-sm text-muted-foreground">$96.00</span>
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

<card sx="p-4 w-full max-w-panel-max-w-sm">
  <column gap="3">
    <row align="center" justify="between" nowrap>
      <strong sx="uppercase tracking-wide text-sm">{label()}</strong>
      <span sx="font-mono text-sm text-muted-foreground">{total()}</span>
    </row>
  </column>
</card>`,

  usageProps: `<bill-split label="Tapas" total="120"></bill-split>`,

  // Step 3 — state and derived values arrive in the controller.
  controller: `<controller>
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
</controller>`,

  // Step 3, template half — the row that spends what the controller returned.
  counterRow: `<row align="center" justify="between" nowrap gap="3">
  <row align="center" gap="2" nowrap>
    <button size="sm" variant="outline" on:click={() => remove()} aria-label="One fewer person">−</button>
    <span sx="font-mono text-sm tabular-nums w-8 text-center">{people()}</span>
    <button size="sm" variant="outline" on:click={() => add()} aria-label="One more person">+</button>
  </row>
  <span sx="font-mono text-2xl tabular-nums">{share()}</span>
</row>`,

  // Step 5 — a slot: the hole the caller fills.
  slotDef: `<column gap="3">
  ...the header and the counter row...
  <slot name="note"></slot>
</column>`,

  // Step 6 — using the finished component from a page.
  usage: `<bill-split label="Dinner" total="96">
  <p slot="note" sx="text-xs text-muted-foreground">
    Tip included. Arguments not.
  </p>
</bill-split>`,
}
