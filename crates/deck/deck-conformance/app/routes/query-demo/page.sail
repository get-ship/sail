<controller>
import { controller, query, mutation, signal } from '@ship/sail-runtime'
import { crm } from '../../clients/crm-demo'

// query-demo — a second, independent compile-time proof of the
// query()/mutation() derived-identity model (query-rename-slice-spec.md),
// alongside apps/sail-site/routes/query-demo (the one actually used for
// the browser probe — see query-slice-report.md). `crm` is a REAL
// ES-imported client (app/clients/crm-demo.ts); the compiler statically
// derives the cache key from `crm.deals.list`'s member path + the
// `stage()` arg (crates/sail/compiler/src/check/query_keys.rs) — no
// handwritten key anywhere in this file.
//
// NOT browser-tested here: this app's `/_deck/view/{name}` page-serving
// route was extracted to the separate `ship-views` repo (see
// deck-conformance's lib.rs module doc) — a pre-existing, unrelated gap
// this job found but didn't introduce. `sail-compiler build` compiling
// this file cleanly (verified) is still real evidence the derivation
// pass handles a genuine Rust-backed one-shot endpoint, not just the
// in-memory sail-site fixture.
export default controller(() => {
  const stage = signal<'open' | 'closed'>('open')

  // The thunk accepts the cache's AbortSignal and forwards it to the SDK
  // call so dispose/invalidate abort the real `fetch()`, not just discard
  // its result — see native-cache.ts's AbortSignal wiring. The compiler's
  // key derivation reads the call expression regardless of the thunk's
  // parameter list.
  const deals = query((abortSignal) => crm.deals.list({ stage: stage() }, abortSignal))

  const close = mutation((abortSignal) => crm.deals.update({ id: 'd1', stage: 'closed' }, abortSignal))

  return { stage, deals, close }
})
</controller>

<column gap="3" sx="p-6 max-w-breakpoint-tablet mx-auto">
  <h1 sx="text-2xl font-bold text-foreground">query() demo</h1>
  <p sx="text-muted-foreground text-sm">A real query()/mutation() round-trip against crm-demo.ts.</p>

  <row gap="2">
    <button on:click={() => stage.set('open')} aria-pressed={stage() === 'open'}>open</button>
    <button on:click={() => stage.set('closed')} aria-pressed={stage() === 'closed'}>closed</button>
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
    <li>{deal.id} — {deal.stage} — {deal.amount}</li>
    {/for}
  </column>
  {/if}

  <button on:click={() => close.mutate()} data-testid="close-deal">close d1</button>
  <p data-testid="close-state">{close.state().kind}</p>
</column>
