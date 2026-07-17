// Code snippets for the Live data page. Kept OUT of the controller so the
// compiler's scope scan isn't fooled by patterns inside these strings.
// The live() controller + the while-loading/while-error/if-ok template are
// compile-checked against the in-repo sui-compiler; data() matches the real
// @ship/sui-runtime DataOpts signature.
export const code = {
  demo: `<controller>
import { controller, live, signal } from '@ship/sui-runtime'
export default controller(() => {
  const attempt = signal(1)
  const users = live(() => {
    attempt() // tracked: bump it and the source re-runs
    return fetchUsers()
  })
  return { users, reload: () => attempt.set((a) => a + 1) }
})
</controller>

<button on:click={reload}>Reload</button>

{#while-loading of users}
  <spinner></spinner>
{/while-loading}
{#while-error err of users}
  <alert tone="destructive">{err.message}</alert>
{/while-error}
{#if users().kind === 'ok'}
  {#for name of users().value key={name}}
    <badge>{name}</badge>
  {/for}
{/if}`,

  source: `// the <controller> block body (inline in users.sui)
import { controller, live } from '@ship/sui-runtime'

export default controller(() => {
  // live() wraps any async source in a reactive state signal:
  //   { kind: 'loading' } → { kind: 'ok', value } | { kind: 'error', error }
  const users = live(() => fetch('/api/users').then((r) => r.json()))
  return { users }
})`,

  states: `<!-- users.sui — the three states are explicit, never forgotten -->
{#while-loading of users}
  <p role="status">Loading…</p>
{/while-loading}

{#while-error err of users}
  <p role="alert">{err.message}</p>
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
