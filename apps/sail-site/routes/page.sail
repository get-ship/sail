<!--
  Ship UI landing — option 2a ("Code ⇆ Live"): a docs-shaped introduction, not
  a long marketing scroll. The hero states the pitch, an install line proves
  it ships today, and one synced panel puts the exact program that's running
  right next to the program's own source — the same demo /overview uses for
  its first working example, shown here as the front door. A 3-up feature
  strip closes it. Deeper tours (motion, theming, reactive data, deck, ai,
  components) live one click away via the docs chrome (top switcher + rail)
  that now wraps this route too — see the report for the full route map.
-->

<controller>
// Landing — option 2a. One synced "code ⇆ live" panel: a filterable team
// list where Invite actually appends a signal, narrated by an auto-cycling
// step label + dot rail so a reader who never touches the keyboard still
// sees what "synced" means.

import { controller, signal, effect, filteredList, type ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

const STEPS = [
  'Filtering — type to narrow filtered().',
  'Reactive — the list recomputes on every keystroke.',
  'Inviting — one click appends with signal.set().',
  'Synced — the code on the left is the exact program running.',
]
const BENCH = ['Katherine Johnson', 'Annie Easley', 'Margaret Hamilton', 'Radia Perlman']

export default controller((_ctx: ViewScope<Scope>) => {
    const step = signal(0)
    effect(() => {
      const t = setInterval(() => step.set((n) => (n + 1) % STEPS.length), 2600)
      return () => clearInterval(t)
    })

    const people = signal([
      { id: 1, name: 'Ada Lovelace', team: 'Engineering', status: 'active' },
      { id: 2, name: 'Grace Hopper', team: 'Engineering', status: 'away' },
      { id: 3, name: 'Alan Turing', team: 'Research', status: 'active' },
    ])
    const { query, filtered } = filteredList(() => people(), (p, q) =>
      (p.name + ' ' + p.team).toLowerCase().includes(q),
    )
    const invite = (): void => {
      people.set((l) => [
        ...l,
        { id: l.length + 1, name: BENCH[(l.length - 3) % BENCH.length] ?? `Member ${l.length + 1}`, team: 'Engineering', status: 'active' },
      ])
    }

    return {
      code,
      step, stepLabel: () => STEPS[step()],
      people, query, filtered, invite,
    }
})
</controller>

<column gap="8" sx="max-w-full min-w-0">
  <!-- ══ HERO ═══════════════════════════════════════════════════════════ -->
  <column gap="4" sx="max-w-breakpoint-tablet">
    <p sx="font-mono text-xs uppercase tracking-wide text-accent">Introduction</p>
    <h1 sx="text-display font-bold tracking-tighter">Ship UI</h1>
    <p sx="text-lg text-muted-foreground leading-relaxed">
      A batteries-included language for web pages. Components, layout,
      styling, theming, animation and state ship in the syntax — a page is
      markup, and nothing has to be assembled first.
    </p>
    <row gap="2" align="center" sx="font-mono text-sm p-3 border border-border rounded-lg bg-surface-1 w-fit">
      <span sx="text-muted-foreground">$</span> npm install @ship/sail-runtime
    </row>
    <row gap="3">
      <a variant="solid" tone="primary" href="/getting-started" motion:press={{ scale: 'press' }}>Get started</a>
      <a variant="default" href="/why">Why a language?</a>
    </row>
  </column>

  <!-- ══ ONE PROGRAM, BOTH SIDES ═══════════════════════════════════════ -->
  <column gap="0" sx="border border-border rounded-xl bg-surface-1 overflow-hidden min-w-0">
    <row justify="between" align="center" sx="px-6 py-4 border-b border-border">
      <column gap="1">
        <strong sx="text-md">One program, both sides.</strong>
        <p sx="text-sm text-muted-foreground">{stepLabel()}</p>
      </column>
      <row gap="1" align="center">
        <span dot tone={step() === 0 ? 'primary' : 'muted'} size="sm"></span>
        <span dot tone={step() === 1 ? 'primary' : 'muted'} size="sm"></span>
        <span dot tone={step() === 2 ? 'primary' : 'muted'} size="sm"></span>
        <span dot tone={step() === 3 ? 'primary' : 'muted'} size="sm"></span>
      </row>
    </row>
    <row gap="0" align="stretch" sx="max-w-full">
      <column sx="p-6 border-r border-border grow-1 min-w-0 overflow-x-auto">
        <code-block code={code.team}></code-block>
      </column>
      <column gap="3" sx="p-6 grow-1 min-w-0">
        <row justify="between" align="center">
          <strong>Team</strong>
          <p sx="font-mono text-xs text-muted-foreground">{filtered().length} of {people().length}</p>
        </row>
        <input placeholder="Filter people — try 'eng'" bind:value={query} />
        <column gap="2">
          {#for p of filtered() key={p.id}}
            <row gap="3" align="center" sx="p-3 border border-border rounded-lg bg-surface-0" motion:enter={{ opacity: 'fade' }}>
              <avatar name={p.name} tone="primary"></avatar>
              <column gap="0" sx="grow-1">
                <strong sx="text-sm">{p.name}</strong>
                <p sx="text-xs text-muted-foreground">{p.team}</p>
              </column>
              <badge tone={p.status === 'active' ? 'success' : 'muted'}>{p.status}</badge>
            </row>
          {/for}
        </column>
        <row gap="2">
          <button on:click={invite} variant="solid" tone="primary" motion:press={{ scale: 'press' }}>Invite</button>
          <button variant="outline">Export</button>
        </row>
      </column>
    </row>
  </column>

  <!-- ══ WHAT COMES WITH IT ══════════════════════════════════════════════ -->
  <row gap="3" align="stretch" sx="max-w-full">
    <card sx="p-5 grow-1 min-w-0">
      <strong sx="text-accent">48 components</strong>
      <p sx="text-sm text-muted-foreground mt-1">Accessible, keyboard-complete, themed.</p>
    </card>
    <card sx="p-5 grow-1 min-w-0">
      <strong sx="text-accent">Layout &amp; styling</strong>
      <p sx="text-sm text-muted-foreground mt-1"><code>&lt;row&gt;</code> <code>&lt;column&gt;</code> <code>&lt;grid&gt;</code>, sx tokens — no CSS to write.</p>
    </card>
    <card sx="p-5 grow-1 min-w-0">
      <strong sx="text-accent">9.4 kB gzipped</strong>
      <p sx="text-sm text-muted-foreground mt-1">Signals, no virtual DOM — Solid-class performance.</p>
    </card>
  </row>
</column>
