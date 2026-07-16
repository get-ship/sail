---
group: Views
order: 5
title: SUI — Runtime Primitives
description: The signal/bind/effect primitives the SUI runtime exposes.
---

# SUI — Runtime Primitives

All symbols below are re-exported from `packages/sui/runtime/src/index.ts`. Anything not in that barrel is internal.

## Reactivity

```ts
import { signal, derived, effect, batch, untrack } from '@ship/sui-runtime'

const count   = signal(0)
const doubled = derived(() => count() * 2)

effect(() => console.log(doubled()))     // re-runs on tracked-dep change
batch(() => { count.set(1); count.set(2) })  // flushes once
untrack(() => count())                    // read without subscribing
```

`signal<T>(initial)` returns `Signal<T>`: call to read, `.set(v)` / `.update(fn)` to write. `derived<T>(fn)` returns `Derived<T>` (read-only). `effect(fn)` re-runs on dep change; `batch(fn)` coalesces writes; `untrack(fn)` reads without recording.

Scopes (`createScope`, `createChildScope`, `withScope`, `onCleanup`, `currentScope`) own a tree of disposers — child scopes dispose when their parent does. The dev-only leak detector lives at `@ship/sui-runtime/dev` (separate subpath so it doesn't ship in production).

## DOM bindings

Used by compiler-emitted code; rarely written by hand.

| Symbol | Wires |
|---|---|
| `bindText(node, fn)` | Reactive text node |
| `bindShow(el, fn)` | Toggle `display: none` |
| `bindAttr(el, name, fn)` | Reactive attribute |
| `bindClass(el, name, fn)` | Reactive class toggle |
| `bindProp(el, name, fn)` | Reactive JS property |
| `bindValue(el, signal)` | Two-way `<input>` value |
| `bindChecked(el, signal)` | Two-way `<input type="checkbox">` |
| `bindHtml(el, fn)` | HTML-string content, morphed across re-renders |
| `bindRawHtml(el, fn)` | HTML-string content via `innerHTML`; morph only if `morph` directive is on the element |

`on(el, event, handler, opts)` + `matchesKey(ev, 'cmd+k')` wire DOM events. `OnOptions` accepts `once`, `passive`, `capture`, `preventDefault`, `stopPropagation`, and a `keys: 'cmd+k|escape'` shortcut filter.

## Control flow — block syntax in templates

```html
{#if loading()}…{:else if error()}…{:else}…{/if}
{#for item of items() key={item.id}}…{/for}
{#match status()}{:case 'idle'}…{:case 'ok'}…{:default}…{/match}
{#while-loading of agents}<p>Loading…</p>{/while-loading}
{#while-error err of agents}<p>{err.message}</p>{/while-error}
```

Runtime helpers behind these: `bindIf`, `bindFor`, `bindForSignal`, `bindWhileLoading`, `bindWhileError`. The `<sui-if>`, `<sui-for>`, `<sui-match>` custom elements register on module load.

`{#for}` requires `key={EXPR}` — without a stable key the reconciler falls back to positional matching and DOM identity (focus, selection, in-progress edits) is lost on reorder. `sui-for-key` lint warns when missing.

## `<data>` — fetch + subscribe

```html
<!-- one-shot fetch -->
<data name="settings" source="ship.settings.state()" />

<!-- live subscription via SSE -->
<data name="agents" source="ship.agents.summaries()" live />

<!-- reactive args — re-subscribes when selectedId changes -->
<data name="agent" source="ship.agents.detail({id: $scope.selectedId()})" live />
```

The expression must resolve to a `LiveSource<T>` (anything with `.subscribe(sig)`) — every SDK method in `packages/sdk-client/src/index.ts` returns this shape,

```ts
type LiveState<T> = { kind: 'loading' }
                  | { kind: 'error'; error: Error }
                  | { kind: 'ok';    value: T }

unwrap(sig)              // T | undefined
unwrapOr(sig, fallback)  // T
```

## Live SSE — end-to-end loop

```
.sui  <data live source="ship.git.overview()" live />
  ↓ sui-compiler
.js   $scope.git = live(() => ship.git.overview())
  ↓ @ship/sdk-client createClient()
HTTP  GET /_deck/data?expr=ship.git.overview()    [text/event-stream]
  ↓ apps/shipd/src/views/sui_data.rs
Rust  registry::call("git.overview", {}) → initial; subscribe `config` ns
  ↓ kernel event → re-resolve
SSE   event: patch \n data: {"op":"set","value":{…}}
  ↓ makeSseSource() in @ship/sdk-client
sig   LiveSignal<T> flips loading → ok → ok (per patch)
  ↓ bindWhileLoading / bindWhileError / bindIf
DOM   morphs in place — no re-render of the surrounding view
```

**Worked example.** `views/home/` ships the canonical demo: `<data live>` subscribes to `ship.git.overview()`; the daemon's git-signal singleton refreshes on `.ship/` filesystem events and on git ops, so staging a file flips the badge live.

```bash
just dev-restart                                # rebuilds shipd + restarts the daemon
PORT=$(cat $(./target/debug/ship daemon dir)/port)

# 1. Confirm the SDK channel is live (Ctrl-C after a few `patch` events).
curl -N "http://localhost:$PORT/_deck/data?expr=$(printf 'ship.git.overview()' | jq -sRr @uri)"

# 2. Open the view, stage a file, watch the badge flip.
xdg-open "http://localhost:$PORT/"
touch CHANGE_ME && git add CHANGE_ME            # status flips to "1 changed"
git restore --staged CHANGE_ME && rm CHANGE_ME  # back to "clean"
```

**Adding a new live source** needs three things:

1. **Registry entry** — `apps/shipd/src/services/registry/<domain>.rs` returns the current value synchronously. Dispatcher in `registry.rs` + schema row in `services/schema.rs` keep the catalog honest.
2. SDK method — `packages/sdk-client/src/index.ts` exposes the method returning the SSE source.
3. **(Optional) signal-backed source** — if the data has a tighter wake-up surface than the default `config` kernel namespace, mirror `apps/shipd/src/views/git_signal.rs`: `Signal<Value>` plus a singleton background task that updates it. `sui_sse.rs::is_signal_backed_expr` picks it up by registry name.

## Mutations — `<form mutation>` and `wireFormMutation`

```html
<form mutation="ship.workspaces.activate">
  <input name="id" />
  <button type="submit" disabled={$mutation().kind === 'loading'}>Switch</button>
</form>
```

The compiler wires the form to `wireFormMutation` against the named SDK call. The handler reads form fields, invokes the mutation, and exposes a `$mutation` signal of `{ kind: 'idle' | 'loading' | 'error' | 'ok' }` to the template. POST/DELETE routes are served by `apps/shipd/src/views/sui_mutation.rs`.

Out-of-band (controller-driven) mutations call the typed SDK directly: `await ship.sessions.start({...})`.

## Motion

`transition:` / `in:` / `out:` are retired — the compiler rejects them with a
teaching error pointing at `motion:`, the one animation system. `motion:` is a
directive, not a runtime import: `motion:enter`, `motion:exit`, and trigger
variants like `motion:hover` take a static preset object literal
(`motion:exit={{ duration: 150 }}`) and are wired through the
`TransitionRunner` seam (`packages/sui/runtime/src/transition-runner.ts`),
backed by `packages/sui/runtime/src/motion.ts`'s lifecycle animations.

See `packages/sui/tutorial/src/prose/06-motion-is-a-directive.md` for the
canonical walkthrough.

## Morph — in-place DOM patching

Add the `morph` directive to any element whose contents are produced by `bindRawHtml` (or another HTML-string-replacing path) to preserve focus, scroll, selection, and event-listener identity across re-renders:

```html
<div morph>
  <!-- server-rendered HTML — bindRawHtml in the controller -->
</div>
```

Without the directive, `bindRawHtml` replaces `innerHTML` and destroys any in-progress UI state. With it, updates route through `morph()` — same-id elements are reused (even across reorders), attributes are swapped only when they change, and only truly-different subtrees are touched.

`bindHtml` always morphs (no directive required); the directive only matters for `bindRawHtml` and any future HTML-producing binding. Limitations: doesn't preserve `<select>` value across morphs (browser quirk), matches only within a single DOM tree (component boundaries are in-DOM in light DOM — there is no shadow to morph into), matches by `id` and tag-plus-position only.

## Error boundary

Wrap fault-prone subtrees in `<sui-error-boundary>`. Throws inside the wrapped subtree (effects, controller mount, render expressions) are caught and the fallback is rendered:

```html
<sui-error-boundary fallback-text="Couldn't load feed">
  <sui-some-component prop:data={maybeThrows()}></sui-some-component>
</sui-error-boundary>
```

Use `fallback-text` for a string fallback or `<template slot="fallback">` for custom markup. Call `.reset()` on the host element to retry. The boundary fires `sui-error-boundary:caught` (bubbling, composed) with `{ error, source }` where `source` is `'effect' | 'mount' | 'render' | 'unknown'`. Nest boundaries to scope recovery — the innermost catches first.

Lookup is DOM-driven (`findNearestBoundary` walks up the host's light-DOM tree — there are no shadow roots to cross in the default model), so the runtime, component factory, and effect layers don't need to carry a boundary reference.

## Portal

```html
<sui-popover-trigger>Open</sui-popover-trigger>
<sui-portal target="body">
  <sui-popover-panel>Floating content</sui-popover-panel>
</sui-portal>
```

Teleports slotted content into a different DOM location for stacking-context escape. `target` is a CSS selector (default `body`). The portal element stays the logical parent — reactive bindings inside still fire and events bubble through the original DOM ancestry, not the target's. Missing target warns to the console and leaves children in place. Teleported nodes are removed from the target when the portal unmounts.

Use it for *document-level state markers* (`[data-sui-dialog-open]`, `[data-sui-popover-open]`) and for content that genuinely needs to escape transformed / overflow ancestors (toast stacks). Don't re-portal panels whose host already uses the native top layer (`<dialog>.showModal()`, `popover="auto"`) — the host's native rendering already handles stacking.

Worked exemplars: `sui-dialog`, `sui-popover`, `sui-toast-container`.

## Projection

Native `<slot>` is a shadow-DOM-only feature, so the runtime
reimplements the `slot=` / `<slot>` contract in light DOM
(`packages/sui/runtime/src/slot-project.ts`). Author surface is
the same as web components:

```html
<!-- in a component body -->
<slot name="leading"></slot>
<slot></slot>
<slot name="trailing"></slot>
```

The runtime entry point is `projectSlots(host, captured)` —
captured is the host's original children, taken at connect before
the template overwrote them. It assigns each child to the matching
`<slot name="…">` (or the default `<slot>` for un-named children)
and drops unslotted content. The same-named slots that shadow has
(named, default, fallback-dropped-when-assigned) all apply.

For component authors who need the declarative surface:
`resolveSlots(host, slots, originalChildren)` takes an array of
`SlotDecl` (name + anchor + fallback) and does the assignment;
`snapshotChildren(host)` returns the host's children at call time;
`slotFilled(host, name)` reports whether content landed. The
component factory uses `projectSlots` directly; the declarative
surface is for hand-written component bodies.

For the empty-state pattern (host attribute when a region is
empty), `bindSlotEmpty(slot, host, attrName)` (`slot-empty.ts`)
toggles the named boolean attribute on `host` based on whether
the slot has visible content. It works in both shadow and light
components — in light DOM it watches the slot wrapper's children
via `MutationObserver` (no `slotchange` event fires); in shadow
it listens to `slotchange`. The compiler emits one `bindSlotEmpty`
call per `<slot reflect-empty="…">` directive in a component
body. The attribute name is author-chosen per slot — the directive's
value IS the host attribute (`<slot reflect-empty="data-icon-only">`
toggles `data-icon-only` on the host); there is no fixed name to
specify.

Known limit: projection runs once at connect. Children appended
to the host LATER are not re-projected — the dynamic composition
path in SUI templates goes through compiled call sites, not raw
DOM appends, so this covers the authored surface.

## Template hooks — `<use-*>`

```html
<use-kbd><on key="/" action="search" /></use-kbd>
<use-click-outside bind:target={dialogEl} bind:action={close} />
<use-focus-trap bind:target={dialogEl} />
<use-clipboard />
<use-title source={title} />
<use-scroll-anchor />
<use-intersection callback={onIntersect} />
```

The compiler emits a side-effect import of `@ship/sui-runtime/template-hooks` into views that contain `<use-*>` elements, so hook code only ships with views that use it. Register a custom hook via `registerTemplateHook(name, factory)`; `listRegisteredHooks()` enumerates.

## Routes — reactive URL state

```ts
import { controller, derived } from '@ship/sui-runtime'
import { defineRoutes, navigate } from '@ship/sui-runtime/route'
import type { Scope } from './page.sui'

defineRoutes([{ pattern: '/agents/:id' }])

export default controller<Scope>({
  scope: ({ route }) => ({
    agentId: derived(() => route.params.id() ?? ''),
    tab:     derived(() => route.query.tab() ?? ''),
    goHome:  () => navigate('/'),
  }),
})
```

`route` is injected into every controller section's scope. `route.params.<name>` and `route.query.<name>` are signals — call them (`route.params.id()`) to read and subscribe inside `effect`, `derived`, or a `<data>` source thunk. `popstate` and `navigate` both fire the global updater so every attached view re-evaluates. (`route` is also importable as a `route()` factory from `@ship/sui-runtime/route` for non-controller code; the namespace form `route.params.id()` is the controller exemplar — see `views/agents/[id]/controller.ts`.)

## Mount API

```ts
import { mount, type MountHandle } from '@ship/sui-runtime'

const handle: MountHandle = mount(target, payload, opts)
// later: handle.dispose()
```

`mount(target, payload, opts)` is the single entry hosts call to put a view on screen — used by the daemon view-host and by `apps/shell/src/main.ts` for route-driven mounts. The HMR layer disposes and re-mounts through the same call.
