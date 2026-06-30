import { controller } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const code = {
  if: [
    '<!-- {#if} splices one branch into the surrounding template. -->',
    '{#if isOpen()}',
    '  <section class="panel">',
    '    <h3>Open</h3>',
    '    <p>...content...</p>',
    '  </section>',
    '{:else}',
    '  <button on:click={() => isOpen.set(true)}>Open</button>',
    '{/if}',
  ].join('\n'),
  for: [
    '<!-- items is a signal of an array. key= is required for stable reorders. -->',
    '<ul>',
    '  {#for item of items() key={item.id}',
    '    <li>',
    '      <input',
    '        value={item.text}',
    '        on:input={(e) => item.text = e.target.value}',
    '      />',
    '      <button on:click={() => remove(item.id)}>×</button>',
    '    </li>',
    '  {/for}',
    '</ul>',
  ].join('\n'),
  match: [
    '{#match status()}',
    "  {:case 'loading'}",
    '    <sui-spinner></sui-spinner>',
    "  {:case 'error'}",
    '    <sui-alert tone="destructive">{error().message}</sui-alert>',
    "  {:case 'ok'}",
    '    <ul>{#for x of data() key={x.id}<li>{x.name}</li>{/for}</ul>',
    '  {:default}',
    '    <sui-empty-state>No data yet</sui-empty-state>',
    '{/match}',
  ].join('\n'),
}

export default controller({ scope: () => ({ code }) })
