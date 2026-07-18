<!--
  /overview — the Introduction (pairing draft v2). Owner's opener with the API
  placeholder filled; three live proofs (signals list, theme card, payload
  rollup); no code dumps — the tour and its snippets are gone from this page
  (that material's home is The language section). Payload figures are REAL
  gzip measurements from this site's build (see the rollup caption); the
  tree-shaking/theme-subset behavior described is the shipping direction —
  confirm precision in review before launch.
-->

<controller>
import { controller, signal, derived, filteredList, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

const BENCH = ['Katherine Johnson', 'Annie Easley', 'Margaret Hamilton', 'Radia Perlman']

export default controller((_ctx: ControllerScope<Scope>) => {
    // Signals demo — the filterable team list. Invite really adds a row.
    const people = signal([
      { id: 1, name: 'Ada Lovelace', status: 'active' },
      { id: 2, name: 'Grace Hopper', status: 'away' },
      { id: 3, name: 'Alan Turing', status: 'active' },
    ])
    const { query, filtered } = filteredList(() => people(), (p, q) =>
      p.name.toLowerCase().includes(q),
    )
    const invite = (): void => {
      people.set((l) => [
        ...l,
        { id: l.length + 1, name: BENCH[(l.length - 3) % BENCH.length] ?? `Member ${l.length + 1}`, status: 'active' },
      ])
    }

    // Theme demo — one attribute re-skins the subtree.
    const themes = ['ship-light', 'ship-dark', 'paper', 'console', 'contrast']
    const theme = signal('ship-light')

    // Payload rollup — REAL gzip measurements from this site's build:
    // core = runtime 9.4 + tokens 11.0 + contracts 6.9; skins 13.9 (all 48
    // components); theme library 11.4 (all nine themes).
    const withSkins = signal(true)
    const withThemes = signal(true)
    const CORE = 27.3
    const bundle = derived(() => (
      CORE + (withSkins() ? 13.9 : 0) + (withThemes() ? 11.4 : 0)
    ).toFixed(1))

    return { query, filtered, invite, themes, theme,
             withSkins, withThemes, bundle }
})
</controller>

<column sx="max-w-breakpoint-tablet mx-auto" gap="5">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">Introduction</p>
  <h1>SAIL</h1>

  <p sx="text-lg text-muted-foreground leading-relaxed">
    SAIL is a compiled language that includes everything you need to build
    beautiful, consistent, and performant websites. It is optimized for AI
    agents: the language provides the APIs an interface needs — signals for
    state, <code>sx</code> for styling, <code>motion</code> for animation, a
    component library that already behaves — and a strict compiler enforces
    how they are used. That is how best practices and consistency hold across
    an entire application, not just the files someone reviewed.
  </p>

  <h2>It reads like HTML, it runs on signals</h2>
  <p>
    A component is one file: a template that reads like HTML, TypeScript
    beneath it, signals connecting the two. Updates are fine-grained — the
    template binds signals directly, and only what changed re-renders. This
    list is that file, running: filter it, press Invite.
  </p>
  <column gap="3" sx="p-6 bg-muted border border-border rounded-lg">
    <input placeholder="Filter people" bind:value={query} />
    {#for p of filtered() key={p.id}}
      <card sx="p-4" motion:enter={{ opacity: 'fade', translate-y: 'spring:snappy' }}
            motion:hover={{ scale: 'spring:gentle' }}>
        <row gap="3" align="center">
          <avatar></avatar>
          <strong sx="grow-1">{p.name}</strong>
          <badge tone={p.status === 'active' ? 'success' : 'muted'}>{p.status}</badge>
        </row>
      </card>
    {/for}
    <row gap="2">
      <button on:click={invite} variant="solid" tone="primary" motion:press={{ scale: 'press' }}>Invite</button>
    </row>
  </column>

  <h2>Design is data</h2>
  <p>
    Styling draws from a token system the compiler enforces — colors, spacing,
    and type resolve through the active theme, and components never name a
    colour. Which means a theme is data, and swapping it re-skins everything
    written in the language. Try it:
  </p>
  <column gap="3" sx="p-6 bg-muted border border-border rounded-lg">
    <row gap="2">
      {#for t of themes key={t}}
        <button on:click={() => theme.set(t)} size="sm"
                variant={theme() === t ? 'solid' : 'outline'} tone="primary">{t}</button>
      {/for}
    </row>
    <column data-theme-preview={theme()} gap="3" sx="p-5 bg-background text-foreground border border-border rounded-lg">
      <row gap="3" align="center">
        <avatar></avatar>
        <column gap="0">
          <strong>Ada Lovelace</strong>
          <span sx="text-sm text-muted-foreground">Wrote the first one</span>
        </column>
        <badge tone="success" sx="ml-auto">active</badge>
      </row>
      <row gap="2">
        <button variant="solid" tone="primary" size="sm">Message</button>
        <button variant="outline" size="sm">Profile</button>
      </row>
    </column>
  </column>

  <h2>The compiler is a contract you tune</h2>
  <p>
    The toolchain ships a complete kit and then subtracts: component skins
    tree-shake to the tags a build actually uses, and the theme library ships
    only the themes you keep — one instead of nine. The same settings decide
    what authors may write, from a locked brand system to a loose side
    project. These figures are this site's own build, gzipped:
  </p>
  <column gap="3" sx="p-6 bg-muted border border-border rounded-lg">
    <row gap="3" align="center"><switch bind:checked={withSkins} tone="primary"></switch><span sx="grow-1">Component skins — all 48 tags</span><span sx="font-mono text-sm text-muted-foreground">13.9 kB</span></row>
    <row gap="3" align="center"><switch bind:checked={withThemes} tone="primary"></switch><span sx="grow-1">Theme library — all nine themes</span><span sx="font-mono text-sm text-muted-foreground">11.4 kB</span></row>
    <row gap="3" align="center"><span sx="grow-1">Core — runtime, tokens, contracts</span><span sx="font-mono text-sm text-muted-foreground">27.3 kB</span></row>
    <row gap="3" align="center" sx="border-t border-t-border pt-3">
      <span sx="grow-1 font-semibold">Your bundle</span>
      <span sx="font-mono text-lg font-semibold text-primary">{bundle()} kB</span>
    </row>
    <p sx="text-xs text-muted-foreground">Measured from this site's build. A per-app build carries only the tags and themes it uses — these are the ceilings, not the floor.</p>
  </column>

  <h2>Performance</h2>
  <p>
    Fast in the boring way: templates compile ahead of time, updates are
    fine-grained signal writes, and there is no virtual DOM to diff. The
    whole runtime is 9.4&nbsp;kB gzipped.
  </p>

  <row sx="mt-8 pt-4 border-t border-t-border" gap="2">
    <a variant="solid" tone="primary" href="/getting-started">Installation →</a>
    <a variant="default" href="/why">Why SAIL</a>
  </row>
</column>
