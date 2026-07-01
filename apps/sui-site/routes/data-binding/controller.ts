import { controller } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const code = {
  seam: [
    '<!-- The declarative seam — same shape for every source type. -->',
    '<!-- "live" turns the result into a LiveSource that re-renders on change. -->',
    '<!-- Loading + error states are owned by the runtime. -->',
    '<data name="users" source="ship.users.list()" live>',
    '  <!-- the result binds to the host name; render UI below -->',
    '</data>',
  ].join('\n'),
  match: [
    '<!-- The runtime keeps the seam envelope in a signal; you match on it. -->',
    'const users = data("ship.users.list()", { live: true })',
    '',
    '<template>',
    '  {#match users()}',
    "    {:case 'loading'} <sui-spinner></sui-spinner>",
    "    {:case 'error'}",
    '      <sui-alert tone="destructive">Could not load users.</sui-alert>',
    "    {:case 'ok'}",
    '      <ul>{#for u of users().value key={u.id}<li>{u.name}</li>{/for}</ul>',
    '  {/match}',
    '</template>',
  ].join('\n'),
  mutate: [
    '<!-- ship.users.create(...) returns the new envelope; the SDK -->',
    '<!-- pushes it back through the LiveSource so the list re-renders. -->',
    'async function add() {',
    '  await ship.users.create({ name: draft() })',
    "  draft.set('')",
    '}',
  ].join('\n'),
}

export default controller({ scope: () => ({ code }) })
