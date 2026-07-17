---
group: Views
order: 4
title: SUI — Cookbook
description: Worked recipes for common SUI view tasks.
---

# SUI — Cookbook

Copy-paste-and-modify worked examples for an agent authoring a SUI
view. Every recipe is either lifted from a working view in this repo
(cited) or testable in isolation against `@ship/sail-runtime`.

Read `agent-authoring.md` first — these are the patterns; that file
explains the rules.

> **Projection in the recipes.** Several recipes fill component
> regions with `<el slot="region-name">…</el>`. The `slot="…"`
> attribute names the region; the runtime
> (`@ship/sail-runtime/slot-project.ts`) matches it to the
> corresponding `<slot name="…">` wrapper in the host's body at
> connect. The contract is the same as web-component slots, but
> the implementation is light-DOM (no shadow root, the slot
> wrapper is `display: contents`). See
> [template.md#projection](template.md#projection) for the full
> model; the recipes below rely on it.

---

## 1. Simple list view with live SSE

A typed list, the 3-branch loading/error/ok shape, an empty state.
Worked exemplar in the repo: `views/agents/page.sail` (template plus
its inline `<controller>` block).

`views/example-list/page.sail`:

```html
<data name="items" source="ship.agents.summaries()" live />

<div class="ship-view ex-list">
  <header class="ex-head">
    <h1>Items</h1>
    <p>{total()} item{total() === 1 ? '' : 's'}</p>
  </header>

  {#while-loading of items}
    <ul class="ex-list-rows" aria-label="Loading items">
      {#for n of [0,1,2,3] key={n}}
        <li><sail-skeleton shape="line" size="md"></sail-skeleton></li>
      {/for}
    </ul>
  {/while-loading}

  {#while-error err of items}
    <sail-error-state title="Couldn't load items">{err.message}</sail-error-state>
  {/while-error}

  {#if items().kind === 'ok'}
    {#if rows().length === 0}
      <sail-empty-state>
        <span slot="title">Nothing yet</span>
        Add the first one.
      </sail-empty-state>
    {:else}
      <ul class="ex-list-rows">
        {#for r of rows() key={r.id}}
          <li>{r.name}</li>
        {/for}
      </ul>
    {/if}
  {/if}
</div>
```

`views/example-list/page.sail`'s inline `<controller>` block:

```ts
<controller>
import { derived, unwrap } from '@ship/sail-runtime'

export default controller(({ items }) => {
  const rows  = derived(() => unwrap(items)?.items ?? [])
  const total = derived(() => unwrap(items)?.items.length ?? 0)
  return { rows, total }
})
</controller>
```

`manifest.jsonc`:

```jsonc
{ "name": "example-list", "path": "/example-list", "title": "Items",
  "layout": "default", "auth": "user",
  "rail": { "label": "Items", "icon": "list" } }
```

---

## 2. Form + mutation flow

User input → POST → optimistic UI → live patch reconciles. Worked
exemplar: `views/workspaces/page.sail` and its inline `<controller>` block.

`page.sail`:

```html
<data name="projects" source="ship.workspaces.list()" live />

<div class="ship-view ex-mut">
  {#if activationError()}
    <sail-error-state variant="inline" size="sm" title="Couldn't switch">
      {activationError()}
      <sail-button slot="action" size="sm" on:click={() => activationError.set('')}>Dismiss</sail-button>
    </sail-error-state>
  {/if}

  {#if projects().kind === 'ok'}
    <ul>
      {#for p of items() key={p.id}}
        <li data-active={effectiveActiveId() === p.id ? '1' : '0'}>
          <span>{p.label || p.name}</span>
          <sail-button
            prop:disabled={activatingId() === p.id || effectiveActiveId() === p.id}
            on:click={() => activate(p.id)}>
            {activatingId() === p.id ? 'Activating…' : effectiveActiveId() === p.id ? 'Active' : 'Activate'}
          </sail-button>
        </li>
      {/for}
    </ul>
  {/if}
</div>
```

Its inline `<controller>` block:

```ts
<controller>
import { signal, derived, unwrap } from '@ship/sail-runtime'

interface Row { id: string; name: string; label?: string }
interface State { items: Row[]; active: string | null }

export default controller(({ ship, projects }) => {
  const activatingId    = signal('')
  const activationError = signal('')
  const pendingActiveId = signal('')   // optimistic overlay

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const s = ship as any
  async function activate(id: string) {
    activatingId.set(id); activationError.set(''); pendingActiveId.set(id)
    try {
      await s.workspaces.activate({ id })
      // live <data> patch arrives; effectiveActiveId() flips to `id` on its own
    } catch (e) {
      pendingActiveId.set('')
      activationError.set((e as Error).message)
    } finally {
      activatingId.set('')
    }
  }

  const items = derived((): Row[] => (unwrap(projects) as State | undefined)?.items ?? [])
  const effectiveActiveId = derived((): string => {
    const live = (unwrap(projects) as State | undefined)?.active ?? ''
    const pending = pendingActiveId()
    return pending && pending !== live ? pending : live
  })

  return { activatingId, activationError, pendingActiveId, activate, items, effectiveActiveId }
})
</controller>
```

Declarative-form alternative (no optimistic overlay):

```html
<form mutation="ship.workspaces.activate">
  <input name="id" />
  <button type="submit" prop:disabled={$mutation().kind === 'loading'}>Switch</button>
</form>
```

`$mutation` is auto-injected into scope by the compiler when a
`<form mutation="…">` is present (see
`packages/sail/runtime/src/index.ts` `wireFormMutation` and
`references/runtime-primitives.md` § Mutations).

---

## 3. Master-detail with a selection signal

A list on the left drives a detail panel on the right. The selection
is a `Signal<string>`; the detail view subscribes to a live source
parameterized by it. Pattern from `views/skills/page.sail`'s inline
`<controller>` block, adapted for the typed-SDK path.

`page.sail`:

```html
<data name="rows" source="ship.agents.summaries()" live />
<data name="detail" source="ship.agents.detail({id: $scope.selectedId()})" live />

<div class="ex-md">
  <aside>
    {#while-loading of rows}<sail-skeleton shape="line"></sail-skeleton>{/while-loading}
    {#if rows().kind === 'ok'}
      <ul>
        {#for r of items() key={r.id}}
          <li class:selected={selectedId() === r.id}>
            <button on:click={() => selectedId.set(r.id)}>{r.name}</button>
          </li>
        {/for}
      </ul>
    {/if}
  </aside>

  <section>
    {#if !selectedId()}
      <sail-empty-state>Select an item from the list.</sail-empty-state>
    {:else}
      {#while-loading of detail}<sail-spinner></sail-spinner>{/while-loading}
      {#while-error err of detail}<sail-error-state>{err.message}</sail-error-state>{/while-error}
      {#if detail().kind === 'ok'}
        <h2>{detail().value.name}</h2>
        <pre>{JSON.stringify(detail().value, null, 2)}</pre>
      {/if}
    {/if}
  </section>
</div>
```

Its inline `<controller>` block:

```ts
<controller>
import { signal, derived, unwrap } from '@ship/sail-runtime'

export default controller(({ rows }) => {
  const selectedId = signal('')
  const items = derived(() => unwrap(rows)?.items ?? [])
  return { selectedId, items }
})
</controller>
```

The `<data live source="ship.agents.detail({id: $scope.selectedId()})">`
form re-subscribes whenever `selectedId` changes — the compiler tracks
deps inside the `source=` expression. See `references/runtime-primitives.md`
§ `<data>` reactive args.

---

## 4. Tabs / sub-routes

Two flavors. **In-page tabs** (no URL change) use a signal. **Sub-routes**
(URL changes; back button works) use `defineRoutes` + `route.params.<name>()`.

### 4a. In-page tabs

From `views/git/page.sail`'s inline `<controller>` block (`tab: signal<GitTab>('changes')`):

```html
<sail-segmented prop:value={tab()}>
  <sail-segmented-option value="changes" on:click={() => tab.set('changes')}>Changes</sail-segmented-option>
  <sail-segmented-option value="history" on:click={() => tab.set('history')}>History</sail-segmented-option>
</sail-segmented>

{#match tab()}
  {:case 'changes'}<section class="changes">…</section>
  {:case 'history'}<section class="history">…</section>
{/match}
```

```ts
<controller>
import { signal } from '@ship/sail-runtime'

type Tab = 'changes' | 'history'

export default controller(() => {
  const tab = signal<Tab>('changes')
  return { tab }
})
</controller>
```

### 4b. Sub-routes (parameter in URL)

Directory layout (mirrors `views/agents/`):

```
views/<name>/
  page.sail                  # /<name>          — list
  manifest.jsonc            # { path: "/<name>" }
  [id]/
    page.sail                # /<name>/[id]    — detail
    manifest.jsonc          # { path: "/<name>/[id]" }
```

`[id]/page.sail`'s inline `<controller>` block reads the param (pattern
from `views/agents/[id]/page.sail`):

