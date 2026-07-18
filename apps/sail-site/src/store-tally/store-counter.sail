<!--
  <store-counter> — one writer/reader of the shared `docs.tally` store
  (see ./tally.ts). The State chapter mounts two of these next to one
  <store-total>; clicking either one moves all three, which is the
  point of the demo: `useStore` resolves the SAME instance across
  separate elements.
-->
<scope>
  props: {
    label = 'Counter'
  }
</scope>

<controller>
import { controller } from '@ship/sail-runtime'
import { useStore } from '@ship/sail-runtime'
import { tally } from './tally'

export default controller(() => {
  const { count, add } = useStore(tally)
  return { count, add }
})
</controller>

<card sx="p-4 w-full">
  <column gap="2" align="center">
    <span sx="text-xs uppercase tracking-wide text-muted-foreground">{label()}</span>
    <output data-tally-count sx="text-2xl font-semibold tabular-nums">{count()}</output>
    <button variant="soft" tone="primary" size="sm" on:click={() => add(1)}
            motion:press={{ scale: 'press' }}>+1</button>
  </column>
</card>
