<!--
  <store-total> — a read-only consumer of the shared `docs.tally`
  store (see ./tally.ts). It never writes; it simply proves that a
  separate element resolves the SAME store instance the counters
  write to.
-->
<controller>
import { controller } from '@ship/sail-runtime'
import { useStore } from '@ship/sail-runtime'
import { tally } from './tally'

export default controller(() => {
  const { count } = useStore(tally)
  return { count }
})
</controller>

<card tone="primary" variant="outline" sx="p-4 w-full">
  <column gap="2" align="center">
    <span sx="text-xs uppercase tracking-wide text-muted-foreground">Total (reads the same store)</span>
    <output data-tally-total sx="text-2xl font-semibold tabular-nums">{count()}</output>
  </column>
</card>
