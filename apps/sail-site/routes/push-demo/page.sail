<controller>
// /push-demo — browser-probe fixture for query()'s PUSH path. `feed` is an
// ES-imported client; `feed.clock.stream()` returns a LiveSource<Tick>, so
// query() subscribes (push) instead of fetching once (pull) — the SAME
// query() call, routed by the method's return-value shape. Each interval
// tick re-renders the tick count below with zero console errors.

import { controller, query } from '@ship/sail-runtime'
import { feed } from './push-clock'

export default controller(() => {
  const tick = query(() => feed.clock.stream())
  return { tick }
})
</controller>

<column gap="3" sx="p-6 max-w-breakpoint-tablet mx-auto">
  <h1 sx="text-2xl font-bold text-foreground">push() demo</h1>
  <p sx="text-muted-foreground text-sm">
    A client method returning <code>LiveSource&lt;T&gt;</code> makes
    <code>query()</code> subscribe — one <code>ok</code> per push.
  </p>

  {#if tick().kind === 'loading'}
  <p sx="text-muted-foreground text-sm" data-testid="push-loading">waiting for first push…</p>
  {/if}
  {#if tick().kind === 'error'}
  <p sx="text-destructive text-sm" data-testid="push-error">Error</p>
  {/if}
  {#if tick().kind === 'ok'}
  <p sx="text-foreground font-mono" data-testid="push-tick">tick #{tick().value.n}</p>
  <p sx="text-muted-foreground text-xs font-mono" data-testid="push-at">{tick().value.at}</p>
  {/if}
</column>
