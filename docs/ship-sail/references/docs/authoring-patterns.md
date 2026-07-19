---
group: Routes
order: 3
title: SAIL — Authoring Patterns
description: Canonical patterns for .sail route authoring.
---

# SAIL — Authoring Patterns

How to author a SAIL route, a SAIL component, and the controller that lives inside each.

## Component vs route

| Trait | Route | Component |
|---|---|---|
| Filename stem | `page.sail` (manifest-keyed route) | `<tag>.sail` (registers a custom element) |
| Lives under | `views/<name>/`, `.ship/views/<name>/` | `packages/components/ui-components/src/<tag>/`, `_components/<view>-<tag>/` |
| Has `manifest.jsonc`? | Yes — required | Yes — anatomy regions / containment declaration live here |
| Has `<prop>` directives? | No | Yes — declares the public attribute / property contract |
| Has projection regions? | No | Yes — declared in the anatomy manifest; runtime projects matching children at connect. |
| Has a shadow root? | No | **Only** for components that wrap foreign content (tiptap, third-party widgets, FFI boundary); declared in the manifest, paid for knowingly. |
| Has scoped CSS? | `style.css` (auto-linked, page-scoped) | Component rules compile to `@scope (sail-x) to ([data-sail])` donuts. A skin restyles an existing anatomy; a skin cannot add regions or states. |
| Form participation? | Native (the element is in the light DOM) | Native (the host is in the light DOM; projected inputs are real descendants). |
| `customElements.define`? | n/a | Compiler-emitted; never written by hand |
| Used by | Other routes, the shell, deep links from `<a>` | Composed inside routes or other components |

A route that wants reusable markup pulls a component in (`packages/components/ui-components/` for first-party, `_components/` for route-private). The reverse is illegal — components don't mount routes.

> **Anatomy manifest shape** (shipped): the compiler emits an `anatomy`
> block on each declaration in the standard `custom-elements.json` —
> `{ "dom": "light" | "shadow", "classes": [...], "slots": [...] }`.
> Classes are the author class names from the template
> (compile-generated `sx-*`/`sail-*` atomics excluded); slots list the
> projection points (`(default)` for the unnamed slot); states ride the
> standard `attributes`/`members` entries as `reflects: true`.
>
> **`[ROADMAP: shadow-bounded-component-list]`**. The
> components that legitimately need a shadow boundary are the
> ones that wrap foreign content. Today's stdlib does not name a
> single one — the tiptap wrapper, the embedded third-party
> widgets, and the FFI boundary are placeholders for things that
> have not yet landed. Until a real component needs a shadow
> boundary, the manifest's "containment" field is unset and the
> compiler never emits a shadow root.

## The controller block

One inline `<controller>` block, authored directly in the `.sail` file — no paired sibling file. The template still declares markup and bindings; the controller still does the work, it's just one function now instead of a sectioned object.

```ts
// inside page.sail
<controller>
export default controller((ctx) => {
  const { agents, navigate } = ctx
  const filter   = signal('')
  const total    = derived(() => unwrap(agents)?.items.length ?? 0)
  const list     = useFilteredList(agents, (a, q) => a.name.includes(q))
  const dialogEl = ref<HTMLDivElement>(null)
  const badge    = derived(() => `${total()} agents`)

  const newAgent = () => navigate('/agents/create')

  mount((el) => { /* runs after first DOM attach; return a cleanup fn or nothing */ })
  onCleanup(() => { /* runs on dispose */ })

  return { filter, total, list, dialogEl, badge, newAgent }
})
</controller>
```

One function, one job: `controller(fn)` where `fn` is `(ctx) => Record<string, unknown>`. `ctx` is the ambient route scope — `ship`, `clients`, `route`, `navigate`, directly accessible. Whatever the function returns IS the contributed template scope: state signals, derived values (including `query()`/`queryKeyed()` data signals), plain helpers, and event handlers all live together as ordinary object properties — there's no separate section for handlers anymore.

`mount(fn)` and `onCleanup(fn)` are ambient — ✅ callable from anywhere in the setup call graph (the controller body itself, or a plain helper function it calls), no import needed. `mount` fires once after the instance's DOM connects; `onCleanup` runs on dispose. Mutation is always `signal.set(newValue)` (or the updater form `signal.set(prev => next)`) — there's no writable-state-proxy assignment style.

A multi-statement `on:event` in a template-only file (no controller) is a compile error — that's the signal you've outgrown tier-1.