```ts
<controller>
import { derived, live, unwrap } from '@ship/sail-runtime'

export default controller(({ route, ship }) => {
  const agentId = derived(() => route.params.id() ?? '')
  const ctx     = live(() => ship.agents.wire({ id: agentId() }))
  const data    = derived(() => unwrap(ctx) ?? null)
  return { agentId, ctx, data }
})
</controller>
```

If `defineRoutes` is needed (catalog-style detail views):

```ts
import { defineRoutes } from '@ship/sail-runtime/route'
// TOP-LEVEL — before the controller default export.
defineRoutes([{ pattern: '/<name>/:id' }])
```

Worked exemplar: `apps/shell/src/studio/components/[name]/page.sail`'s
inline `<controller>` block.

---

## 5. Conditional rendering branches

`{#match}` is the right tool when the discriminator is a small set of
known string tags; `{#if}/{:else if}/{:else}` is right for arbitrary
boolean predicates.

```html
{#match status()}
  {:case 'idle'}     <p>Idle</p>
  {:case 'running'}  <sail-spinner></sail-spinner>
  {:case 'completed'}<sail-icon name="check"></sail-icon>
  {:case 'errored'}  <sail-icon name="x" tone="destructive"></sail-icon>
  {:default}         <p>Unknown: {status()}</p>
{/match}
```

