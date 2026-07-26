// Code snippets for the Data page. Kept OUT of the controller so the
// compiler's scope scan isn't fooled by patterns inside these strings.
// The query() controller + the while-loading/while-error/if-ok template are
// compile-checked against the in-repo sail-compiler.
export const code = {
  demo: `<controller>
import { query } from '@ship/sail-runtime'
import { api } from './api'
export default controller(() => {
  const attempt = signal(1)
  // A signal read in the call args is a tracked dep: bump it and query()
  // re-keys, re-fetching through 'loading'.
  const users = query(() => api.users.list({ attempt: attempt() }))
  return { users, reload: () => attempt.set((a) => a + 1) }
})
</controller>

<button on:click={reload}>Reload</button>

<while-loading of="users">
  <spinner></spinner>
</while-loading>
<while-error as="err" of="users">
  <alert tone="destructive">{err.message}</alert>
</while-error>
<if cond={users().kind === 'ok'}>
  <for as="name" of={users().value} key={name}>
    <badge>{name}</badge>
  </for>
</if>`,

  source: `// the <controller> block body (inline in users.sail)
import { query } from '@ship/sail-runtime'
import { api } from './api' // an imported client — query() derives the key from it

export default controller(() => {
  // query() turns a client call into a reactive state signal:
  //   { kind: 'loading' } → { kind: 'ok', value } | { kind: 'error', error }
  // A method that returns Promise<T> fetches once (pull); one that returns
  // LiveSource<T> subscribes (push) — same three states either way.
  const users = query(() => api.users.list())
  return { users }
})`,

  states: `<!-- users.sail — the three states are explicit, never forgotten -->
<while-loading of="users">
  <p role="status">Loading…</p>
</while-loading>

<while-error as="err" of="users">
  <p role="alert">{err.message}</p>
</while-error>

<if cond={users().kind === 'ok'}>
  <ul><for as="u" of={users().value} key={u.id}><li>{u.name}</li></for></ul>
</if>`,
}
