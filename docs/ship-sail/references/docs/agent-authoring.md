---
group: Agents
order: 2
title: SAIL — Agent Authoring
description: How agents author .sail views end to end.
---

# SAIL — Agent-Authoring Guide

For an AI agent producing a Ship view in one shot. Read the parent
`SKILL.md` first for the four-layer overview and the hard rules; this
file is the do-now checklist.

Throughout: file paths are relative to the repo root, every snippet is
either lifted from a real view (cited inline) or testable in isolation
against `bun run --filter @ship/sail-runtime test`.

> **Light-DOM contract.** Every `sail-*` element renders into its
> own custom element in the page's tree — no shadow root by
> default. Cross-root ARIA (`label[for]`, `aria-describedby`),
> native form participation, and a single document cascade all
> work without any retargeting. A component opts into a shadow
> boundary only when it wraps foreign content the page cascade
> would interfere with (the tiptap wrapper, embedded
> third-party widgets, the FFI boundary); that opt-in is declared
> in the anatomy manifest, not the template.

---

## 1. The 30-second scaffold

```sh
deck scaffold view <name>          # e.g. tasks
deck scaffold view <name> --path /tasks
```

Output (`views/<name>/`):

| File | Purpose | Required |
|---|---|---|
| `page.sail` | Template — declares route body, control flow, and (if needed) an inline `<controller>` block for state, lifecycle, data subscriptions, mutations | Yes |
| `style.css` | Page-scoped CSS, auto-linked by the daemon | Optional |
| `manifest.jsonc` | `{ name, path, title, layout, auth, rail? }` | **Yes — routes are manifest-driven** |
| `_components/<view>-<tag>/` | View-private component (`<tag>.sail`, with its own inline `<controller>` block if needed) | Optional |

**Rules from `references/4-layer-overview.md` § Layer 4 that an agent
gets wrong most often:**

- Filename stem says what the file IS: `page.sail`, never `view.sail`,
  never `<name>.sail`.
- A controller is always an inline `<controller>` block inside the
  `.sail` file — a paired `<stem>.controller.ts` sibling file is a
  compile error. The directory IS the view.
- View-private components live under `_components/<view>-<tag>/`, and
  the tag MUST start with the view name (so `tasks-row`, not `row`)
  to avoid `customElements.define()` collisions across views.

Verify the scaffold parses before adding logic:

```sh
cargo run -p sail-lint --example check -- views/<name>/page.sail
```

---

## 2. Anatomy of a `.sail` template

```html
<!-- views/<name>/page.sail — declarative only. No <script>. -->

<!-- 1. TOP-LEVEL DIRECTIVES — declare scope before the body -->
<prop name="tone" values="default|primary" default="default" reflect /> <!-- components only -->

<!-- 2. BODY — element tree with reactive expressions -->
<div class="<name>-view">
  <header>
    <h1>{title()}</h1>            <!-- signals are CALLED to read -->
    <p>{total()} item{total() === 1 ? '' : 's'}</p>
  </header>

  {#while-loading of items}…skeleton…{/while-loading}
  {#while-error err of items}<p>{err.message}</p>{/while-error}
  {#if items().kind === 'ok'}
    {#if rows().length === 0}
      <sail-empty-state>No rows.</sail-empty-state>
    {:else}
      <ul>
        {#for r of rows() key={r.id}}<li>{r.name}</li>{/for}
      </ul>
    {/if}
  {/if}

  <!-- single-expression handlers stay inline; multi-statement ones become a named function returned from the controller -->
  <sail-button on:click={() => count.set(count() + 1)}>+</sail-button>

  <style>/* scoped per-component CSS belongs here for components */</style>
</div>
```

Surface in compact form (full grammar in
`references/authoring-patterns.md`):

