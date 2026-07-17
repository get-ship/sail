---
group: Views
order: 6
title: SSR and the attach runtime
description: How `shipd` renders `.sail` to HTML server-side, and how the client runtime attaches reactivity to the rendered DOM. Progressive enhancement first.
---

# SSR and the attach runtime

SAIL is **server-rendered first**. `shipd` renders the template to HTML with initial values inlined; the page is meaningful before any JavaScript runs. A small client runtime then attaches event handlers, signals, and SSE subscriptions to the existing DOM.

This is progressive enhancement, not hydration of an empty shell. The framework's design assumes the daemon owns the data and the page should reflect it immediately on first paint.

## The pipeline

```
.sail source
    │
    ▼
sail-compiler  (Rust, AOT)
    │
    ▼
IR (lowered, span-tracked)         ─── consumed by both sides ───
    │                                                            │
    ├──────────────────────────┐                                 │
    ▼                          ▼                                 ▼
Rust render walker   ESM attach instructions   (eval rig, LSP, tooling)
in shipd                    in browser
    │                          │
    ▼                          ▼
Server HTML  ────►  Browser parses  ────►  Attach runtime wires reactivity
                                            + opens SSE for `data live`
```

The IR is the single source of truth. Both the server render walker (Rust, in `shipd`) and the client attach runtime read the same IR. There is no second grammar, no client parser on the production path.

## Server render

`shipd` evaluates the template against initial signal values:

| Template construct | Server behavior |
|---|---|
| Static HTML | Emitted verbatim. |
| `{expr}` (renderable subset) | Evaluated against initial state; result inlined. |
| `{expr}` (non-renderable) | Marker emitted; client evaluates on attach. |
| `{#if cond}` | Server evaluates `cond`; only the matching branch is emitted, with markers identifying which branch is live. Client swaps on signal change. |
| `{#for x of xs}` | Server expands `xs` against initial value; emits all items with `key` markers. Client adds/removes by key. |
| `{#match expr}` | Server evaluates `expr`; only the matching case is emitted. |
| `{#while-loading of X}` | Server emits the body if `X.kind === 'loading'`, otherwise emits a hidden placeholder. Client swaps on envelope flips. |
| `{#while-error err of X}` | Server emits the body if `X.kind === 'error'` (with `err` bound to `X.error`), otherwise emits a hidden placeholder. Client swaps on envelope flips. |
| `<data live />` | Server emits the initial value inline (with envelope `kind: 'ok'`); client opens the SSE channel for patches. |
| `on:click={fn}` | Marker emitted; no listener attached server-side. |
| `bind:value={sig}` | Marker emitted; client wires both directions. |
| `<use-* />` | Marker emitted; client invokes the hook factory at mount. |
| Custom elements (`<sail-button>`) | Emitted as their tag. Browser self-upgrades on parse via standard custom-element registration. |

## The renderable expression subset

Expressions inside `{ ... }` are SSR'd if they stay inside this subset:

- Signal calls — `count()`, `agents().value.items`
- Property access — `user().name`, `a.id`
- Arithmetic and comparison — `count() + 1`, `age() >= 18`, `n() % 2 === 0`
- Boolean operators — `loading() || error()`, `enabled() && active()`
- Ternaries — `cond ? a : b`
- Template literals with no embedded function calls — `` `${name()} (${total()})` ``

Expressions **outside** the subset (function invocations beyond signal reads, closures, `new`, `await`, `this`, arbitrary JS) become client-only. The compiler emits a placeholder marker; the client runtime evaluates on attach. The compiler also emits a warning so the author can decide whether to refactor toward the subset.

### Method allowlist

Method calls inside the subset are restricted to the allowlist in `crates/sail/compiler/src/check/expr_subset.rs::METHOD_ALLOWLIST`. Anything outside is rejected at compile time. Current entries (read-only, deterministic, no I/O, no eval-shaped args):

`slice`, `map`, `filter`, `includes`, `toString`, `length`, `keys`, `values`, `entries`, `toFixed`

### Closure rule — adding a method

Any addition to the allowlist must satisfy four constraints, **in this order**:

1. **Referentially transparent.** No I/O, no allocation that escapes, no mutation, no closures-as-args, no eval-shaped args. If the method takes a function (e.g. `.map(fn)`), the function must itself stay inside the renderable subset — verified by walking the AST.
2. **SSR implementation in `crates/sail/compiler/src/render/eval/methods.rs::dispatch`.** Match the method name, evaluate via the `coerce`/`exec` helpers, return a `serde_json::Value`. Must produce identical output to the runtime JS semantics for the same inputs — round-trip tested.
3. **Test pair.** One test in `crates/sail/compiler/tests/` exercising the method through SSR (input → expected HTML). One test in a framework integration test exercising the same expression in a `.sail` view's attached state — confirms SSR output matches client-eval output character-for-character.
4. **Docs entry.** Add to the allowlist line above (this section), with one line of rationale if non-obvious. If the method has subtle browser-vs-server semantics (number formatting locales, etc.), call them out.

Reject the addition if any of (1)–(4) can't be met. The subset is intentionally narrow — a method that requires special-casing in SSR is a method we shouldn't admit. Better to direct the author to a `derived()` in the controller.

## Markers

The serialization shape between the server-emitted HTML and the client attach runtime:

