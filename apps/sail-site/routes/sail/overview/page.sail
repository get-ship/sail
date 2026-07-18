<!--
  /overview — the introduction: what SAIL is, a small working program, and a
  tour that actually teaches usage. The demo below is live and its source is
  the snippet above it — same program.
-->

<controller>
// /overview — the first page carries a WORKING demo, not a still life: a
// filterable team list where Invite actually adds a row. The snippet above
// it is this exact program in its single-file form.

import { controller, signal, filteredList, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

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
    return { code, query, filtered, invite }
})
</controller>

<column sx="max-w-breakpoint-tablet mx-auto" gap="5">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">Introduction</p>
  <h1>SAIL</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    A batteries-included framework for building web pages. It ships a component
    library, a design system, layout, theming, animation, and state — so a page
    is markup, and nothing else has to be assembled first.
  </p>
  <h2>What a page looks like</h2>
  <p>
    A small, complete one — a filtered list with state and animation. No
    imports, no stylesheet; this is the entire file:
  </p>
  <code-block code={code.page}></code-block>

  <p><strong>Running:</strong> type in the filter, press Invite.</p>
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

  <h2>How you use it</h2>
  <p>
    <strong>Layout is three words.</strong> <code>row</code>, <code>column</code>
    and <code>grid</code> take <code>gap</code>, <code>align</code> and
    <code>justify</code> — there is no flexbox incantation to remember. A raw
    <code>&lt;div&gt;</code> is legal but mute: it can only speak
    <code>sx</code> tokens, so it is a shape, not an escape.
    <a href="/sail/layout">Layout →</a>
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
    <a href="/sail/styling">Styling →</a>
  </p>
  <code-block code={code.styling}></code-block>

  <p>
    <strong>State is signals.</strong> Read by calling, write with
    <code>.set()</code>; <code>derived</code> computes, <code>effect</code>
    reacts and cleans up on unmount. The template binds them directly — no
    virtual DOM in between. <a href="/sail/reactivity">State →</a>
  </p>
  <code-block code={code.state}></code-block>

  <p>
    <strong>Async data is three explicit states.</strong> <code>query()</code>
    turns a client call into a state signal; the template physically cannot render
    the value without having handled loading and error.
    <a href="/sail/data">Data →</a>
  </p>
  <code-block code={code.data}></code-block>

  <p>
    <strong>Animation is one directive.</strong> Eight triggers, nine presets,
    reduced-motion respected everywhere. The framework holds exiting elements
    in the DOM until their animation finishes. <a href="/sail/motion">Animation →</a>
  </p>
  <code-block code={code.motion}></code-block>

  <p>
    <strong>Themes are complete aesthetics.</strong> Nine ship with the
    framework; one attribute swaps everything, because components read tokens
    and never name colours. Try the picker in the top right of this page.
    <a href="/sail/theming">Theming →</a>
  </p>
  <code-block code={code.theming}></code-block>

  <h2>Performance</h2>
  <p>
    Fast in the boring way: templates compile ahead of time, updates are
    fine-grained signal writes, and there is no virtual DOM to diff. The
    whole runtime is 9.4&nbsp;kB gzipped.
  </p>

  <row sx="mt-8 pt-4 border-t border-t-border" gap="2">
    <a variant="solid" tone="primary" href="/sail/getting-started">Installation →</a>
    <a variant="default" href="/sail/why">Why SAIL</a>
  </row>
</column>
