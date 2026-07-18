<controller>
// /query-demo — browser-probe fixture for query()/mutation()'s derived-
// identity model (query-rename-slice-spec.md). `crm` is a real
// ES-imported client (./crm-demo.ts); the compiler statically derives
// the cache key from `crm.deals.list`'s member path + the `stage()`
// arg — no handwritten key anywhere in this file. Mirrors /data's
// query() pull demo (visible delay, a reload/toggle affordance) so the two
// are directly comparable in the browser.

import { controller, query, mutation, signal } from '@ship/sail-runtime'
import { crm } from './crm-demo'

export default controller(() => {
  const stage = signal<'open' | 'closed'>('open')

  const deals = query((abortSignal) => crm.deals.list({ stage: stage() }))

  const close = mutation((abortSignal) => crm.deals.update({ id: 'd1', stage: 'closed' }))

  return { stage, deals, close }
})
</controller>

<column gap="3" sx="p-6 max-w-breakpoint-tablet mx-auto">
  <h1 sx="text-2xl font-bold text-foreground">query() demo</h1>
  <p sx="text-muted-foreground text-sm" data-testid="deals-stage">stage: {stage()}</p>

  <row gap="2">
    <button on:click={() => stage.set('open')} data-testid="stage-open">open</button>
    <button on:click={() => stage.set('closed')} data-testid="stage-closed">closed</button>
  </row>

  {#if deals().kind === 'loading'}
  <p sx="text-muted-foreground text-sm" data-testid="deals-loading">Loading…</p>
  {/if}
  {#if deals().kind === 'error'}
  <p sx="text-destructive text-sm" data-testid="deals-error">Error</p>
  {/if}
  {#if deals().kind === 'ok'}
  <column gap="1" sx="list-none p-0 m-0" data-testid="deals-list">
    {#for deal of deals().value.items key={deal.id}}
    <li data-testid="deal-row">{deal.id} — {deal.stage} — {deal.amount}</li>
    {/for}
  </column>
  {/if}

  <button on:click={() => close.mutate()} data-testid="close-deal">close d1 (auto-invalidates crm.deals.*)</button>
  <p data-testid="close-state">mutation: {close.state().kind}</p>
</column>