Single-expression handlers (`on:click={() => count.set(count() + 1)}`) live in the template; multi-statement handlers become a plain function returned from the controller and referenced by name (`on:click={newAgent}`).

## Templates declare. Controllers do.

The `sail-controller-purity` rule rejects markup-producing code inside the `<controller>` block:

- `<el>.innerHTML = …` / `outerHTML = …` / `insertAdjacentHTML(…)`
- A sibling `render.ts` of HTML-string template-literal helpers
- Re-querying the host's children after `mount` for what a
  `bind:this` would have given you (use `bind:this` for refs)
- A `.sail` body that is just `<div bind:this={ref}></div>` — the template is the renderer

For HTML-blob cases (markdown rendering, third-party SVG generators, icon-pack swaps), use `bindHtml(el, () => string)` from `@ship/sail-runtime`. It auto-morphs across re-renders.

> **The host element IS the component.** A `bind:this` on the
> component's own root gives the controller a handle to the same
> element the consumer sees in the light DOM. No "query the shadow
> DOM" workaround.

## `<prop>` directive — components only

```html
<prop name="value"    type="string" default="" />
<prop name="tone"     values="default|primary|danger" default="default" reflect />
<prop name="disabled" type="bool"   default="false" reflect />
<prop name="rows"     type="json"   default="[]" />
```

| Attribute | Meaning |
|---|---|
| `name` | Public prop / attribute name (kebab-case). Camel-case mirror auto-generated for the property accessor. |
| `type` | `string` \| `bool` \| `number` \| `json`. Pick the decoder in `decode.ts`. |
| `values` | Enum string — pipe-separated literals. Mutually exclusive with `type`. |
| `default` | Coerced via the decoder; required for non-string types. |
| `reflect` | Reflect property writes back to the attribute. Required for CSS to react. |

> **Form participation — mostly free in light DOM.** The
> `form-value` and `form-disabled` flags still parse and still
> wire `ElementInternals.setFormValue` / `formDisabledCallback`
> (`crates/sail/compiler/src/ir/directive.rs`,
> `packages/sail/runtime/src/component-factory.ts`), so a
> component that needs explicit form association can still
> declare it. The light-DOM default means most components
> don't: a `<sail-input>` descendant of a host `<form>`
> participates natively without any prop flag, because it is a
> real child of the form in the same DOM tree. ARCHITECTURE.md
> calls this "the ElementInternals plumbing mostly dies" — not
> "is removed".
>
> **`[ROADMAP: form-participation-surface]`**. The L1 reset
> covers the common cases (text input, textarea, select, button).
> Today's stdlib components (`<sail-input>`, `<sail-textarea>`,
> `<sail-select>`, `<sail-checkbox>`, `<sail-switch>`) still declare
> `form-value` / `form-disabled` on their value/disabled props;
> whether the port keeps those declarations or drops them
> (relying on native form participation) is the design call.
> The flag is not removed from the compiler; the question is
> which stdlib components still need it.

The prop table is otherwise unchanged.

## Refs — `bind:this`

```ts
// inside the controller function
const dialogEl = ref<HTMLDivElement>(null)
```

```html
<div bind:this={dialogEl}>…</div>
```

`ref<T>(null)` constructs a `Signal<T | null>` slot the runtime writes when the element mounts and clears on unmount. The `bind_this_target` compiler check requires every `bind:this` target to be declared as `ref(null)` in scope. Authors never call `.set()` on a ref themselves — read with `dialogEl()`.

`<use-click-outside bind:target={dialogEl} …>` and the other template hooks consume refs directly.

> **What changed.** The host element is the component. A
> `bind:this` on the component's own root gives the controller a
> handle to the same element the consumer sees in the light DOM.
> No "query the shadow DOM" workaround.

## Passing complex values into components

| Form | Use when |
|---|---|
| `<sail-toast prop:variant={'success'}>` | Static string literal — daemon sees it in SSR. |
| `<sail-toast variant="success">` | Static string literal — sugar for the above; identical. |
| `<sail-grid prop:rows={rows}>` | Pass a signal-or-object identity. The component receives the live reference. |
| `<sail-grid prop:columns={columns()}>` | Pass a snapshot — re-evaluates on dep change. |
| `<sail-grid prop:total={count() + extra()}>` | Computed value — re-runs as a derived. |

