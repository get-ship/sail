<!--
  /showcase — three complete programs, the code beside the living UI (design
  3a). A mockup is a component page in disguise; these have state: the team
  list filters as you type and mutates; the fleet walks deploys through their
  states while latency drifts; the player's clock is an effect whose cleanup
  IS the pause button. Each screen wears a different theme on purpose: the
  markup says what the screen is, never what it looks like.
-->

<controller>
// /showcase — screens that WORK, not mockups. The team screen runs the
// filteredList + selection compositions over a mutable list; the fleet
// screen walks a deploy through its states and jitters latency on a
// mount-owned interval so the dashboard reads as live.

import { controller, signal, effect, filteredList, selection, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

interface Person {
  id: number
  name: string
  team: string
  status: 'active' | 'away'
}
interface Service {
  id: string
  name: string
  status: 'live' | 'rolling'
  p99: number
  load: number
}

export default controller((_ctx: ControllerScope<Scope>) => {
    const people = signal<Person[]>([
      { id: 1, name: 'Ada Lovelace', team: 'Engines', status: 'active' },
      { id: 2, name: 'Grace Hopper', team: 'Compilers', status: 'active' },
      { id: 3, name: 'Alan Turing', team: 'Research', status: 'away' },
      { id: 4, name: 'Barbara Liskov', team: 'Compilers', status: 'active' },
      { id: 5, name: 'Edsger Dijkstra', team: 'Research', status: 'away' },
    ])
    const { query, filtered } = filteredList(() => people(), (p, q) =>
      (p.name + ' ' + p.team).toLowerCase().includes(q),
    )
    const { selected, toggle, clear } = selection(() => people(), (p) => p.id)

    const flipStatus = (): void => {
      const cur = selected()
      if (!cur) return
      people.set((list) =>
        list.map((p) =>
          p.id === cur.id ? { ...p, status: p.status === 'active' ? 'away' : 'active' } : p,
        ),
      )
    }
    const remove = (): void => {
      const cur = selected()
      if (!cur) return
      people.set((list) => list.filter((p) => p.id !== cur.id))
      clear()
    }

    const services = signal<Service[]>([
      { id: 'api', name: 'api-gateway', status: 'live', p99: 41, load: 82 },
      { id: 'search', name: 'search-index', status: 'live', p99: 118, load: 47 },
      { id: 'billing', name: 'billing-worker', status: 'live', p99: 22, load: 94 },
    ])
    const deploys = signal<string[]>(['#212 · billing-worker', '#213 · search-index'])
    const deploy = (id: string): void => {
      services.set((l) => l.map((s) => (s.id === id ? { ...s, status: 'rolling' } : s)))
      setTimeout(() => {
        services.set((l) => l.map((s) => (s.id === id ? { ...s, status: 'live' } : s)))
        const name = services().find((s) => s.id === id)?.name ?? id
        deploys.set((d) => [`#${214 + d.length - 2} · ${name}`, ...d].slice(0, 4))
      }, 1500)
    }

    // Ambient life: latency and load drift so the dashboard reads as live.
    // An effect with no tracked reads runs once; its returned cleanup runs on
    // unmount — the idiomatic home for an interval.
    effect(() => {
      const t = setInterval(() => {
        services.set((l) =>
          l.map((s) => ({
            ...s,
            p99: Math.max(8, Math.round(s.p99 + (Math.random() - 0.5) * 14)),
            load: Math.min(100, Math.max(5, Math.round(s.load + (Math.random() - 0.5) * 10))),
          })),
        )
      }, 1200)
      return () => clearInterval(t)
    })

    // The player. Position drifts on an effect interval that TRACKS playing():
    // pause re-runs the effect, whose cleanup stops the timer — the same idiom
    // that stops it on unmount. The equalizer is not simulated: the bars only
    // exist (and loop) while playing() holds them in the DOM.
    const playing = signal(true)
    const position = signal(23)
    effect(() => {
      if (!playing()) return
      const t = setInterval(() => position.set((p) => (p + 1) % 100), 150)
      return () => clearInterval(t)
    })
    const togglePlay = (): void => playing.set((p) => !p)

    return {
      code, people, query, filtered, selected, toggle, flipStatus, remove,
      services, deploys, deploy,
      playing, position, togglePlay,
    }
})
</controller>

<column sx="max-w-breakpoint-tablet mx-auto" gap="5">
  <center>
    <column gap="3" align="center" sx="py-6">
      <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2" motion:enter={{ opacity: 'fade' }}>Showcase</p>
      <h1 motion:enter={{ opacity: 'fade', translate-y: 'spring:gentle' }}>Show, don't tell.</h1>
      <p sx="max-w-panel-max-w-md text-lg text-muted-foreground leading-relaxed" motion:enter={{ opacity: 'fade', translate-y: 'spring:gentle' }}>
        Each example is the complete program on the left, running on the
        right. Type in it, press it, pause it — nothing is a screenshot.
      </p>
    </column>
  </center>

  <!-- ── 01 · TEAM ─────────────────────────────────────────────────────── -->
  <h2><span sx="text-primary font-mono">01</span> Team — filter, select, mutate <small sx="text-muted-foreground">· theme: paper</small></h2>
  <grid cols="2" gap="4">
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">The whole program</p>
      <code-block code={code.team}></code-block>
    </column>
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">Running — type, click rows, remove someone</p>
      <column data-theme-preview="paper" gap="3" sx="p-5 rounded-lg border border-border bg-background text-foreground h-full">
        <input placeholder="Filter people" bind:value={query} />
        <column gap="1">
          {#for p of filtered() key={p.id}}
            <row gap="3" align="center"
                 on:click={() => toggle(p.id)}
                 aria-current={selected()?.id === p.id ? 'true' : null}
                 sx="p-2 rounded-md cursor-pointer hover:bg-muted current:bg-muted current:ring-primary">
              <avatar></avatar>
              <strong sx="grow-1">{p.name}</strong>
              <p sx="text-sm text-muted-foreground">{p.team}</p>
              <badge tone={p.status === 'active' ? 'success' : 'muted'}>{p.status}</badge>
            </row>
          {/for}
        </column>
        {#if selected()}
          <card motion:enter={{ opacity: 'fade', translate-y: 'spring:snappy' }}
                motion:exit={{ opacity: 'fade-out' }} sx="p-4">
            <row gap="3" align="center">
              <column gap="1" sx="grow-1">
                <strong>{selected().name}</strong>
                <p sx="text-sm text-muted-foreground">{selected().team} · {selected().status}</p>
              </column>
              <button on:click={flipStatus} variant="outline">
                {selected().status === 'active' ? 'Set away' : 'Set active'}
              </button>
              <button on:click={remove} tone="destructive" variant="outline">Remove</button>
            </row>
          </card>
        {/if}
      </column>
    </column>
  </row>
  <p>
    <code>filteredList</code> and <code>selection</code> are compositions —
    plain functions over signals, shipped with the runtime. The filter, the
    row highlight (<code>current:</code> on <code>aria-current</code>), the
    detail card animating in and out, the list mutations rippling through the
    derived view: none of it is wiring you wrote.
  </p>

  <!-- ── 02 · FLEET ────────────────────────────────────────────────────── -->
  <h2><span sx="text-primary font-mono">02</span> Fleet — deploys with states <small sx="text-muted-foreground">· theme: console</small></h2>
  <grid cols="2" gap="4">
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">The whole program</p>
      <code-block code={code.fleet}></code-block>
    </column>
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">Running — latency drifts; press Deploy</p>
      <column data-theme-preview="console" gap="3" sx="p-5 rounded-lg border border-border bg-background text-foreground h-full">
        {#for s of services() key={s.id}}
          <card sx="p-4">
            <column gap="2">
              <row gap="2" align="center">
                <badge tone={s.status === 'live' ? 'success' : 'warning'}>{s.status}</badge>
                <strong sx="grow-1">{s.name}</strong>
                <button on:click={() => deploy(s.id)} disabled={s.status === 'rolling'}>
                  {s.status === 'rolling' ? 'Rolling…' : 'Deploy'}
                </button>
              </row>
              <progress value={s.load} max="100"></progress>
              <p sx="text-sm text-muted-foreground">p99 {s.p99} ms · load {s.load}%</p>
            </column>
          </card>
        {/for}
        <separator></separator>
        <column gap="1">
          {#for d of deploys() key={d}}
            <p sx="text-sm text-muted-foreground" motion:enter={{ opacity: 'fade', translate-y: 'spring:snappy' }}>{d}</p>
          {/for}
        </column>
      </column>
    </column>
  </row>
  <p>
    The deploy walks <code>live → rolling → live</code> through two signal
    writes; the button disables itself off the same state, and the finished
    deploy prepends to the log. The drifting latency is one effect with an
    interval — its cleanup runs when you navigate away, which is why the
    idiom is an effect and not a bare <code>setInterval</code>.
  </p>

  <!-- ── 03 · PLAYER ───────────────────────────────────────────────────── -->
  <h2><span sx="text-primary font-mono">03</span> Player — motion is one attribute per gesture <small sx="text-muted-foreground">· theme: glass</small></h2>
  <grid cols="2" gap="4">
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">The whole program</p>
      <code-block code={code.player}></code-block>
    </column>
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">Running — pause it, watch the equalizer stop</p>
      <column data-theme-preview="glass" gap="3" sx="p-5 rounded-lg border border-border bg-background text-foreground h-full">
        <center sx="h-full">
        <column gap="3" sx="w-full">
        <card sx="p-4">
          <row gap="4" align="center">
            <div sx="w-9 h-9 rounded-lg bg-tone-primary-bg border border-primary">
              <center sx="h-full"><strong sx="text-xl text-primary">♪</strong></center>
            </div>
            <column gap="1" sx="grow-1 min-w-0">
              <strong>Signals in the Rain</strong>
              <p sx="text-sm text-muted-foreground">The Reactive Quartet</p>
              <progress value={position()} max="100"></progress>
            </column>
            {#if playing()}
              <row gap="1" align="center" sx="min-h-7">
                <div sx="w-1 h-5 rounded-full bg-primary" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.1s' }}></div>
                <div sx="w-1 h-7 rounded-full bg-primary opacity-75" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.1s' }}></div>
                <div sx="w-1 h-4 rounded-full bg-primary" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.1s' }}></div>
                <div sx="w-1 h-6 rounded-full bg-primary opacity-75" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.1s' }}></div>
                <div sx="w-1 h-5 rounded-full bg-primary" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.1s' }}></div>
              </row>
            {:else}
              <row gap="1" align="center" sx="min-h-7">
                <div sx="w-1 h-2 rounded-full bg-muted-foreground opacity-50"></div>
                <div sx="w-1 h-2 rounded-full bg-muted-foreground opacity-50"></div>
                <div sx="w-1 h-2 rounded-full bg-muted-foreground opacity-50"></div>
                <div sx="w-1 h-2 rounded-full bg-muted-foreground opacity-50"></div>
                <div sx="w-1 h-2 rounded-full bg-muted-foreground opacity-50"></div>
              </row>
            {/if}
            <button on:click={togglePlay} variant="solid" tone="primary" motion:press={{ scale: 'press' }}>
              {playing() ? 'Pause' : 'Play'}
            </button>
          </row>
        </card>
        <p sx="text-xs text-muted-foreground font-mono">
          {playing() ? 'playing · the clock is an effect interval' : 'paused — the effect’s cleanup stopped the clock'}
        </p>
        </column>
        </center>
      </column>
    </column>
  </row>
  <p>
    The position clock is an effect that <em>tracks</em> <code>playing()</code>:
    pausing re-runs it, and its cleanup stops the interval — the same cleanup
    that runs on unmount. The equalizer isn't choreographed either way: the
    bars only exist while <code>playing()</code> keeps them in the DOM, and
    <code>motion:loop</code> is the entire animation.
  </p>

  <row sx="mt-8 pt-4 border-t border-t-border" gap="2">
    <a variant="default" href="/sail/why">← Why Sail</a>
    <a variant="solid" tone="primary" href="/components">Component library →</a>
  </row>
</column>
