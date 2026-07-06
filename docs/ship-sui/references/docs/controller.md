---
group: Views
order: 3
title: The controller surface
description: The optional `.ts` paired with a template. Three tiers, six sections, scope rules, mutation discipline.
---

# The controller surface

A controller is an **optional** TypeScript file paired with a template by directory. `views/agents/page.sui` ↔ `views/agents/controller.ts`. No config, no import statement in the template, no manifest entry — convention is the wiring.

The controller is the view's view-model: reactive state, derived values, event handlers, lifecycle, and composable hook bundles. Templates declare structure; controllers own everything reactive.

## Three tiers

Graduated complexity. Use the smallest tier that fits.

### Tier 1 — no controller

Pure declarative views. The template uses `<data>` for subscriptions and inline expressions for anything computed. **No reactive state** — the moment you need a local signal, graduate to tier 2 and add a paired controller.

```html
<!-- views/static/page.sui -->
<data name="overview" source="ship.git.overview()" live />

<p>Branch: {overview().value?.branch}</p>
```

No `controller.ts`. Most one-shot read-only views land here.

### Tier 2 — object-literal controller

For views that need named handlers, derived values, or local reactive state:

```ts
// views/settings/controller.ts
import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => ({
    theme: signal<'light' | 'dark' | 'system'>('system'),
    accent: signal('blue'),
  }),
  on: {
    setTheme:  (s, v: 'light' | 'dark' | 'system') => { s.theme = v },
    setAccent: (s, v: string)                      => { s.accent = v },
  },
})
```

Plain object. Static analysis is trivial — every section is data. Sections can also be functions when they need scope:

```ts
export default controller<Scope>({
  scope: () => ({ query: signal('') }),
  derived: {
    filtered: ({ agents, query }) =>
      agents().value?.items.filter(a => a.name.includes(query())) ?? [],
  },
  on: {
    setQuery: (s, q: string) => { s.query = q },
    refresh:  ({ ship })     => ship.agents.refresh(),
  },
})
```