`prop:*` lets the runtime pass structured data through — it skips attribute serialization, which can't represent arrays / objects / signals. Bare-attribute syntax (`variant="success"`) is for string literals.

## Cross-component reactive context — `shared` / `useShared`

For component packs that split into a public outer host + internal pieces (e.g. `sail-tabs` / `sail-tab`), share signals from parent to descendants without prop drilling. The parent calls `shared(signal(…))` inside the controller function; descendants call `useShared<ParentScope>('parent-tag')` to resolve the same identity. `useShared` walks DOM ancestry in the light DOM; returns `null` when no provider is found.

Worked examples and lookup semantics are in the `ship-views` skill's `references/shared-context.md`.

## Scope is typed

The compiler emits `Scope` in `<file>.sail.d.ts` covering BOTH template reactive identifiers AND the keys the controller function returns. `mount((el) => { … })` sees direct, typed members via the closure over the controller's returned scope:

```ts
mount((el) => {
  caughtError.set(null)  // Signal<Error | null>
  reset()                 // () => void
})
```

No casts. No "scope signals missing" defensive throws. If a key is mistyped or missing, the TypeScript compiler catches it before the runtime does.

## Scaffolds

```sh
deck scaffold view settings
deck scaffold view settings --component settings-toolbar
deck scaffold component my-widget --controller
```

The first generates `views/settings/` with `page.sail` (including an inline `<controller>` block) + `style.css` + `manifest.jsonc`. The `--component` flag also scaffolds `_components/settings-toolbar/`. The component scaffold drops `packages/components/ui-components/src/my-widget/` with `<tag>.sail`; `--controller` adds the inline `<controller>` block to it.

## User-app routes — authoring outside the Ship repo

Built-in routes live under `<repo-root>/views/<name>/`. You can author the same kind of route inside *your own* workspace — the daemon doesn't require Ship's repo layout to discover and serve them.

Two layouts are honored, both rooted at the workspace `shipd --workspace <path>` was launched with:

```
~/my-project/
├── views/
│   └── my-route/
│       ├── page.sail            # required (controller, if any, is an inline <controller> block inside)
│       ├── style.css           # optional — route-scoped CSS
│       └── manifest.jsonc      # optional — rail label/icon, route type
└── .ship/
    └── views/
        └── another-route/
            ├── index.sail                 # required (note the stem: index, not page)
            └── manifest.jsonc            # optional
```

- **`<workspace>/views/<name>/page.sail`** — same convention as in-tree routes. Use this when the route *is* the app surface (e.g. you're building a Ship-shaped tool).
- **`<workspace>/.ship/views/<name>/index.sail`** — colocates the route with the rest of your `.ship/` config. Use this for routes that are auxiliary to a non-Ship project.

A controller is always authored inline as a `<controller>` block inside the `.sail` file — a paired `<stem>.controller.ts` sibling file is a compile error.

When both layouts contain a route with the same name, the top-level `views/` directory wins — that's how a workspace can shadow a built-in by id without renaming.

### Rail registration

Drop a `manifest.jsonc` next to the leaf source to opt the route into the launcher / rail surface:

```jsonc
{
  "type": "html",
  "rail": {
    "label": "My Route",
    "icon": "rocket"
  }
}
```

`/api/v1/views` will include the route with its rail metadata, and any shell that renders the rail dynamically will pick it up. (Shell2's rail is currently hardcoded; dynamic rendering ships in the next iteration.)

### Running the daemon against your workspace

```sh
shipd --workspace ~/my-project
```

The daemon persists this workspace into `~/.ship/machine/daemons/<hash>/workspace`. The resolver reads from there, so all `/sail/<name>`, `/_deck/views/<name>/payload`, and `/_deck/views/<name>/view.js` requests are scoped to your workspace.

## Hot reload in dev

`.sail` edits (including changes inside the inline `<controller>` block) trigger an in-place route re-mount without a full page reload. `apps/shell/src/hmr.ts` subscribes to `shipd`'s `/_deck/dev/hmr` SSE channel. When the bundler emits `module-changed` for `views/<name>/view`, shell cache-busts the rebuilt `view.js` and re-mounts through `mount()`. Other bundles (sail-components, islands, sail-runtime, sail-dom, sail-components) trigger a full reload — custom-element re-registration without reload is not yet supported.

Production builds inline `import.meta.env.DEV` to `false`, so HMR is a no-op and no EventSource opens.

Run `just dev` and edit any file under `views/<name>/`.
