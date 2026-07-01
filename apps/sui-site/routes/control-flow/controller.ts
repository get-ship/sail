import { controller } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

// Each sample is compile-checked against the in-repo sui-compiler (paired with
// a controller declaring isOpen/items/status). Note the {#for} tag closes with
// }} — a dropped brace is the classic control-flow mistake.
const code = {
  if: [
    '<!-- {#if} splices one branch in place — no wrapper element. -->',
    '{#if isOpen()}',
    '  <section class="panel"><h3>Details</h3></section>',
    '{:else}',
    '  <button on:click={() => isOpen.set(true)}>Show details</button>',
    '{/if}',
  ].join('\n'),
  for: [
    '<!-- items() is a signal of an array. key= is required for stable reorders. -->',
    '<ul>',
    '  {#for item of items() key={item.id}}',
    '    <li>{item.label}</li>',
    '  {/for}',
    '</ul>',
  ].join('\n'),
  match: [
    '<!-- match a discriminant; {:default} catches the rest. -->',
    '{#match status()}',
    "  {:case 'loading'}<p class=\"muted\">Loading…</p>",
    "  {:case 'ready'}<p>Done</p>",
    '  {:default}<p class="muted">Idle</p>',
    '{/match}',
  ].join('\n'),
}

export default controller({ scope: () => ({ code }) })