| Source construct | Emitted marker |
|---|---|
| `{expr}` text interpolation | `<!--sail:text:bN-->value<!--/sail-->` comment pair |
| `on:click={fn}` | `on:click="bN"` attribute preserved on the element |
| `bind:value={sig}` | `bind:value="bN"` attribute preserved on the element |
| `class:active={expr}` | `class:active="bN"` attribute preserved |
| `style:color={expr}` | `style:color="bN"` attribute preserved |
| `attr:href={expr}` | `attr:href="bN"` attribute preserved |
| `{#if}` / `{#for}` / `{#match}` — active branch | `<sail-if key="bN">…</sail-if>` (or `<sail-for>`, `<sail-match>`) |
| `{#if}` / `{#for}` / `{#match}` — inactive branch | `<sail-if key="bN" hidden></sail-if>` |
| `{#while-loading}` / `{#while-error err}` — active branch | `<sail-while-loading key="bN">…</sail-while-loading>` (or `<sail-while-error>`) |
| `{#while-loading}` / `{#while-error err}` — inactive branch | `<sail-while-loading key="bN" hidden></sail-while-loading>` |
| Initial signal values | `<script type="application/json" id="__set_init__">…</script>` |

`b<N>` keys are allocated by a counter in source order; the sidecar binding manifest (exported as `bindings` from the compiled ESM) maps each key to its kind, expression, and dependency list. Boundary entries (`if`, `for`, `match`, `while-loading`, `while-error`) carry a `renderKey` pointing at the body's render callback exported alongside `default` on the compiled module — `attach()` invokes that callback through `bindWhileLoading` / `bindWhileError` (or the parallel `bindIf`/`bindFor`/`bindMatch`) when the deps change. The attach runtime queries `[on:*]`, `[bind:*]`, `[class:*]`, `[style:*]`, `[attr:*]` to find leaf bindings and walks `<sail-if>/<sail-for>/<sail-match>/<sail-while-loading>/<sail-while-error>` elements for control-flow boundaries. Net `data-*` attributes in framework-emitted HTML: zero.

## Client attach runtime

The browser-side runtime is **attach-only**. It does not parse templates, does not construct DOM from scratch, does not run the IR's render path. Its job:

1. Walk the rendered DOM looking for markers.
2. For each marker, read the binding from the manifest (or inline annotation).
3. Initialize signals from inlined initial values (`<data>` markers carry `kind: 'ok'` envelopes with values; reactive component state returned by the controller function materializes when the controller runs at hydrate).
4. Attach event listeners (`on:click`, `bind:`).
5. Wire reactive bindings (text interp, attribute interp, control-flow boundaries) to track signals and update the DOM on change.
6. Open SSE channels for each `<data live>` and apply patches as they arrive.
7. Invoke template hook factories (`<use-*>`) and register their cleanups.
8. Run controller `mount` after all of the above.

State changes from local signals (button clicks, bind:value) flow through handlers, mutate state, and trigger surgical DOM updates against the existing nodes. There is no morph step — the runtime knows precisely which nodes correspond to which bindings.

`data live` SSE patches arrive as JSON; the runtime applies them to the live signal and downstream bindings update.

## Progressive enhancement

A view rendered without JavaScript is still meaningful:

- Static structure renders.
- Initial values from `<data>` are inlined; reactive controller-scope signals are dormant until JS runs.
- `{#if}` / `{#for}` / `{#match}` show their initial branches.
- Custom elements show their server-rendered children (the framework does not require client-side construction for non-island components).

What's lost without JS:

- Event handlers (`on:click` does nothing; submit-style handling depends on form fallback)
- `bind:value` updates (form fallback works; reactive updates do not)
- `data live` patches (initial values are visible; updates are not)
- Template hooks (`<use-kbd>`, `<use-title>` etc. are no-ops)

For the local-daemon case Ship targets, this matters less than for the public-web case — JS is essentially always available. But the architecture *enables* PE, and that's the discipline: the server renders meaningful HTML, the client enhances it.

## What replaced the Phase 3 architecture

Two pieces of the previous (Phase 3) architecture are gone:

- **Client-side template parser** — the client never parses `.sail` source on the production path. The parser lives in `@ship/sail-dom/legacy` and is used only by tests and the eval rig. The default `@ship/sail-dom` entry exports only `attach` and has zero parser code.
- **The empty HTML shell** — `shipd` no longer serves an empty `<div id="root">` shell at `/sail/<view>`. It serves server-rendered HTML directly.

Both follow from the IR-as-source-of-truth move. Two parsers (Rust + JS) and two emit shapes (empty shell + client-rendered DOM) become one parser (Rust) and one emit shape (rendered HTML).

## Import surface: default vs. legacy

`@ship/sail-dom` exposes two import paths with distinct responsibilities:

| Path | Exports | Who uses it |
|---|---|---|
| `from '@ship/sail-dom'` | `attach()`, `AttachOptions`, `loadSetup` | Production views, daemon asset bundles |
| `from '@ship/sail-dom/legacy'` | `hydrate()`, `parseSetSource()`, `HydrateOptions` | Tests, eval harness |

The parser (`hydrate`, `parseSetSource`, `directives/`) lives exclusively in the `legacy` subpath. The default `@ship/sail-dom` bundle (`sail-dom.js`) contains zero parser code; it sits at ≤ 4.8 KB gzip today, against a 9 KB ceiling enforced by `scripts/lint/check-sail-bundle-size.sh` in CI.

**Never import `@ship/sail-dom/legacy` from a production view.** Doing so would pull the full parser (~5 KB gzip) into the view bundle and violate the size budget. The bundle-size lint asserts zero references to `parseSetSource` in `sail-dom.js`.