Per-section functions are how the controller pulls from the [shared view scope](index.md#the-shared-view-scope). The runtime calls each with `{ ship, ...templateScope, ...derivedScope }`; sections destructure what they need.

### Typing the template scope (one declaration, every section gets it)

The cheapest way to avoid annotating every section's destructure: declare the template-side identifiers once and pass them as the `controller<Scope>` generic. Sections inherit the type:

```ts
import type { LiveSignal, Signal } from '@ship/sui-runtime'
import type { GitOverview } from '@ship/sdk-client'
import type { Scope } from './page.sui'   // compiler-emitted .sui.d.ts

export default controller<Scope>({
  scope: () => ({ tab: signal<'changes' | 'history' | 'branches'>('changes') }),
  derived: {
    branch:  ({ overview }) => /* overview is typed automatically */,
    clean:   ({ overview }) => /* same */,
    // ...15 more, none with per-section annotations
  },
  on: {
    showHistory: (s) => { s.tab = 'history' },  // s typed as MutableState<Scope>
  },
})
```

`MutableState<Scope>` is computed from `Scope`: it picks every key whose value is a `Signal<T>` and exposes that key as an assignable `T`. The signals returned from the controller's `scope:` factory plus the template's `<data>` / `<prop>` / `<use-* as=>` declarations are the single source of truth.

The compiler emits a `<file>.sui.d.ts` declaration alongside each compiled template, exporting a `Scope` interface derived from the template's `<data>` / `<prop>` / `<use-* as=>` elements. The signals returned from `scope:` are typed by TS itself. Import the generated `Scope` as the controller generic to replace the manual `type Scope = { … }` declaration.

### Tier 3 — function controller

For views that need to close over instantiation-time variables, share computed references across sections, or compose multiple controller hooks:

```ts
// views/agents/controller.ts
import { controller, signal, useFilteredList, useSelection } from '@ship/sui-runtime'

export default controller(({ ship }) => ({
  scope: () => ({ searchOpen: signal(false) }),
  use: {
    list: useFilteredList(/* see hooks.md */),
    sel:  useSelection(/* see hooks.md */),
  },
  on: {
    toggleSearch: (s) => { s.searchOpen = !s.searchOpen },
    refresh:      ()  => ship.agents.refresh(),
  },
}))
```

The outer function receives `{ ship }` (and any other instantiation-time context) and returns the controller object. Sections still destructure their own scope as in tier 2.

## The six sections

| Section | Type | Purpose |
|---|---|---|
| `scope` | `(init, host?) => Record<string, unknown>` | Factory that contributes signals, derived, hooks, and shared providers/consumers to template scope. Runs first. |
| `derived` | record of `({ scope }) => value` | Pure memos. Auto-tracked dependencies. |
| `on` | record of `(state, ...args) => void` or `({ scope }, ...args) => void` | Event handlers. The **only** section that mutates state. |
| `use` | record of hook results, OR `({ scope }) => ({ name: hookResult })` | Composition of controller hooks. Each result is namespaced by key. |
| `mount` | `(state, el) => void` | Runs once after the DOM is in place. For wiring third-party libs, observers, computed-initial state assignments, or any one-shot imperative setup. |
| `unmount` | `(state) => void` | Runs on disconnect. Tear down whatever `mount` set up. |

Reactive state is declared inside `scope:` — return signals (and any derived/hooks/shared providers) from the factory and the runtime merges them into template scope.

Computed-initial cases (`crypto.randomUUID()`, an SDK call, a derivation over `route`) seed a placeholder in `scope:` and assign in `mount`:

```ts
export default controller({
  scope: () => ({ id: signal('') }),
  mount: ({ state }) => {
    state.id = crypto.randomUUID()
  },
})
```

No other sections exist. No `effect`, no `watch`, no `computed`, no `data`, no `live`, no `task`. If your case doesn't fit one of the six, see [the discipline](index.md#the-discipline) — the answer is almost always a combination of the existing sections, not a new section.

## Scope rules

Each section is invoked with the [shared view scope](index.md#the-shared-view-scope) as its argument. The shape of that argument differs slightly by section:

| Section | Argument shape | Mutation |
|---|---|---|
| `scope` | `(init, host?)` | **factory** — its return value is merged into scope before any other section runs |
| `derived` | `{ ship, ...allSignals }` | **read-only** state proxy |
| `on` (handler) | `(stateMutator, ...args)` OR `({ ship, ...allSignals }, ...args)` | **writable** state proxy in the first arg |
| `use` | `{ ship, ...allSignals }` | hooks are factories — see [hooks.md](hooks.md) |
| `mount` / `unmount` | `(stateMutator, el?)` | writable state proxy |

The mutation rule is a static guarantee: in `derived` and templates, every signal key is callable (`count()`); in `on` / `mount` / `unmount`, the same key is assignable (`s.count = …`). **You cannot mutate state from a derived or a template.** Every state change has a name (the `on.*` key, or the lifecycle hook) and a single call site.

## What's NOT in the controller

For clarity — these go in the template, never the controller:

- Server data subscriptions (use `<data>` in the template).
- Host-side side effects like keybindings, document title, focus traps (use `<use-*>` template hooks).
- DOM structure (templates own structure, full stop).

If you find yourself wanting to construct DOM in `mount`, you almost certainly want a template hook or a child component instead.

## File layout

Most common shape:

```
views/<name>/
  page.sui              # template (required)
  controller.ts         # controller (optional)
  style.css             # styles (optional)
  manifest.jsonc        # route descriptor (required for views)
```

The compiler emits `<file>.sui.d.ts` into the `.ship/sui-out/` mirror tree (not next to the source). Component files use the same directory-pairing convention. The file IS the component; the directory name IS the tag; `<prop>` directives at the top declare the element's API:

```
packages/components/ui-components/src/sui-button/
  sui-button.sui            # template + <prop> directives
  sui-button.controller.ts  # controller(def) (only if needed)
  sui-button.css            # extracted styles (optional)
```

The compiler pairs by directory; no manifest, no metadata file for components. Adding a `<tag>.controller.ts` next to a `<tag>.sui` is the only signal the framework requires.

## Tier-1 / tier-2 threshold rule

The compiler enforces condition (1) of the threshold rule statically. A **multi-statement** `on:event` handler body in a tier-1 template (no paired `controller.ts`) is a build-time error:

```html
<!-- ✗ fails without controller.ts -->
<button on:click={() => { x.set(1); y.set(2) }}>Save</button>
```

```
page.sui: line 1 col 30: [on:click handler]
multi-statement handler — paired `<view>.ts` controller required for tier-2 work
```

A single-expression or single-statement block is fine at tier 1:

```html
<!-- ✓ OK — single expression -->
<button on:click={() => count.set(count() + 1)}>+</button>

<!-- ✓ OK — single-statement block -->
<button on:click={() => { count.set(count() + 1) }}>+</button>
```

Once you add a paired `controller.ts` the check is skipped entirely — the controller owns complex handler logic. Conditions 2–5 of the threshold rule (multi-signal data thunks, `use:` compositions, lifecycle hooks, multi-statement state mutations) are not yet enforced statically but follow the same architectural boundary.
