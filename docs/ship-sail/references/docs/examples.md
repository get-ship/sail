---
group: Views
order: 7
title: Canonical examples
description: Three example shapes to anchor the framework's discipline. Home (minimal controller + live data), agents (controller + a derived bundle), form (composition + two-way binding).
---

# Canonical examples

These three examples cover the discipline boundaries. Each tier has a representative; each example exercises a different combination of primitives.

## Tier 1 — home (minimal controller + live data)

The smallest reactive shape: a single signal returned from the inline controller plus an inline arrow handler in the template, alongside one `<data live>` subscription. Worked exemplar in the repo: `views/home/`.

```html
<!-- views/home/page.sail (excerpt) -->
<data name="git" source="ship.git.overview()" live />

<section class="counter">
  <p>You've opened the home page <strong>{seen()}</strong> times this session.</p>
  <sail-button size="sm" variant="outline" on:click={() => seen.set(seen() + 1)}>Bump</sail-button>
</section>
```

```ts
<controller>
// views/home/page.sail (excerpt)
import { signal, derived } from '@ship/sail-runtime'

export default controller(() => {
  const seen = signal(0)
  // derived(...) values project the `git` envelope: branch, clean, headShort, …
  return { seen }
})
</controller>
```

What this exercises: the controller function returning a bag of signals, expression interpolation, `on:click` with a call-only arrow body, a `<data live>` subscription.

What this avoids: a pile of named handlers for trivial state. Inline handlers cover the simplest mutations.

## Tier 2 — agents (controller + a derived bundle)

The actual working view in the repo today (`views/agents/`). Server data via `<data>`, two memos, one navigation handler. No compositions yet — the composition-based version is below.

```html
<!-- views/agents/page.sail (excerpt) -->
<data name="agents" source="ship.agents.summaries()" live />

<header class="agents-head">
  <h1>Agents</h1>
  <p>{total()} agent{total() === 1 ? '' : 's'}</p>
  <sail-button tone="primary" on:click={newAgent}>New agent</sail-button>
</header>

{#while-loading of agents}
  <p>Loading agents…</p>
{/while-loading}

{#while-error err of agents}
  <p>Error loading agents: {err.message}</p>
{/while-error}

{#if agents().kind === 'ok'}
  {#if items().length === 0}
    <sail-empty-state>No agents yet.</sail-empty-state>
  {:else}
    <div class="agents-grid">
      {#for a of items() key={a.id}}
        <sail-list-tile navigable={'/agents/' + a.id}>{a.name}</sail-list-tile>
      {/for}
    </div>
  {/if}
{/if}
```

```ts
<controller>
// views/agents/page.sail
import { derived } from '@ship/sail-runtime'
import type { LiveSignal } from '@ship/sail-runtime'
import type { AgentsSummariesResponse, AgentSummary } from '@ship/sdk-client'

export default controller(({ agents, navigate }: { agents: LiveSignal<AgentsSummariesResponse>; navigate: (path: string) => void }) => {
  const items = derived((): AgentSummary[] => {
    const s = agents()
    return s.kind === 'ok' ? s.value.items : []
  })
  const total = derived(() => items().length)

  const newAgent = () => navigate('/agents/create')

  return { items, total, newAgent }
})
</controller>
```

What this exercises:

- `<data live>` subscription — daemon-streamed data, declared in the template
- The controller function reading its `ctx` argument directly (no closure-scoped vars needed)
- `derived` reading template-declared scope (`agents`, then `items`)
- A handler reading framework-injected `navigate` off `ctx`
- `{#while-loading}` / `{#while-error err}` boundary blocks

What this avoids: a separate live-subscription section in the controller, calling `query()` from JS.

## Tier 3 — agents with compositions

The same view, scaled with the v1 compositions (`useFilteredList`, `useSelection`, `<use-kbd>`) — all shipped, see [status.md](status.md).

```html
<!-- views/agents/page.sail -->
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
  <sail-button tone="primary" on:click={newAgent}>New agent</sail-button>
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
        <sail-list-tile
          tone={sel.selected()?.id === a.id ? 'primary' : 'default'}
          on:click={() => sel.select(a.id)}
        >{a.name}</sail-list-tile>
      </li>
    {/for}
  </ul>
{/if}
```

The search-toggle signal lives in the inline controller, alongside two compositions called directly:

```ts
<controller>
// views/agents/page.sail
import { signal, useFilteredList, useSelection } from '@ship/sail-runtime'

export default controller(({ agents, navigate }) => {
  const searchOpen = signal(false)

  const filter = useFilteredList(agents, (a, q) => a.name.toLowerCase().includes(q.toLowerCase()))
  const sel    = useSelection(agents, (a) => a.id)

  const toggleSearch   = () => searchOpen.set((v) => !v)
  const newAgent        = () => navigate('/agents/create')
  const clearSelection  = () => sel.clear()

  return { searchOpen, filter, sel, toggleSearch, newAgent, clearSelection }
})
</controller>
```

What this exercises beyond tier 2:

- Template hooks (`<use-kbd>` for document keybindings)
- Compositions (`useFilteredList`, `useSelection`) — plain functions called inside the controller, contributing UI pattern bundles
- The controller function destructuring `ctx` directly (`({ agents, navigate }) => ...`)
- Bundle namespacing in templates (`filter.filtered`, `sel.selected`) — whatever shape a composition returns flows straight into the returned scope
- Action references inside `<use-kbd>` resolving to handlers returned from the controller

## Tier 3 — settings form (controller + useForm composition)

Forms exercise two-way binding, validation, and the `useForm` composition — all shipped, see [status.md](status.md).

```html
<!-- views/settings/page.sail -->
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
<controller>
// views/settings/page.sail
import { useForm } from '@ship/sail-runtime'

export default controller(({ ship, settings }) => {
  const form = useForm({
    initial:  () => settings(),
    validate: (v) => ({
      accent: /^#[0-9a-f]{6}$/i.test(v.accent) ? null : 'must be a hex color',
    }),
    submit:   (v) => ship.settings.write(v),
  })

  return { form }
})
</controller>
```

What this exercises:

- The controller function returning a single composition's result directly (no extra closure-scoped vars needed)
- `useForm` composition bundling state (values, errors, pending, dirty), computed values (errors per field), and handlers (change, submit, reset) into one object
- `bind:value` two-way binding on a `<select>` and an `<input>`
- `on:submit` on the form element

What this avoids: hand-rolled form state with five signals and four handlers. The pattern lives in the composition.

## What these examples establish

| Decision | Where it lives |
|---|---|
| Server data | `<data>` in the template |
| Local state (any) | `signal(...)` returned from the controller function |
| Computed-initial state (`crypto.randomUUID()`, etc.) | Placeholder signal + assignment inside `mount` |
| Computed values | `derived(...)` (named) or inline expressions (one-off) |
| Event handlers | Plain functions returned from the controller |
| UI pattern bundles | A composition called inside the controller function |
| Host primitives | Template hook (`<use-*>`) |
| Lifecycle | Ambient `mount(...)` / `onCleanup(...)` |

Three examples, three tiers, one controller function, both composition and template-hook categories, every binding form. If a new view doesn't fit one of these shapes, it is *almost certainly* combinable from these primitives. If it isn't, escalate to a discipline conversation — see [the discipline](index.md#the-discipline).