| Form | Meaning |
|---|---|
| `query(() => ship.foo.bar())` in the controller | `ship.foo.bar` returning a `LiveSource` opens a subscription (push); returning a `Promise` fetches once (pull). Either way, produces a `LiveSignal<T>`. |
| `{#if expr}…{:else if expr}…{:else}…{/if}` | Conditional. `:else if` is allowed. |
| `{#for x of arr() key={x.id}}…{/for}` | List. **`key={EXPR}` is required** (lint warns). |
| `{#match expr}{:case 'a'}…{:default}…{/match}` | Discriminated union dispatch. |
| `{#while-loading of liveSig}…{/while-loading}` | Loading branch for a live `query()` signal. |
| `{#while-error err of liveSig}…{/while-error}` | Error branch; `err: Error` in scope. |
| `bind:value={sig}` / `bind:checked={sig}` | Two-way input binding. |
| `bind:this={refSig}` | Element ref; controller scope must declare `refSig: ref(null)`. |
| `on:event={fn}` | DOM or `sail-*`/`ship-*` custom event listener. |
| `class:x={fn}` / `style:prop={fn}` / `attr:name={fn}` | Reactive class / style / attribute toggle. |
| `prop:rows={rowsSignal}` | Structured prop into a child component (skips attr serialization). |
| `<slot name="leading"></slot>` (component body) + `slot="leading"` (consumer) | Projection region: consumer-side `slot="…"` attribute fills the matching `<slot name="…">` wrapper; runtime projects at connect. |
| `<use-kbd>…</use-kbd>` / `<use-click-outside>` | Template hooks; side-effect import emitted by compiler. |

**Scoped CSS.** A view's `style.css` is page-scoped (auto-linked). A
component's CSS goes in a `<style>` block inside the `.sail`, or for
external CSS via `compile_opts.styles`. The compiler does **not**
shadow-DOM anything by default — every `sail-*` renders into its
own element with no shadow root. Scoped CSS is `@scope
([data-component="sail-x"]) to ([data-sail])` (a donut that stops
at nested component boundaries); the compiler rewrites `:host` →
`:scope` and `::slotted(<sel>)` → `slot > <sel>` at emit time
(`crates/sail/compiler/src/emit/styles.rs` `light_dom_selectors`).
Class names still need a per-view prefix (`<name>-…`, see
`views/agents/style.css` for the `.agents-*` pattern) for the
**page-cascade** portion — `@scope` covers component-internal
rules, not view-scoped page CSS.

---

## 3. Anatomy of an inline `<controller>` block

```ts
<!-- inside views/<name>/page.sail -->
<controller>
import { signal, derived, query, unwrap } from '@ship/sail-runtime'

export default controller((ctx) => {
  const { ship, route, navigate } = ctx
  const items = query(() => ship.<domain>.<op>(args))
  const q = signal('')
  const rows  = derived(() => unwrap(items)?.rows ?? [])
  const total = derived(() => unwrap(items)?.rows.length ?? 0)

  const create = () => navigate('/<name>/create')

  mount((el) => { /* runs after first attach */ })
  onCleanup(() => { /* runs on dispose */ })

  return { items, q, rows, total, create }
})
</controller>
```

One function, one job: `controller(fn)` where `fn: (ctx) => Record<string, unknown>`.

