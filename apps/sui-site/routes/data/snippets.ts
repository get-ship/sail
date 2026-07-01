// Code snippets for the Live data page. Kept OUT of the controller so the
// compiler's scope scan isn't fooled by patterns inside these strings.
// The live() controller + the while-loading/while-error/if-ok template are
// compile-checked against the in-repo sui-compiler; data() matches the real
// @ship/sui-runtime DataOpts signature.
export const code = {
  source: `// users.controller.ts
import { controller, live } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    // live() wraps any async source in a reactive state signal:
    //   { kind: 'loading' } → { kind: 'ok', value } | { kind: 'error', error }
    const users = live(() => fetch('/api/users').then((r) => r.json()))
    return { users }
  },
})`,

  states: `<!-- users.sui — the three states are explicit, never forgotten -->
{#while-loading of users}
  <p class="muted">Loading…</p>
{/while-loading}

{#while-error err of users}
  <p class="err">{err.message}</p>
{/while-error}

{#if users().kind === 'ok'}
  <ul>{#for u of users().value key={u.id}}<li>{u.name}</li>{/for}</ul>
{/if}`,

  server: `// A live SERVER signal: SSR'd initial value + SSE patches.
import { data } from '@ship/sui-runtime'

const feed = data({
  initial: [],                   // server-rendered at request time
  subscribe: '/api/feed/stream', // SSE endpoint; patches merge in live
})
// feed.signal — read it like any signal
// feed.dispose() — close the stream`,
}
