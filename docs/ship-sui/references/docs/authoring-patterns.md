---
group: Views
order: 3
title: SUI — Authoring Patterns
description: Canonical patterns for .sui view authoring.
---

# SUI — Authoring Patterns

How to author a SUI view, a SUI component, and the controller that pairs with each.

## Component vs view

| Trait | View | Component |
|---|---|---|
| Filename stem | `page.sui` (manifest-keyed route) | `<tag>.sui` (registers a custom element) |
| Lives under | `views/<name>/`, `.ship/views/<name>/` | `packages/components/ui-components/src/<tag>/`, `_components/<view>-<tag>/` |
| Has `manifest.jsonc`? | Yes — required | No |
| Has `<prop>` directives? | No | Yes — declares the public attribute / property contract |
| Has slots? | No | Yes — `<slot>` / `<slot name="…">` |
| Has shadow DOM? | No — composes into the page | Optional; toggled via `component_meta.shadow` in the compile opts |
| Has scoped CSS? | `style.css` (auto-linked, page-scoped) | `<style>` block inside `.sui` (or external CSS prepended via `compile_opts.styles`) |
| Form-associated? | No | Optional — `<prop … form-value>` / `form-disabled` |
| `customElements.define`? | n/a | Compiler-emitted; never written by hand |
| Used by | Other views, the shell, deep links from `<a>` | Composed inside views or other components |

A view that wants reusable markup pulls a component in (`packages/components/ui-components/` for first-party, `_components/` for view-private). The reverse is illegal — components don't mount routes.

## The 2-file bet

Two files, paired by directory + filename stem. The compiler reads both — the `.sui` template emits the render module, the `.controller.ts` types its `Scope` and contributes signals / lifecycle / handlers.

```ts
// controller.ts
import { controller, signal, derived, ref, shared, useShared,
         useFilteredList, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'   // compiler-emitted .d.ts

export default controller<Scope>({
  scope: ({ agents }) => {
    const filter   = signal('')
    const total    = derived(() => unwrap(agents)?.items.length ?? 0)
    const list     = useFilteredList(agents, (a, q) => a.name.includes(q))
    const dialogEl = ref<HTMLDivElement>(null)
    return { filter, total, list, dialogEl }
  },
  derived: {                         // optional separate block — same effect as in scope
    badge: ({ total }) => `${total()} agents`,
  },
  on: {
    newAgent: ({ navigate }) => navigate('/agents/create'),
  },
  mount:   ({ state }, el) => { /* runs after first DOM attach */ },
  unmount: ({ state })     => { /* runs on dispose */ },
})
```

Four sections, each one job:

- **`scope:`** — factory that returns the typed bag of signals, derived, refs, hooks, shared providers / consumers. Runs first; entries land in template scope. Replaces the historical `<state>` / `<derived>` / `<use>` split.
- **`derived:`** — separate block for computed values, same lifetime as scope. Convention; not required.
- **`on:`** — multi-statement event handlers. The first arg is a writable proxy: `s.count = s.count + 1` mutates state.
- **`mount:` / `unmount:`** — lifecycle. Runs after first attach / on disconnect.

A multi-statement `on:event` in a template-only file (no controller) is a compile error — that's the signal you've outgrown tier-1.

Single-expression handlers (`on:click={() => count.set(count() + 1)}`) live in the template; multi-statement handlers move to the controller's `on:`.

## Templates declare. Controllers do.

The `sui-controller-purity` rule rejects markup-producing code in `*.controller.ts`:

- `<el>.innerHTML = …` / `outerHTML = …` / `insertAdjacentHTML(…)`
- A sibling `render.ts` of HTML-string template-literal helpers
- Re-querying the shadow DOM after `mount` (use `bind:this` for refs)
- A `.sui` body that is just `<div bind:this={ref}></div>` — the template is the renderer

For HTML-blob cases (markdown rendering, third-party SVG generators, icon-pack swaps), use `bindHtml(el, () => string)` from `@ship/sui-runtime`. It auto-morphs across re-renders.

## `<prop>` directive — components only

```html
<prop name="value"    type="string" default="" form-value />
<prop name="tone"     values="default|primary|danger" default="default" reflect />
<prop name="disabled" type="bool"   default="false" reflect form-disabled />
<prop name="rows"     type="json"   default="[]" />
```

| Attribute | Meaning |
|---|---|
| `name` | Public prop / attribute name (kebab-case). Camel-case mirror auto-generated for the property accessor. |
| `type` | `string` \| `bool` \| `number` \| `json`. Pick the decoder in `decode.ts`. |
| `values` | Enum string — pipe-separated literals. Mutually exclusive with `type`. |
| `default` | Coerced via the decoder; required for non-string types. |
| `reflect` | Reflect property writes back to the attribute. Required for CSS to react. |
| `form-value` | Mark this prop as the form-association value. Compiler emits `static formAssociated = true`. |
| `form-disabled` | Wires `formDisabledCallback` to this prop. |

Form-association example: `packages/components/ui-components/src/sui-input/sui-input.sui` (real `<prop name="value" type="string" default="" form-value />` line). Full contract: see the `ship-views` skill's `references/form-association.md`.

## Refs — `bind:this`

```ts
// in scope: factory
const dialogEl = ref<HTMLDivElement>(null)
```

```html
<div bind:this={dialogEl}>…</div>
```

