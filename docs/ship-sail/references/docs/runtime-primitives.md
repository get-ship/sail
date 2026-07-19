---
group: Views
order: 5
title: SAIL — Runtime Primitives
description: The signal/bind/effect primitives the SAIL runtime exposes.
---

# SAIL — Runtime Primitives

All symbols below are re-exported from `packages/sail/runtime/src/index.ts`. Anything not in that barrel is internal.

## Reactivity

```ts
import { signal, derived, effect, batch, untrack } from '@ship/sail-runtime'

const count   = signal(0)
const doubled = derived(() => count() * 2)

effect(() => console.log(doubled()))     // re-runs on tracked-dep change
batch(() => { count.set(1); count.set(2) })  // flushes once
untrack(() => count())                    // read without subscribing
```

`signal<T>(initial)` returns `Signal<T>`: call to read, `.set(v)` / `.update(fn)` to write. `derived<T>(fn)` returns `Derived<T>` (read-only). `effect(fn)` re-runs on dep change; `batch(fn)` coalesces writes; `untrack(fn)` reads without recording.

Scopes (`createScope`, `createChildScope`, `withScope`, `onCleanup`, `currentScope`) own a tree of disposers — child scopes dispose when their parent does. The dev-only leak detector lives at `@ship/sail-runtime/dev` (separate subpath so it doesn't ship in production).

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

Runtime helpers behind these: `bindIf`, `bindFor`, `bindForSignal`, `bindWhileLoading`, `bindWhileError`. The `<sail-if>`, `<sail-for>`, `<sail-match>` custom elements register on module load.

`{#for}` requires `key={EXPR}` — without a stable key the reconciler falls back to positional matching and DOM identity (focus, selection, in-progress edits) is lost on reorder. `sail-for-key` lint warns when missing.

## `query()` / `queryKeyed()` — fetch + subscribe

```ts
<controller>
import { controller, query, queryKeyed } from '@ship/sail-runtime'

export default controller(({ ship }) => ({
  // one-shot fetch
  settings: query(() => ship.settings.state()),

  // live subscription
  agents: query(() => ship.agents.summaries()),

  // reactive args — re-subscribes when selectedId changes
  agent: queryKeyed(
    () => ['ship', 'agents', 'detail', selectedId()],
    () => ship.agents.detail({ id: selectedId() }),
  ),
}))
</controller>
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
.ts   query(() => ship.git.overview())   // in the controller — ship.git.overview() returns a LiveSource → query() routes to the push (subscribe) path
  ↓ sail-compiler
.js   $scope.git = query(() => ship.git.overview())
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

**Worked example.** `views/home/` ships the canonical demo: a live `query()` subscribes to `ship.git.overview()`; the daemon's git-signal singleton refreshes on `.ship/` filesystem events and on git ops, so staging a file flips the badge live.

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
`TransitionRunner` seam (`packages/sail/runtime/src/transition-runner.ts`),
backed by `packages/sail/runtime/src/motion.ts`'s lifecycle animations.

See `packages/sail/tutorial/src/prose/06-motion-is-a-directive.md` for the
canonical walkthrough.

### Two lanes

`motion:` is one of two lanes, and the split is about **who owns the clock**.

| | `motion:` | `timeline()` |
| --- | --- | --- |
| clock | the browser | you (a signal) |
| runs on | the compositor (WAAPI) | the main thread |
| surface | a directive + a closed preset catalogue | a runtime import |
| good at | hover/press, enter/exit, reveal-on-scroll, ambient loops | scrubbing, holds, per-segment easing, many elements against one progress value |

`motion:` is cheaper and should be preferred whenever it can express the
motion. It cannot express motion whose timing something else owns — a slider
scrubbing an animation, or a multi-stop sequence that must settle at an exact
offset and hold there. That is `timeline()`.

### `timeline(progress, tracks, options?)`

`packages/sail/runtime/src/timeline.ts`. Takes a progress reader (`() =>
number`, `0..1`) and a map of keyframe tracks; returns one memoized signal per
track. Return them from a controller and the existing `(--name)` reflection
carries each to the style layer — a timeline track resolves in an `sx` read
exactly like a `signal(...)` does (`crates/sail/compiler/src/check/sx_vars.rs`).

```ts
const { tileX, tileScale } = timeline(progress, {
  tileX: [[0, '0px'], [0.34, '120px'], [0.44, '120px'], [1, '0px']],
  tileScale: [[0.44, 1], [0.54, 1.75], [0.64, 1]],
}, { easing: [0.6, 0, 0.2, 1] })
```

- **Offsets** are `0..1` and must ascend. Progress outside the first/last
  offset clamps.
- **Holds** are the same VALUE repeated at a later offset. Repeating an
  *offset* is an error — it is almost always the typo for a hold.
- **Easing** on a stop governs the segment that STARTS at it (WAAPI's rule, so
  captured `cubic-bezier(...)` values transfer verbatim). `options.easing` is
  the default for segments that name none; it defaults to `linear`. Accepts a
  bezier tuple, a CSS easing name, or a `0..1 → 0..1` function.
- **Units** are strict: a track is all numbers or all strings sharing one unit,
  validated at construction. Numbers come back as numbers; unit strings keep
  their unit. Colors and multi-part values are not supported — split them into
  numeric channels.

Holds are free downstream: a held segment recomputes to an `Object.is`-equal
value, so the reflection effect never writes.

### Drivers

`packages/sail/runtime/src/drivers.ts`. Each returns a plain signal, so they
are interchangeable with a hand-written `signal(0)` that a range input writes.
All of them stop when the enclosing scope disposes.

- `clock(durationMs, { loop?, direction?, autoplay?, reducedMotion? })` —
  wall-clock progress on motion.dev's frame loop (the same loop `motion:` uses,
  so a page mixing both lanes still has one rAF). Carries `.play()`,
  `.pause()`, `.stop()`, `.running()`.
- `scrollProgress(target?, { axis? })` — document scroll, or a target
  element's `cover 0% → 100%` travel through the viewport.
- `springValue(source, springOptions?)` — chases another signal with real
  physics; retargeting mid-flight preserves velocity. Its output is a value,
  not necessarily `0..1`.

**Reduced motion is decided once, in the driver.** `clock` defaults to
jump-to-end — the settled value, no frame loop — matching what `motion.ts`
does for `motion:` directives; `reducedMotion: 'start' | 'play'` overrides.
`springValue` mirrors its source instantly. `scrollProgress` deliberately does
not suppress: it reports what the user is already doing, and freezing it would
freeze the page mid-scroll.

### Cost

`timeline()` writes custom properties from JS every frame — main-thread style
recalc, which is exactly what `motion:` avoids. Measured in headless Chromium
(`packages/sail/runtime/e2e/timeline-tile.spec.ts`), one clock driving N
elements at three channels each: ~60fps at 1, 12, and 60 elements (180
channels). Well past what a page choreographs at once, but it is not free, and
it is why `motion:` stays the default.

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

Wrap fault-prone subtrees in `<sail-error-boundary>`. Throws inside the wrapped subtree (effects, controller mount, render expressions) are caught and the fallback is rendered:

```html
<sail-error-boundary fallback-text="Couldn't load feed">
  <sail-some-component prop:data={maybeThrows()}></sail-some-component>
</sail-error-boundary>
```

Use `fallback-text` for a string fallback or `<template slot="fallback">` for custom markup. Call `.reset()` on the host element to retry. The boundary fires `sail-error-boundary:caught` (bubbling, composed) with `{ error, source }` where `source` is `'effect' | 'mount' | 'render' | 'unknown'`. Nest boundaries to scope recovery — the innermost catches first.

Lookup is DOM-driven (`findNearestBoundary` walks up the host's light-DOM tree — there are no shadow roots to cross in the default model), so the runtime, component factory, and effect layers don't need to carry a boundary reference.

## Portal

```html
<sail-popover-trigger>Open</sail-popover-trigger>
<sail-portal target="body">
  <sail-popover-panel>Floating content</sail-popover-panel>
</sail-portal>
```

Teleports slotted content into a different DOM location for stacking-context escape. `target` is a CSS selector (default `body`). The portal element stays the logical parent — reactive bindings inside still fire and events bubble through the original DOM ancestry, not the target's. Missing target warns to the console and leaves children in place. Teleported nodes are removed from the target when the portal unmounts.

Use it for *document-level state markers* (`[data-sail-dialog-open]`, `[data-sail-popover-open]`) and for content that genuinely needs to escape transformed / overflow ancestors (toast stacks). Don't re-portal panels whose host already uses the native top layer (`<dialog>.showModal()`, `popover="auto"`) — the host's native rendering already handles stacking.

Worked exemplars: `sail-dialog`, `sail-popover`, `sail-toast-container`.

## Projection

Native `<slot>` is a shadow-DOM-only feature, so the runtime
reimplements the `slot=` / `<slot>` contract in light DOM
(`packages/sail/runtime/src/slot-project.ts`). Author surface is
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
path in SAIL templates goes through compiled call sites, not raw
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

The compiler emits a side-effect import of `@ship/sail-runtime/template-hooks` into views that contain `<use-*>` elements, so hook code only ships with views that use it. Register a custom hook via `registerTemplateHook(name, factory)`; `listRegisteredHooks()` enumerates.

## Routes — reactive URL state

```ts
<controller>
import { derived } from '@ship/sail-runtime'
import { defineRoutes, navigate } from '@ship/sail-runtime/route'

defineRoutes([{ pattern: '/agents/:id' }])

export default controller(({ route }) => {
  const agentId = derived(() => route.params.id() ?? '')
  const tab     = derived(() => route.query.tab() ?? '')
  const goHome  = () => navigate('/')

  return { agentId, tab, goHome }
})
</controller>
```

`route` is part of the ambient `ctx` every controller function receives. `route.params.<name>` and `route.query.<name>` are signals — call them (`route.params.id()`) to read and subscribe inside `effect`, `derived`, or a `query()`/`queryKeyed()` thunk. `popstate` and `navigate` both fire the global updater so every attached view re-evaluates. (`route` is also importable as a `route()` factory from `@ship/sail-runtime/route` for non-controller code; the namespace form `route.params.id()` is the controller exemplar — see `views/agents/[id]/page.sail`'s `<controller>` block.)

## Mount API

```ts
import { mount, type MountHandle } from '@ship/sail-runtime'

const handle: MountHandle = mount(target, payload, opts)
// later: handle.dispose()
```

`mount(target, payload, opts)` is the single entry hosts call to put a view on screen — used by the daemon view-host and by `apps/shell/src/main.ts` for route-driven mounts. The HMR layer disposes and re-mounts through the same call.