```html
{#if loading()}
  <sail-spinner></sail-spinner>
{:else if error()}
  <sail-error-state>{error()}</sail-error-state>
{:else if items().length === 0}
  <sail-empty-state>Nothing yet.</sail-empty-state>
{:else}
  <ul>{#for r of items() key={r.id}}<li>{r.name}</li>{/for}</ul>
{/if}
```

**Gotcha.** Branches inside `{#for}` had a leak where inline-updates
crossed branches (fix in `789d0dab`). If you find rows visually
swapping data after a re-key, update `@ship/sail-runtime` and
`sail-compiler` to the post-fix versions; do NOT work around it in the
template.

---

## 6. Skeleton + empty + error states (canonical shape)

Canonical skeleton → empty → error shape — the `{#while-loading}` →
`{#while-error}` → `{#if … 'ok'}` block. Same outer wrapper
across all three branches so the layout doesn't reflow when state
transitions.

```html
<data name="sessions" source="ship.sessions.list()" live />

<div class="ses-wrap">
  {#while-loading of sessions}
    <ul class="ses-list" data-test="loading" aria-label="Loading sessions">
      {#for n of [0,1,2,3] key={n}}
        <li class="ses-skel-row" aria-hidden="true">
          <div class="ses-skel-main">
            <sail-skeleton shape="circle" size="sm"></sail-skeleton>
            <sail-skeleton shape="line" size="sm" style="width:64px"></sail-skeleton>
            <sail-skeleton shape="line" size="sm" style="width:200px"></sail-skeleton>
          </div>
          <sail-skeleton shape="line" size="sm" style="width:56px"></sail-skeleton>
        </li>
      {/for}
    </ul>
  {/while-loading}

  {#while-error err of sessions}
    <sail-error-state title="Couldn't load sessions" data-test="error">
      {err.message}
      <sail-button slot="action" tone="primary" variant="solid"
        on:click={() => window.location.reload()}>Retry</sail-button>
    </sail-error-state>
  {/while-error}

  {#if sessions().kind === 'ok'}
    {#if items().length === 0}
      <sail-empty-state variant="default" size="lg" data-test="empty">
        <sail-icon slot="icon" name="terminal"></sail-icon>
        <span slot="title">No sessions running</span>
        Spawn a shell or start an agent chat to populate this list.
      </sail-empty-state>
    {:else}
      <ul class="ses-list" data-test="list" data-view-enter>
        {#for s of items() key={s.id}}
          <li class="ses-row" data-test="row" data-status={s.status}>
            <span>{s.kind}</span>
            <span>{s.status}</span>
          </li>
        {/for}
      </ul>
    {/if}
  {/if}
</div>
```

