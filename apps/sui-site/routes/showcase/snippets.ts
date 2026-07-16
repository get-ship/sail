// Code snippets for the showcase. Each is the screen in its SINGLE-FILE form
// (controller block + markup) and compiles as written — checked by
// crates/sui/compiler/tests/site_snippets_compile.rs.
export const code = {
  team: `<controller>
import { controller, signal, filteredList, selection } from '@ship/sui-runtime'

export default controller({
  setup: () => {
    const people = signal([
      { id: 1, name: 'Ada Lovelace', team: 'Engines', status: 'active' },
      { id: 2, name: 'Grace Hopper', team: 'Compilers', status: 'active' },
    ])
    const { query, filtered } = filteredList(() => people())
    const { selected, toggle, clear } = selection(() => people(), (p) => p.id)
    const remove = () => {
      people.set((l) => l.filter((p) => p.id !== selected()?.id))
      clear()
    }
    return { query, filtered, selected, toggle, remove }
  },
})
</controller>

<input placeholder="Filter people" bind:value={query} />

{#for p of filtered() key={p.id}}
  <row gap="3" align="center"
       on:click={() => toggle(p.id)}
       aria-current={selected()?.id === p.id ? 'true' : null}
       sx="p-2 rounded-md cursor-pointer hover:bg-muted current:bg-muted current:ring-primary">
    <avatar></avatar>
    <strong sx="grow-1">{p.name}</strong>
    <badge tone={p.status === 'active' ? 'success' : 'muted'}>{p.status}</badge>
  </row>
{/for}

{#if selected()}
  <card motion:enter={{ opacity: 'fade', translate-y: 'spring:snappy' }} sx="p-4">
    <strong>{selected().name}</strong>
    <row gap="2">
      <button on:click={remove} tone="destructive" variant="outline">Remove</button>
    </row>
  </card>
{/if}`,

  fleet: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  setup: () => {
    const services = signal([
      { id: 'api', name: 'api-gateway', status: 'live', p99: 41, load: 82 },
    ])
    const deploy = (id) => {
      services.set((l) => l.map((s) => (s.id === id ? { ...s, status: 'rolling' } : s)))
      setTimeout(() => {
        services.set((l) => l.map((s) => (s.id === id ? { ...s, status: 'live' } : s)))
      }, 1500)
    }
    return { services, deploy }
  },
})
</controller>

{#for s of services() key={s.id}}
  <card sx="p-4">
    <row gap="2" align="center">
      <badge tone={s.status === 'live' ? 'success' : 'warning'}>{s.status}</badge>
      <strong sx="grow-1">{s.name}</strong>
      <button on:click={() => deploy(s.id)} disabled={s.status === 'rolling'}>
        {s.status === 'rolling' ? 'Rolling…' : 'Deploy'}
      </button>
    </row>
    <progress value={s.load} max="100"></progress>
    <p sx="text-sm text-muted-foreground">p99 {s.p99} ms</p>
  </card>
{/for}`,

  player: `<controller>
import { controller, signal, effect } from '@ship/sui-runtime'

export default controller({
  setup: () => {
    const playing = signal(true)
    const position = signal(23)
    // The interval lives in an effect: it tracks playing(), so pausing
    // re-runs it, and its cleanup stops the timer on pause and unmount.
    effect(() => {
      if (!playing()) return
      const t = setInterval(() => position.set((p) => (p + 1) % 100), 150)
      return () => clearInterval(t)
    })
    return { playing, position, toggle: () => playing.set((p) => !p) }
  },
})
</controller>

<card sx="p-4">
  <row gap="4" align="center">
    <column gap="1" sx="grow-1">
      <strong>Signals in the Rain</strong>
      <progress value={position()} max="100"></progress>
    </column>
    {#if playing()}
      <row gap="1" align="center">
        <div sx="w-1 h-5 rounded-full bg-primary" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.1s' }}></div>
        <div sx="w-1 h-5 rounded-full bg-primary" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.1s' }}></div>
        <div sx="w-1 h-5 rounded-full bg-primary" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.1s' }}></div>
      </row>
    {/if}
    <button on:click={toggle} variant="solid" tone="primary" motion:press={{ scale: 'press' }}>
      {playing() ? 'Pause' : 'Play'}
    </button>
  </row>
</card>`,
}