`ref<T>(null)` constructs a `Signal<T | null>` slot the runtime writes when the element mounts and clears on unmount. The `bind_this_target` compiler check requires every `bind:this` target to be declared as `ref(null)` in scope. Authors never call `.set()` on a ref themselves — read with `dialogEl()`.

`<use-click-outside bind:target={dialogEl} …>` and the other template hooks consume refs directly.

## Passing complex values into components

| Form | Use when |
|---|---|
| `<sui-toast prop:variant={'success'}>` | Static string literal — daemon sees it in SSR. |
| `<sui-toast variant="success">` | Static string literal — sugar for the above; identical. |
| `<sui-grid prop:rows={rows}>` | Pass a signal-or-object identity. The component receives the live reference. |
| `<sui-grid prop:columns={columns()}>` | Pass a snapshot — re-evaluates on dep change. |
| `<sui-grid prop:total={count() + extra()}>` | Computed value — re-runs as a derived. |

`prop:*` lets the runtime pass structured data through — it skips attribute serialization, which can't represent arrays / objects / signals. Bare-attribute syntax (`variant="success"`) is for string literals.

## Cross-component reactive context — `shared` / `useShared`

For component packs that split into a public outer host + internal pieces (e.g. `sui-tabs` / `sui-tab`), share signals from parent to descendants without prop drilling. The parent calls `shared(signal(…))` inside `scope:`; descendants call `useShared<ParentScope>('parent-tag')` to resolve the same identity. `useShared` walks DOM ancestry across shadow boundaries; returns `null` when no provider is found.

Worked examples and lookup semantics are in the `ship-views` skill's `references/shared-context.md`.

## Scope is typed

The compiler emits `Scope` in `<file>.sui.d.ts` covering BOTH template reactive identifiers AND the keys you add via `scope: () => ({...})` factory. `mount(state, host)` sees direct, typed members:

```ts
mount(state, host) {
  state.caughtError.set(null)  // Signal<Error | null>
  state.reset()                 // () => void
}
```

No casts. No "scope signals missing" defensive throws. If a key is mistyped or missing, the TypeScript compiler catches it before the runtime does.

## Scaffolds

```sh
deck scaffold view settings
deck scaffold view settings --component settings-toolbar
deck scaffold component my-widget --controller
```

The first generates `views/settings/` with `page.sui` + `controller.ts` + `style.css` + `manifest.jsonc`. The `--component` flag also scaffolds `_components/settings-toolbar/`. The component scaffold drops `packages/components/ui-components/src/my-widget/` with `<tag>.sui`; `--controller` adds the paired controller.

## User-app views — authoring outside the Ship repo

Built-in views live under `<repo-root>/views/<name>/`. You can author the same kind of view inside *your own* workspace — the daemon doesn't require Ship's repo layout to discover and serve them.

Two layouts are honored, both rooted at the workspace `shipd --workspace <path>` was launched with:

```
~/my-project/
├── views/
│   └── my-view/
│       ├── page.sui            # required
│       ├── page.controller.ts  # optional — paired controller (stem must match: page.sui ↔ page.controller.ts)
│       ├── style.css           # optional — view-scoped CSS
│       └── manifest.jsonc      # optional — rail label/icon, view type
└── .ship/
    └── views/
        └── another-view/
            ├── index.sui                 # required (note the stem: index, not page)
            ├── index.controller.ts       # optional — paired controller
            └── manifest.jsonc            # optional
```

- **`<workspace>/views/<name>/page.sui`** — same convention as in-tree views. Use this when the view *is* the app surface (e.g. you're building a Ship-shaped tool).
- **`<workspace>/.ship/views/<name>/index.sui`** — colocates the view with the rest of your `.ship/` config. Use this for views that are auxiliary to a non-Ship project.

The controller filename stem must match the `.sui` stem: `page.sui` ↔ `page.controller.ts`, `index.sui` ↔ `index.controller.ts`. The compiler infers the controller path from the resolved source.

When both layouts contain a view with the same name, the top-level `views/` directory wins — that's how a workspace can shadow a built-in by id without renaming.

### Rail registration

Drop a `manifest.jsonc` next to the leaf source to opt the view into the launcher / rail surface:

```jsonc
{
  "type": "html",
  "rail": {
    "label": "My View",
    "icon": "rocket"
  }
}
```

`/api/v1/views` will include the view with its rail metadata, and any shell that renders the rail dynamically will pick it up. (Shell2's rail is currently hardcoded; dynamic rendering ships in the next iteration.)

### Running the daemon against your workspace

```sh
shipd --workspace ~/my-project
```

The daemon persists this workspace into `~/.ship/machine/daemons/<hash>/workspace`. The resolver reads from there, so all `/sui/<name>`, `/_deck/views/<name>/payload`, and `/_deck/views/<name>/view.js` requests are scoped to your workspace.

## Hot reload in dev

`.sui` and `.controller.ts` edits trigger an in-place view re-mount without a full page reload. `apps/shell/src/hmr.ts` subscribes to `shipd`'s `/_deck/dev/hmr` SSE channel. When the bundler emits `module-changed` for `views/<name>/view`, shell cache-busts the rebuilt `view.js` and re-mounts through `mount()`. Other bundles (sui-components, islands, sui-runtime, sui-dom, sui-components) trigger a full reload — custom-element re-registration without reload is not yet supported.

Production builds inline `import.meta.env.DEV` to `false`, so HMR is a no-op and no EventSource opens.

Run `just dev` and edit any file under `views/<name>/`.
