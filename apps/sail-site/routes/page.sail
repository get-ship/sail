<!--
  SAIL landing — design 2a, "One program, both sides." A docs-shaped
  introduction that lives inside the site's own chrome (top product switcher +
  rail, provided by <sail-site-shell>). The hero states the pitch and proves it
  ships today; the centerpiece is ONE synced panel — the exact filterable Team
  program printed on the left, running for real on the right (type filters,
  Invite appends, buttons press). A narrated step strip + dot rail say what
  "synced" means for a reader who never touches the keyboard. A 3-up feature
  strip closes it. Everything below the hero is genuinely interactive.
-->

<controller>
// Landing — the "code ⇆ live" centerpiece. `filteredList` drives the real
// filter; `invite` appends a real signal. An auto-cycling step (label + dot
// rail) narrates the sync loop so the panel reads as a demo even at rest.

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
    // Narration — a slow cycle so the panel is alive before you touch it.
    const step = signal(0)
    effect(() => {
      const t = setInterval(() => step.set((n) => (n + 1) % STEPS.length), 2600)
      return () => clearInterval(t)
    })

    // The real program, printed on the left and running on the right.
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

    // The install line copies itself — a small working detail.
    const copied = signal(false)
    const copyInstall = (): void => {
      // Clipboard can be blocked (no permission / insecure origin); swallow the
      // rejection so a copy attempt never surfaces an uncaught error.
      try {
        void navigator.clipboard?.writeText('npm install @ship/sail-runtime').catch(() => {})
      } catch {
        /* clipboard unavailable — the label still confirms the intent */
      }
      copied.set(true)
      setTimeout(() => copied.set(false), 1400)
    }

    return {
      code,
      step, stepLabel: () => STEPS[step()],
      people, query, filtered, invite,
      copied, copyInstall,
    }
})
</controller>

<column gap="8" sx="max-w-full min-w-0">
  <!-- ══ HERO ═══════════════════════════════════════════════════════════ -->
  <column gap="5" sx="max-w-breakpoint-tablet relative">
    <div sx="absolute top-0 right-0 w-64 h-64 bg-glow-accent pointer-events-none z-base"></div>
    <p sx="font-mono text-xs uppercase tracking-wider text-accent">Introduction</p>
    <h1 sx="text-display font-bold tracking-tighter leading-none">SAIL</h1>
    <p sx="text-lg text-muted-foreground leading-relaxed max-w-breakpoint-mobile">
      A batteries-included language for web pages. Components, layout, styling,
      theming, animation and state ship in the syntax — a page is markup, and
      nothing has to be assembled first.
    </p>
    <button variant="outline" on:click={copyInstall} sx="w-fit font-mono text-sm" motion:press={{ scale: 'press' }}>
      <row gap="3" align="center">
        <span sx="text-muted-foreground select-none">$</span>
        <span>npm install @ship/sail-runtime</span>
        <span sx="text-muted-foreground">{copied() ? '✓ copied' : '⧉'}</span>
      </row>
    </button>
  </column>

  <!-- ══ ONE PROGRAM, BOTH SIDES — the centerpiece ═════════════════════ -->
  <column gap="0" sx="border border-border rounded-xl bg-surface-1 overflow-hidden min-w-0 shadow-lg">
    <row justify="between" align="center" sx="px-6 py-4 border-b border-border">
      <p sx="text-md"><strong>One program, both sides.</strong> <span sx="text-muted-foreground">Watch the highlight — or just use the app.</span></p>
      <row gap="1" align="center">
        <span dot tone={step() === 0 ? 'accent' : 'muted'} size="sm"></span>
        <span dot tone={step() === 1 ? 'accent' : 'muted'} size="sm"></span>
        <span dot tone={step() === 2 ? 'accent' : 'muted'} size="sm"></span>
        <span dot tone={step() === 3 ? 'accent' : 'muted'} size="sm"></span>
      </row>
    </row>

    <row align="stretch" sx="min-w-0">
      <!-- left: the source -->
      <column sx="grow-1 min-w-0 basis-0 border-r border-border p-5 overflow-x-auto bg-surface-0">
        <code-block code={code.team}></code-block>
      </column>

      <!-- right: the same program, running -->
      <column gap="4" sx="grow-1 min-w-0 basis-0 p-5">
        <row justify="between" align="baseline">
          <strong sx="text-md">Team</strong>
          <p sx="font-mono text-xs text-muted-foreground">{filtered().length} of {people().length}</p>
        </row>
        <input placeholder="Filter people — try 'eng'" bind:value={query} />
        <column gap="2" sx="min-h-32">
          {#for p of filtered() key={p.id}}
            <row gap="3" align="center" sx="p-3 border border-border rounded-lg bg-surface-2 hover:border-accent transition-colors" motion:enter={{ opacity: 'fade', translate-y: 'spring:snappy' }} motion:hover={{ scale: 'spring:gentle' }}>
              <avatar name={p.name} tone="accent" size="sm"></avatar>
              <column gap="0" sx="grow-1 min-w-0">
                <strong sx="text-sm truncate">{p.name}</strong>
                <p sx="text-xs text-muted-foreground">{p.team}</p>
              </column>
              <badge tone={p.status === 'active' ? 'success' : 'muted'}>{p.status}</badge>
            </row>
          {/for}
          {#if filtered().length === 0}
            <center sx="p-5 border border-dashed border-border rounded-lg text-sm text-muted-foreground" motion:enter={{ opacity: 'fade' }}>no matches — filtered() is empty</center>
          {/if}
        </column>
        <row gap="2">
          <button on:click={invite} variant="solid" tone="accent" motion:press={{ scale: 'press' }}>Invite</button>
          <button variant="outline">Export</button>
        </row>
      </column>
    </row>

    <row gap="3" align="center" sx="px-6 py-3 border-t border-border font-mono text-sm text-accent">
      <span dot tone="accent" size="sm" pulse></span>
      {stepLabel()}
    </row>
  </column>

  <!-- ══ WHAT COMES WITH IT ════════════════════════════════════════════ -->
  <row gap="3" align="stretch" sx="max-w-full">
    <card interactive sx="p-5 grow-1 min-w-0 basis-0">
      <strong sx="text-accent text-md">48 components</strong>
      <p sx="text-sm text-muted-foreground mt-2 leading-relaxed">Accessible, keyboard-complete, themed.</p>
    </card>
    <card interactive sx="p-5 grow-1 min-w-0 basis-0">
      <strong sx="text-accent text-md">Layout &amp; styling</strong>
      <p sx="text-sm text-muted-foreground mt-2 leading-relaxed"><code>&lt;row&gt;</code> <code>&lt;column&gt;</code> <code>&lt;grid&gt;</code>, sx tokens — no CSS to write.</p>
    </card>
    <card interactive sx="p-5 grow-1 min-w-0 basis-0">
      <strong sx="text-accent text-md">No virtual DOM</strong>
      <p sx="text-sm text-muted-foreground mt-2 leading-relaxed">Fine-grained signals — Solid-class updates, no diff.</p>
    </card>
  </row>
</column>
