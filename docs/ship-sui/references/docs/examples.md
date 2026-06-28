---
group: Views
order: 7
title: Canonical examples
description: Three example shapes to anchor the framework's discipline. Home (minimal controller + live data), agents (object controller + a derived bundle), form (controller hook + two-way binding).
---

# Canonical examples

These three examples cover the discipline boundaries. Each tier has a representative; each example exercises a different combination of primitives.

## Tier 1 — home (minimal controller + live data)

The smallest reactive shape: a single signal in the controller's `scope:` factory plus an inline arrow handler in the template, alongside one `<data live>` subscription. Worked exemplar in the repo: `views/home/`.

```html
<!-- views/home/page.sui (excerpt) -->
<data name="git" source="ship.git.overview()" live />

<section class="counter">
  <p>You've opened the home page <strong>{seen()}</strong> times this session.</p>
  <sui-button size="sm" variant="outline" on:click={() => seen.set(seen() + 1)}>Bump</sui-button>
</section>
```

```ts
// views/home/controller.ts (excerpt)
import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => ({ seen: signal(0) }),
  // derived: { branch, clean, headShort, … } project the `git` envelope
})
```

What this exercises: `scope:` factory, expression interpolation, `on:click` with a call-only arrow body, a `<data live>` subscription.

What this avoids: a full `derived` / `on` stanza for trivial state. Inline handlers cover the simplest mutations.

## Tier 2 — agents (object controller + a derived bundle)

The actual working view in the repo today (`views/agents/`). Server data via `<data>`, two memos, one navigation handler. No hooks yet — the hook-based version is below.

```html
<!-- views/agents/page.sui (excerpt) -->
<data name="agents" source="ship.agents.summaries()" live />

<header class="agents-head">
  <h1>Agents</h1>
  <p>{total()} agent{total() === 1 ? '' : 's'}</p>
  <sui-button tone="primary" on:click={newAgent}>New agent</sui-button>
</header>

{#while-loading of agents}
  <p>Loading agents…</p>
{/while-loading}

{#while-error err of agents}
  <p>Error loading agents: {err.message}</p>
{/while-error}

{#if agents().kind === 'ok'}
  {#if items().length === 0}
    <sui-empty-state>No agents yet.</sui-empty-state>
  {:else}
    <div class="agents-grid">
      {#for a of items() key={a.id}}
        <sui-list-tile navigable={'/agents/' + a.id}>{a.name}</sui-list-tile>
      {/for}
    </div>
  {/if}
{/if}
```

```ts
// views/agents/controller.ts
import { controller } from '@ship/sui-runtime'
import type { LiveSignal } from '@ship/sui-runtime'
import type { AgentsSummariesResponse, AgentSummary } from '@ship/sdk-client'

export default controller({
  derived: {
    items: ({ agents }: { agents: LiveSignal<AgentsSummariesResponse> }): AgentSummary[] => {
      const s = agents()
      return s.kind === 'ok' ? s.value.items : []
    },
    total: ({ items }: { items: () => AgentSummary[] }) => items().length,
  },
  on: {
    newAgent: ({ navigate }: { navigate: (path: string) => void }) =>
      navigate('/agents/create'),
  },
})
```

What this exercises:

- `<data live>` subscription — daemon-streamed data, declared in the template
- Object-form controller (no closure-scoped vars; `controller(def)` enough)
- `derived` reading template-declared scope (`agents`, then `items`)
- `on` handler reading framework-injected `navigate` via the proxy fallthrough
- `{#while-loading}` / `{#while-error err}` boundary blocks

What this avoids: a `live:` section in the controller, calling `live()` from JS.

## Tier 3 — agents with hooks

The same view, scaled with the v1 hooks (`useFilteredList`, `useSelection`, `<use-kbd>`) — all shipped, see [status.md](status.md).

