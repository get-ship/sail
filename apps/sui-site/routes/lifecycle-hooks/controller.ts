import { controller } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const code = {
  mount: [
    '// Mount / cleanup: the lifecycle pair.',
    'onMount(() => {',
    '  const id = setInterval(() => tick(), 1000)',
    '  // returning a function registers a cleanup automatically',
    '  return () => clearInterval(id)',
    '})',
  ].join('\n'),
  template: [
    '<!-- template-side bindings live as JSX-like attributes on tags -->',
    '<!-- bind:value=value is shorthand for value=val on:input=… -->',
    '<!-- on:click=fn attaches a native click listener -->',
    '<!-- bind:this=elem exposes the element to mount code into -->',
  ].join('\n'),
  controller: [
    '// scope() effect with cleanup — registered automatically',
    '// when the effect callback returns a function.',
    'const t = signal(0)',
    'effect(() => {',
    '  const id = setInterval(() => t.set(t() + 1), 1000)',
    '  return () => clearInterval(id)',
    '})',
  ].join('\n'),
}

export default controller({ scope: () => ({ code }) })