| Piece | Receives | Returns | Notes |
|---|---|---|---|
| `ctx` (the `fn` argument) | `{ ship, clients, route, navigate }` | — | Ambient view scope, directly accessible or destructured. |
| return value of `fn` | — | Bag of `signal` / `derived` / plain helpers / event handlers | Whatever is returned IS the whole contributed template scope — no separate section for handlers. |
| `mount(fn)` | `fn: (el: HTMLElement) => (() => void) \| void` | — | Ambient, callable anywhere in the setup call graph. Fires once after the instance's DOM connects; a returned function is a cleanup on dispose. |
| `onCleanup(fn)` | `fn: () => void` | — | Ambient. Runs on dispose; no-ops (doesn't throw) outside a scope. |

Mutation is always `signal.set(newValue)` (or `signal.set(prev => next)`) — there's no writable-state-proxy assignment style.

**`query()` + `ship` SDK.** `query()` reads server data. It inspects
what the client method returns and routes accordingly: a method that
returns a `LiveSource<T>` (`{ subscribe(sig) }`, e.g. an SSE channel)
makes `query()` subscribe and keep patching the same cache entry (push);
a method that returns a `Promise<T>` makes it fetch once (pull). Either
way you get the same `LiveState<T>` (`loading`/`ok`/`error`) signal out.
Two wiring paths, depending on mount host:

1. **Direct (preferred when `ship` is in ctx).** Call
   `query(() => ship.foo.bar(args))` in the controller. Since
   `ship.foo.bar(...)` returns a `LiveSource`, `query()` takes the
   push path.

2. **Hand-wired (required for view-host mounts).** The daemon's
   `/sail/<name>` host does NOT inject `ship` into ctx, so a direct
   `query(() => ship.foo.bar())` call has no `ship` to close over and
   sticks on `loading`. Wire the subscription directly to
   `/_deck/data?expr=…` instead. The canonical helper currently lives
   in `views/git/page.sail`'s inline `<controller>` block (`sseSource`)
   — copy verbatim:

```ts
function sseSource<T>(expr: string): LiveSource<T> {
  return {
    subscribe(sig: LiveSignal<T>): () => void {
      const es = new EventSource(`/_deck/data?expr=${encodeURIComponent(expr)}`)
      const onPatch = (ev: MessageEvent): void => {
        try {
          const patch = JSON.parse(ev.data as string) as { op: string; value: T }
          if (patch.op === 'set') sig.set({ kind: 'ok', value: patch.value })
        } catch (err) {
          sig.set({ kind: 'error', error: err instanceof Error ? err : new Error(String(err)) })
        }
      }
      es.addEventListener('patch', onPatch as EventListener)
      return () => { es.close() }
    },
  }
}
```

Default to the direct form (path 1) when authoring inside shell.
Adopt the `sseSource` helper if the view will also be linked to
directly via `/sail/<name>`.

**Refs (`bind:this`).** Declare in the controller function as
`ref(null)`; read with `refSig()` (returns the element or `null`):

```ts
import { ref } from '@ship/sail-runtime'
// inside controller((ctx) => { … })
const dialogEl = ref<HTMLDialogElement>(null)
```

```html
<dialog bind:this={dialogEl}>…</dialog>
```

`bind:this` inside `{#if}` / `{#for}` branches is now safe — the
compiler pre-seeds every `bind:this` slot as `signal(null)` even if the
controller declares no refs (fix recorded in
`.artifacts/sail-arc/view-render-reliability.md` § Diagnosis 2). The
slot returns `null` until the branch renders.

---

## 4. The canonical 3-branch shape

Every live data source has three states. The template MUST render
all three; never assume `ok`:

```html
{#while-loading of items}
  <!-- skeleton with the same outer structure as the loaded state -->
  <ul class="<name>-list" aria-label="Loading items">
    {#for n of [0,1,2] key={n}}
      <li><sail-skeleton shape="line" size="md"></sail-skeleton></li>
    {/for}
  </ul>
{/while-loading}

{#while-error err of items}
  <sail-error-state title="Couldn't load items">
    {err.message}
    <sail-button slot="action" on:click={() => window.location.reload()}>Retry</sail-button>
  </sail-error-state>
{/while-error}

{#if items().kind === 'ok'}
  {#if rows().length === 0}
    <sail-empty-state>
      <span slot="title">No items yet</span>
      Create your first one.
    </sail-empty-state>
  {:else}
    <ul class="<name>-list">
      {#for r of rows() key={r.id}}<li>{r.name}</li>{/for}
    </ul>
  {/if}
{/if}

```

> **Projection in this snippet.** The `<sail-button slot="action">`
> and `<span slot="title">` are projected children — the
> `slot="…"` attribute names the region, the runtime
> (`@ship/sail-runtime/slot-project.ts`) matches it to the
> corresponding `<slot name="…">` wrapper in the host's body at
> connect. The contract is the same as web-component slots, but
> the implementation is light-DOM (no shadow root, the slot is
> a `display: contents` wrapper). See
> [template.md](template.md#projection) for the full model.

Worked exemplars in this repo: `views/home/page.sail` (compact),
`views/agents/page.sail` (full skeleton grid), `views/sessions/page.sail`,
`views/workspaces/page.sail`.

The `kind === 'ok'` guard is **mandatory** before reading
`items().value` — accessing `.value` while `kind === 'loading'` or
`'error'` throws. This was the bug fixed in `dba0113f` (`fix(views/skills
+ probe): null-guard skill().value`).

---

## 5. Wiring to the daemon

`ship.<domain>.<op>(args)` resolves through `@ship/sdk-client`
(`packages/sdk-client/src/index.ts`). Available domains today —
**exhaustively**, as of the SDK source:

| Domain | Ops (live = SSE source; mutate = HTTP) |
|---|---|
| `agents` | `summaries()`, `create_ctx()`, `wire({id})`, `detail({id})`, `settings({id})`, `catalogs()` |
| `canvas` | `doc({id})`, `list()` |
| `chat` | `state({id})` |
| `collections` | `state({id})`, `detail({id})`, `items({id})`, `list()` |
| `dashboard` | `state()` |
| `events` | `list({event_type?, limit?})` |
| `git` | `overview({workspace_id?, limit?})`, `diff({…})`, `annotations({file, workspace_id?})` |
| `mesh` | `snapshot()` |
| `settings` | `state()` |
| `sessions` | `list()` (live); `start({workspace_id, kind})` / `end({id})` (mutate) |
| `workspaces` | `list()`, `active()` (live); `activate({id})` (mutate) |
| `skills` | `list()`, `summaries({q?, kind?, tag?})`, `detail({id})` |

**Adding a new live source** requires three coordinated edits — see
`references/runtime-primitives.md` § Live SSE end-to-end loop. The
short form:

1. Registry entry in `apps/shipd/src/services/registry/<domain>.rs`
   (synchronous current-value resolver) + dispatcher row in
   `registry.rs` + schema row in `services/schema.rs`.
2. SDK method in `packages/sdk-client/src/index.ts`.
3. (Optional) signal-backed source in
   `apps/shipd/src/views/sui_data_resolvers/` for tighter wake-up than
   the default `config` kernel namespace; mirror `git_signal.rs`.

Hand off all three Rust edits to `rust-lane` unless the job spec
explicitly grants `apps/shipd/` scope.

**Mutations** ride the same client. Two forms:

```html
<!-- declarative -->
<form mutation="ship.workspaces.activate">
  <input name="id" />
  <button type="submit" prop:disabled={$mutation().kind === 'loading'}>Switch</button>
</form>
```

```ts
// controller-driven — returned as a plain handler from the controller function
const create = async () => {
  await ship.sessions.start({ workspace_id, kind: 'shell' })
  navigate('/sessions')
}
```

Out-of-band: `await ship.sessions.start({...})` resolves to the daemon
response. The live `query()` subscription receives the projection patch
immediately — **don't** also call a manual refresh.

---

## 6. Registering in shell

Two places, both required:

1. **`views/<name>/manifest.jsonc`** — the route descriptor. Example
   (lifted from `views/canvas/manifest.jsonc`):

   ```jsonc
   {
     "name": "<name>",
     "path": "/<name>",
     "title": "<Title>",
     "layout": "default",
     "auth": "user",
     "rail": { "label": "<Label>", "icon": "<lucide-icon>" }
   }
   ```

   `auth: "public"` for views that work without an active workspace
   (home, dashboard); `auth: "user"` for everything else. `path` may
   include `[param]` segments for sub-routes — see
   `views/agents/[id]/manifest.jsonc`.

2. **`apps/shell/index.html` `<sail-rail>` block** — the shell's rail
   is currently a static list at `apps/shell/index.html:103-113`. Add
   your view:

   ```html
   <sail-rail-item icon="<lucide>" label="<Label>" href="/<name>" data-view="<name>"></sail-rail-item>
   ```

   The label flows through to the anchor's `title` for collapsed-icon
   hover text (`apps/shell/index.html:99-102` comment). The manifest's
   `rail` block is read by the daemon's view-host page; shell reads
   the hardcoded list. **Both must agree** — drift here is a frequent
   probe failure (see `.artifacts/sail-arc/shell-probe-evidence.md` § Rail
   inventory bugs).

`icon` is a lucide-icons name. `<sail-icon>` fetches one icon at a time from
`/_deck/icons/<name>.svg` (the daemon embeds the set + serves per-icon;
browser HTTP-caches each one). Don't bundle the registry, don't hard-code SVG.
For non-lucide art, use `<sail-icon src="…">` (CDN escape hatch).

---

## 7. Common pitfalls

Hard-won. Each one is from a real fix in this repo.

1. **`x().value` before `x().kind === 'ok'`.** `LiveState<T>` is a
   discriminated union — `.value` doesn't exist on `loading` /
   `error`. Always guard. Pattern: `{#if x().kind === 'ok'}{x().value.field}{/if}`,
   or pull the read through `unwrap(x)?.field ?? fallback` /
   `unwrapOr(x, EMPTY).field` (see the `<controller>` blocks in
   `views/home/page.sail` and `views/git/page.sail`). Fix that originated this rule:
   `dba0113f fix(views/skills + probe): null-guard skill().value`.

2. **`(x as any)` cast leftovers.** Every `as any` in a controller is
   a `unknown`-from-the-SDK shape that's never been typed. Either
   replace with the real generated type from `@ship/sdk-client`
   (`AgentSummary`, `GitOverview`, …) or define a local `interface`
   and parse the live envelope through it explicitly. The remaining
   `as any` in `views/agents/[id]/page.sail`'s `<controller>` block is a
   known leftover, not a pattern to copy.

3. **`route` access form.** `route` is part of the ambient `ctx` every
   controller function receives — destructure it (`(ctx) => { const { route, ship } = ctx; … }`)
   and read params with the namespace form `route.params.id()` (it
   returns `Signal<string|undefined>`, so the trailing `()` is the
   read). This is the form `views/agents/[id]/page.sail` uses. A
   `route()` factory is also importable from `@ship/sail-runtime/route`
   for non-controller code; don't mix the two within one file.

4. **`bind:this` inside a branch with no ref declared.** Used to
   throw `r.headerEl is not a function` at mount. Now safe — the
   compiler pre-seeds the slot (`view-render-reliability.md` § 2). But
   if you DO declare refs in the controller function, you must still
   declare `headerEl: ref(null)` explicitly when you want the typed handle.

5. **`{#for}` without `key={EXPR}`.** Lints as `sail-for-key` warning.
   Without a stable key, reorder destroys DOM identity — selection,
   focus, in-progress edits all lost. Use `key={item.id}` if there's a
   stable id; `key={i}` (positional index) is fine for static lists
   but disables identity preservation. Recent fix in this area:
   `789d0dab fix(sail-compiler): D5 inline-update leak across {#if}/{#match}
   branches inside {#for}`.

6. **`el.innerHTML` / `outerHTML` / `insertAdjacentHTML` in a
   controller.** Rejected by the `sail-controller-purity` lint.
   Templates declare; controllers do — use `bindHtml(el, () => string)`
   for HTML-blob cases (markdown, third-party SVG). See parent
   `SKILL.md` § Hard rules item 8.

7. **`route` is not a function (in chat).** The `defineRoutes` call
   must run at the top level of the `<controller>` block's module (it
   side-effect registers patterns). See
   `apps/shell/src/studio/components/[name]/page.sail`'s `<controller>`
   block for the canonical position — BEFORE the `controller((ctx) => {…})`
   default export. Putting it inside the controller function runs it
   too late, and the first `route()` call from a `derived` will throw.

8. **A direct `query(() => ship.foo.bar())` call sticks on `loading`
   under `/sail/<name>`.** The daemon's view-host mounts views without
   injecting `ship` into ctx. If the view will be linked to directly
   (not just through shell), hand-wire the live source with
   `sseSource(...)` (see § 3 above and the `<controller>` blocks in
   `views/skills/page.sail`, `views/git/page.sail`).

9. **Tag-name collisions across views.** Two views each shipping a
   private `row` widget will fight for `customElements.define('row',
   …)`. The convention prefixes view-private tags with the view name
   (`tasks-row`, `agents-card`) — see
   `references/4-layer-overview.md` § Layer 4 rule 5.

10. **Manifest rail vs `apps/shell/index.html` rail.** They are NOT
    auto-synced. § 6 above.

---

## 8. Verification

Run these before declaring the edit done. Each one fails fast on a
class of error the others miss.

```sh
# 1. Fast per-edit lint — grammar, missing bindings, unstable keys, purity:
cargo run -p sail-lint --example check -- views/<name>/page.sail

# 2. Full compile — lint folds in; aborts on errors:
cargo run -p sail-compiler -- build views/<name>/page.sail

# 3. Build all .sail sources + run lint + cargo test (proxy for CI):
just smoke-deploy
```

Then exercise the view in a real browser. `apps/shell/inventory-probe.mjs`
walks every registered view and screenshots each one:

```sh
just dev                                  # daemon + bundler + shell watch
node apps/shell/inventory-probe.mjs      # writes /tmp/shell-inventory/
cat /tmp/shell-inventory/report.json     # per-view: status, els, hasError, errors[]
ls  /tmp/shell-inventory/view_<name>.png # per-view screenshot
```

A healthy view in the probe report has `status === '200'`,
`errors.length === 0`, `hasError === false`, and either
`hasSkeleton === true` (live data still streaming) or
`els >= 35` (rendered content). `loadingText === true` after the
800 ms settle window means the view is stuck on `'Loading…'` text —
fix the live-data wire-up (§ 5, § 7-8).

Direct-mount path (skips shell's router):
`http://localhost:$PORT/sail/<name>` — confirms the view-host serves
the compiled bundle and SSR snapshot. Useful when bisecting
shell-routing bugs vs view-content bugs.

---

## Where to go next

| Question | File |
|---|---|
| Runtime symbol I'm using doesn't exist | `references/runtime-primitives.md` |
| Component vs view; what `<prop>` accepts | `references/authoring-patterns.md` |
| Where the compiler lives, what `view-host` serves | `references/4-layer-overview.md` |
| deck bundler / LSP / per-edit lint loop | `references/tooling.md` |
| Copy-paste-and-modify worked examples | `references/cookbook.md` |
