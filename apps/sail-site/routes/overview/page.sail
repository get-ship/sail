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

    return { query, filtered, invite }
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

  <column gap="3">
    <h2>Design is data</h2>
    <p>
      Styling draws from a token system the compiler enforces — colors,
      spacing, and type resolve through the active theme, and components never
      name a colour. Which means a theme is data, and swapping it re-skins
      everything written in the language. Pick one — this entire site is
      written in SAIL:
    </p>
    <theme-picker inline></theme-picker>
  </column>

  <column sx="max-w-breakpoint-tablet" gap="3">
    <h2>The compiler pushes back</h2>
    <p>
      Every <code>.sail</code> file passes through a compiler that checks more
      than syntax. A colour that isn't a token, a stylesheet smuggled in beside
      a component, an animation bolted on outside the motion system — these
      don't compile. And a refusal is never a dead end: the error names the
      correct form, so the next attempt — an agent's or yours — is the right
      one.
    </p>
    <p>
      How strict is up to you. Each project carries a small config that sets
      the contract: a brand platform locks styling to tokens and keeps raw
      containers rare; a personal site loosens the same dials. Same language,
      same compiler — a different set of promises, chosen by the team that has
      to live with the code.
    </p>

    <h2>What's in the box</h2>
    <p>
      Batteries included means the parts every site ends up assembling are
      already here, as language rather than dependencies. Interface patterns —
      dialogs, comboboxes, tables, tabs, toasts — are tags, keyboard-complete
      and accessible, skinned by your theme. Layout is its own vocabulary
      (<code>row</code>, <code>column</code>, <code>grid</code>) rather than a
      CSS technique. Motion is declarative and respects reduced motion by
      default. Data has a first-class shape: <code>query()</code> turns a
      client call into loading, error, and value states the template has to
      handle. Forms, icons, and a typographic system for prose round it out.
      The practical difference: there is no stack to choose — and nothing can
      drift out of step with your design system, because everything reads the
      same tokens.
    </p>
    <p><a href="/components">Explore the components →</a> · <a href="/motion">Animation</a> · <a href="/data">Data &amp; forms</a></p>

    <h2>Performance</h2>
    <p>
      Fast by construction more than by tuning. Templates compile ahead of
      time and updates are fine-grained signal writes — no virtual DOM,
      nothing to diff. The runtime is small and tree-shakable, and the design
      system adds no JavaScript at all: components, themes, and styling ship
      as CSS. That matters most compared to the usual stack, where the
      component library and the design system arrive as their own runtime on
      top of the framework's.
    </p>

    <p sx="text-sm text-muted-foreground">
      SAIL is in alpha. It builds real websites today — this site is one — and
      the language is still settling; we'd rather be honest about that than
      polished about it.
    </p>

    <row sx="mt-4 pt-4 border-t border-t-border" gap="2">
      <a variant="solid" tone="primary" href="/getting-started">Installation →</a>
      <a variant="default" href="/tutorial">Tutorial</a>
      <a variant="default" href="/showcase">Showcase</a>
      <a variant="default" href="/why">Why SAIL</a>
    </row>
  </column>
</column>