The `data-test="…"` hooks let the probe at
`apps/shell/inventory-probe.mjs:89-103` classify each view's state
without relying on text content (skeletons have no visible text — the
probe's `hasSkeleton` flag is true).

---

## 7. Mounting a custom element with props

Two flavors. **String literals** use bare attributes; **structured
data** (objects, arrays, signals) needs `prop:`.

```html
<!-- string literal — works as plain attribute -->
<sail-button variant="solid" tone="primary" size="sm">Save</sail-button>
<sail-button prop:variant={'solid'} prop:tone={'primary'}>Save</sail-button>

<!-- reactive props: re-evaluates on dep change -->
<sail-status-dot
  prop:tone={s.status === 'running' ? 'success' : s.status === 'exited' ? 'muted' : 'warning'}
  size="sm"
  label={s.status}></sail-status-dot>

<!-- structured props: array of rows, columns object, etc. -->
<sail-grid prop:rows={rows} prop:columns={columns()}></sail-grid>

<!-- prop:disabled — boolean prop. NOT `disabled={…}` — that's an attr,
     which only stringifies. -->
<sail-button prop:disabled={spawning()} on:click={spawnShell}>New shell</sail-button>

<!-- slots: explicitly named or default -->
<sail-button>
  <sail-icon slot="leading" name="plus"></sail-icon>
  New agent
</sail-button>
```

Why `prop:` exists: attribute serialization stringifies everything.
Passing an array of rows as `rows="[…]"` round-trips through
`JSON.parse` on read — slow, and loses signal identity. `prop:rows`
keeps the live reference; the component receives the same object the
parent has. See `references/authoring-patterns.md` § Passing complex
values into components for the full table.

To CREATE a new component (not just consume one), drop a pair under
`packages/components/ui-components/src/<tag>/`:

```sh
deck scaffold component my-widget --controller
```

The scaffold lays out:

```
packages/components/ui-components/src/my-widget/
  my-widget.sail   # <prop> directives + body with <slot> wrappers (light-DOM anatomy) + anatomy manifest;
                  # --controller adds an inline <controller> block to this file
  my-widget.css   # optional
```

Component-side `<prop>` syntax (full table in
`references/authoring-patterns.md` § `<prop>` directive):

```html
<prop name="value"    type="string" default="" />
<prop name="tone"     values="default|primary|danger" default="default" reflect />
<prop name="disabled" type="bool"   default="false" reflect />
<prop name="rows"     type="json"   default="[]" />

<!-- Light-DOM anatomy: the host is the element, the regions are
     declared in the body via <slot> wrappers, and the matching
     contract is published in the anatomy manifest. The runtime
     projects consumers' `slot="…"` children into the matching
     wrapper at connect. -->
<div class="my-widget" attr:data-tone={tone()} class:is-disabled={disabled()}>
  <slot name="leading"></slot>
  <slot></slot>
  <slot name="trailing"></slot>
</div>
```

Then consume from a view: `<my-widget tone="primary"
prop:rows={rowsSignal}>…</my-widget>`. The compiler emits the
`customElements.define('my-widget', …)` call — never hand-write it.

---

## 8. Declared clients — a view on a non-Ship host

A view names the data clients it consumes with `<client>`; the host
supplies a keyed map at mount. `ship` is just the key Ship hosts use —
here a deck host injects its own client as `api`. Worked exemplar in
the repo: `crates/deck/deck-conformance/app/views/clock/page.sail`.

`app/views/clock/page.sail`:

```html
<client name="api" />
<data name="t" source="api.demo.clock()" live />

<div class="clock-demo">
  {#while-loading of t}<p>Loading…</p>{/while-loading}
  {#while-error err of t}<p>Error: {err.message}</p>{/while-error}
  {#if t().kind === 'ok'}
    <p>Tick: <strong>{t().value.now}</strong></p>
  {/if}
</div>
```

The host's mount call (the deck view-host does this from the payload's
declared-clients list; a hand-rolled host writes it directly):

```ts
import { mount } from '@ship/sail-runtime'

const handle = await mount(target, payload, {
  clients: { api: myDataClient },
})
```

Typing — the provider augments its own key on the runtime's `Clients`
interface, so `api.demo.clock()` and the `t` signal are fully typed:

```ts
declare module '@ship/sail-runtime' {
  interface Clients { api: MyDataClient }
}
```

Declared names are checked: an undeclared root in a `<data source>` is
a compile error, and the generated `.sail.d.ts` types each data signal
from the client method's return. Views that declare nothing still get
the ambient `ship` (compatibility default); the `sail-client-directive`
lint (Warn) nudges those toward an explicit `<client name="ship" />`.
