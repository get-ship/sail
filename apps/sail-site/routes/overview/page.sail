<!--
  /overview — the Introduction (pairing draft v3).
  v3 rulings applied: query added to the opener's API list; hover feedback is
  CSS (sx hover: + transition atoms), NOT motion: — motion:hover ran a full
  WAAPI pass on enter AND leave (jitter on mouse sweep); prose and demos sit
  side by side; the bundle widget is GONE (bundle story ≠ compiler tunability
  — separate artifact, built when we're ready to compare honestly); tuning
  section is prose about the real policy settings.
-->

<controller>
import { controller, signal, filteredList, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

const BENCH = ['Katherine Johnson', 'Annie Easley', 'Margaret Hamilton', 'Radia Perlman']

export default controller((_ctx: ControllerScope<Scope>) => {
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

    const themes = ['ship-light', 'ship-dark', 'paper', 'console', 'contrast']
    const theme = signal('ship-light')

    return { query, filtered, invite, themes, theme }
})
</controller>

<column sx="max-w-breakpoint-tablet mx-auto" gap="6">
  <column sx="max-w-breakpoint-tablet" gap="3">
    <p sx="font-mono text-xs uppercase tracking-wide text-accent">Introduction</p>
    <h1>SAIL</h1>
    <p sx="text-lg text-muted-foreground leading-relaxed">
      SAIL is a compiled language that includes everything you need to build
      beautiful, consistent, and performant websites. It is optimized for AI
      agents: the language provides the APIs an interface needs — signals for
      state, <code>sx</code> for styling, <code>motion</code> for animation,
      <code>query</code> for data, a component library that already behaves —
      and a strict compiler enforces how they are used. That is how best
      practices and consistency hold across an entire application, not just
      the files someone reviewed.
    </p>
  </column>

  <grid cols="2" gap="6">
    <column gap="3">
      <h2>It reads like HTML, it runs on signals</h2>
      <p>
        A component is one file: a template that reads like HTML, TypeScript
        beneath it, signals connecting the two. Updates are fine-grained —
        the template binds signals directly, and only what changed
        re-renders. No virtual DOM, no diffing.
      </p>
      <p sx="text-sm text-muted-foreground">
        The list on the right is that file, running — filter it, press
        Invite.
      </p>
    </column>
    <column gap="3" sx="p-5 bg-muted border border-border rounded-lg">
      <input placeholder="Filter people" bind:value={query} />
      {#for p of filtered() key={p.id}}
        <card sx="p-3 transition-shadow duration-fast hover:shadow-hover"
              motion:enter={{ opacity: 'fade', translate-y: 'spring:snappy' }}>
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
  </grid>

  <grid cols="2" gap="6">
    <column gap="3">
      <h2>Design is data</h2>
      <p>
        Styling draws from a token system the compiler enforces — colors,
        spacing, and type resolve through the active theme, and components
        never name a colour. Which means a theme is data, and swapping it
        re-skins everything written in the language.
      </p>
      <p sx="text-sm text-muted-foreground">
        The card on the right is one attribute away from a different brand.
      </p>
    </column>
    <column gap="3" sx="p-5 bg-muted border border-border rounded-lg">
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
  </grid>

  <column sx="max-w-breakpoint-tablet" gap="3">
    <h2>The compiler is a contract you tune</h2>
    <p>
      Every project carries a config that decides what authors — human or
      agent — may write. A locked brand system runs strict: styling speaks
      tokens only, raw containers are limited, variants are closed. A looser
      project opens the same dials. The compiler holds whichever line you
      draw, and when generated code steps over it, the answer names the
      correct form instead of just refusing.
    </p>

    <h2>Performance</h2>
    <p>
      Fast in the boring way: templates compile ahead of time, updates are
      fine-grained signal writes, and there is no virtual DOM to diff. The
      whole runtime is 9.4&nbsp;kB gzipped.
    </p>

    <row sx="mt-4 pt-4 border-t border-t-border" gap="2">
      <a variant="solid" tone="primary" href="/getting-started">Installation →</a>
      <a variant="default" href="/why">Why SAIL</a>
    </row>
  </column>
</column>
