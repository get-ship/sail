<!--
  /overview — the Introduction (pairing draft, 2026-07-18). Opener is the
  owner's framing: what SAIL is, prose that breathes — then three live proofs:
  signals (the team list), themes (a data-theme-preview card), the tunable
  toolchain (switches rolling up a payload figure). Code is the receipt after
  the demo, never the greeting. The "How you use it" tour below is legacy and
  likely migrates to The language section — kept for side-by-side review.
  NOTE: the add-on payload figures are ILLUSTRATIVE until measured builds land
  (#96); core 9.4 kB matches the Performance claim but needs re-measuring
  post motion-inlining.
-->

<controller>
import { controller, signal, derived, filteredList, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

const BENCH = ['Katherine Johnson', 'Annie Easley', 'Margaret Hamilton', 'Radia Perlman']

export default controller((_ctx: ControllerScope<Scope>) => {
    // Demo 1 — signals: the filterable team list. Invite really adds a row.
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

    // Demo 2 — themes: one attribute re-skins the subtree.
    const themes = ['ship-light', 'ship-dark', 'paper', 'console', 'contrast']
    const theme = signal('ship-light')

    // Demo 3 — the tunable toolchain: optional pieces roll up the payload.
    // Add-on figures are illustrative until measured builds land (#96).
    const withComponents = signal(true)
    const withMotion = signal(true)
    const withData = signal(true)
    const withThemes = signal(true)
    const CORE = 9.4
    const bundle = derived(() => (
      CORE +
      (withComponents() ? 12 : 0) +
      (withMotion() ? 6 : 0) +
      (withData() ? 4 : 0) +
      (withThemes() ? 2 : 0)
    ).toFixed(1))

    return { code, query, filtered, invite, themes, theme,
             withComponents, withMotion, withData, withThemes, bundle }
})
</controller>

<column sx="max-w-breakpoint-tablet mx-auto" gap="5">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">Introduction</p>
  <h1>SAIL</h1>

  <p sx="text-lg text-muted-foreground leading-relaxed">
    SAIL is a compiled language that includes everything you need to build
    beautiful, consistent, and performant websites. It is optimized for AI
    agents: the language provides the APIs an interface needs, and a strict
    compiler enforces how they are used — so best practices and consistency
    hold across an entire application, not just the files someone reviewed.
  </p>

  <h2>It reads like HTML, it runs on signals</h2>
  <p>
    A component is one file: a template that reads like HTML, TypeScript
    beneath it, signals connecting the two. Updates are fine-grained — the
    template binds signals directly, and only what changed re-renders. This
    list is live: filter it, press Invite.
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
  <p sx="text-sm text-muted-foreground">The entire file behind that demo — markup, state, motion, no stylesheet:</p>
  <code-block code={code.page}></code-block>

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
    Everything optional is a setting. A team decides what the language
    includes and what authors — human or agent — may write, and the toolchain
    ships exactly that much. Turn pieces off and the payload follows:
  </p>
  <column gap="3" sx="p-6 bg-muted border border-border rounded-lg">
    <row gap="3" align="center"><switch bind:checked={withComponents} tone="primary"></switch><span sx="grow-1">Component library</span><span sx="font-mono text-sm text-muted-foreground">12 kB</span></row>
    <row gap="3" align="center"><switch bind:checked={withMotion} tone="primary"></switch><span sx="grow-1">Motion</span><span sx="font-mono text-sm text-muted-foreground">6 kB</span></row>
    <row gap="3" align="center"><switch bind:checked={withData} tone="primary"></switch><span sx="grow-1">Data &amp; forms</span><span sx="font-mono text-sm text-muted-foreground">4 kB</span></row>
    <row gap="3" align="center"><switch bind:checked={withThemes} tone="primary"></switch><span sx="grow-1">Theme runtime</span><span sx="font-mono text-sm text-muted-foreground">2 kB</span></row>
    <row gap="3" align="center" sx="border-t border-t-border pt-3">
      <span sx="grow-1 font-semibold">Your bundle</span>
      <span sx="font-mono text-lg font-semibold text-primary">{bundle()} kB</span>
    </row>
    <p sx="text-xs text-muted-foreground">Add-on figures are illustrative while measured builds land; the core runtime is always included.</p>
  </column>

  <h2>How you use it</h2>
  <p>
    <strong>Layout is three words.</strong> <code>row</code>, <code>column</code>
    and <code>grid</code> take <code>gap</code>, <code>align</code> and
    <code>justify</code> — there is no flexbox incantation to remember. A raw
    <code>&lt;div&gt;</code> is legal but mute: it can only speak
    <code>sx</code> tokens, so it is a shape, not an escape.
    <a href="/layout">Layout →</a>
  </p>
  <code-block code={code.layout}></code-block>

  <p>
    <strong>Components are tags with props.</strong> Forty-eight of them —
    buttons, dialogs, combobox, tabs, tables, toasts — accessible and
    keyboard-complete. <code>tone</code> picks the colour from the design
    system; <code>variant</code> picks how much of it you get.
    <a href="/components">The library →</a>
  </p>
  <code-block code={code.components}></code-block>

  <p>
    <strong>Styling is a closed vocabulary.</strong> <code>sx</code> speaks
    design-system atoms — spacing, colour, radius, shadow, with states as
    prefixes. A value off the system is a build error, not a review comment.
    <a href="/styling">Styling →</a>
  </p>
  <code-block code={code.styling}></code-block>

  <p>
    <strong>State is signals.</strong> Read by calling, write with
    <code>.set()</code>; <code>derived</code> computes, <code>effect</code>
    reacts and cleans up on unmount. The template binds them directly — no
    virtual DOM in between. <a href="/reactivity">State →</a>
  </p>
  <code-block code={code.state}></code-block>

  <p>
    <strong>Async data is three explicit states.</strong> <code>query()</code>
    turns a client call into a state signal; the template physically cannot render
    the value without having handled loading and error.
    <a href="/data">Data →</a>
  </p>
  <code-block code={code.data}></code-block>

  <p>
    <strong>Animation is one directive.</strong> Eight triggers, nine presets,
    reduced-motion respected everywhere. The framework holds exiting elements
    in the DOM until their animation finishes. <a href="/motion">Animation →</a>
  </p>
  <code-block code={code.motion}></code-block>

  <p>
    <strong>Themes are complete aesthetics.</strong> Nine ship with the
    framework; one attribute swaps everything, because components read tokens
    and never name colours. Try the picker in the top right of this page.
    <a href="/theming">Theming →</a>
  </p>
  <code-block code={code.theming}></code-block>

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