```html
<!-- views/agents/page.sui -->
<data name="agents" source="ship.agents.summaries()" live />

<use-kbd>
  <on key="/"        action="toggleSearch" />
  <on key="cmd+n"    action="newAgent" />
  <on key="escape"   action="clearSelection" />
</use-kbd>

<header class="agents-head">
  <h1>Agents — {filter.filtered().length}</h1>
  {#if shell.currentUser}
    <span class="agents-user">{shell.currentUser.name}</span>
  {/if}
  <sui-button tone="primary" on:click={newAgent}>New agent</sui-button>
</header>

{#if searchOpen()}
  <input
    bind:value={filter.query}
    placeholder="Search agents…"
  />
{/if}

{#if agents().kind === 'ok'}
  <ul class="agents-grid">
    {#for a of filter.filtered() key={a.id}}
      <li>
        <sui-list-tile
          tone={sel.selected()?.id === a.id ? 'primary' : 'default'}
          on:click={() => sel.select(a.id)}
        >{a.name}</sui-list-tile>
      </li>
    {/for}
  </ul>
{/if}
```

The search-toggle signal lives in the controller's `scope:` factory:

```ts
// views/agents/controller.ts
import { controller, signal, useFilteredList, useSelection } from '@ship/sui-runtime'

export default controller({
  scope: () => ({ searchOpen: signal(false) }),
  use: ({ agents }) => ({
    filter: useFilteredList(agents, (a, q) => a.name.toLowerCase().includes(q.toLowerCase())),
    sel:    useSelection(agents, (a) => a.id),
  }),
  on: {
    toggleSearch:    (s)              => { s.searchOpen = !s.searchOpen },
    newAgent:        ({ navigate })   => navigate('/agents/create'),
    clearSelection:  ({ sel })        => sel.clear(),
  },
})
```

What this exercises beyond tier 2:

- Template hooks (`<use-kbd>` for document keybindings
- Controller hooks (`useFilteredList`, `useSelection`) for UI pattern bundles
- Per-section scope destructure (`use: ({ agents }) => ...`)
- Hook namespacing in templates (`filter.filtered`, `sel.selected`)
- Action references inside `<use-kbd>` resolving to `on.*`

## Tier 3 — settings form (object controller + useForm hook)

Forms exercise two-way binding, validation, and the `useForm` controller hook — all shipped, see [status.md](status.md).

```html
<!-- views/settings/page.sui -->
<data name="settings" source="ship.settings.state()" />

<form on:submit={form.submit}>
  <label>
    Theme
    <select bind:value={form.values.theme}>
      <option value="light">Light</option>
      <option value="dark">Dark</option>
      <option value="system">System</option>
    </select>
  </label>

  <label>
    Accent color
    <input bind:value={form.values.accent} />
  </label>
  {#if form.errors().accent}
    <p class="error">{form.errors().accent}</p>
  {/if}

  <button type="submit" disabled={form.pending() || !form.dirty()}>
    {form.pending() ? 'Saving…' : 'Save'}
  </button>
  <button type="button" on:click={form.reset} disabled={!form.dirty()}>
    Reset
  </button>
</form>
```

```ts
// views/settings/controller.ts
import { controller, useForm } from '@ship/sui-runtime'

export default controller({
  use: ({ ship, settings }) => ({
    form: useForm({
      initial:  () => settings(),
      validate: (v) => ({
        accent: /^#[0-9a-f]{6}$/i.test(v.accent) ? null : 'must be a hex color',
      }),
      submit:   (v) => ship.settings.write(v),
    }),
  }),
})
```

What this exercises:

- Object-form `controller(def)` (no closure-scoped vars needed)
- `useForm` controller hook bundling `state` (values, errors, pending, dirty), `derived` (errors per field), `on` (change, submit, reset)
- `bind:value` two-way binding on a `<select>` and an `<input>`
- `on:submit` on the form element

What this avoids: hand-rolled form state with five state variables and four handlers. The pattern lives in the hook.

## What these examples establish

| Decision | Where it lives |
|---|---|
| Server data | `<data>` in the template |
| Local state (any) | `scope:` factory in the controller |
| Computed-initial state (`crypto.randomUUID()`, etc.) | `scope:` placeholder + assignment in `mount` |
| Computed values | Controller `derived:` (named) or inline expressions (one-off) |
| Event handlers | Controller `on:` |
| UI pattern bundles | Controller hook in `use:` |
| Host primitives | Template hook (`<use-*>`) |
| Lifecycle | Controller `mount` / `unmount` |

Three examples, three tiers, all five controller sections, both hook categories, every binding form. If a new view doesn't fit one of these shapes, it is *almost certainly* combinable from these primitives. If it isn't, escalate to a discipline conversation — see [the discipline](index.md#the-discipline).
