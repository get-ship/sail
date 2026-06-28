---
group: Views
order: 11
title: Routing
description: Manifest paths, dynamic routes with defineRoutes and route.params, SPA navigation via navigate(), and the in-page-tabs vs sub-routes split.
---

# Routing

A view's URL is declared in its `manifest.jsonc`; dynamic segments are read
through the reactive `route` handle. The runtime owns the *mechanism* (the
`route.*` signals); the host (the shell, the view-host) owns the *policy*
(history, path→view matching, mounting).

## The manifest path

Every view directory carries a `manifest.jsonc` whose `path` is the route the
host serves it at:

```jsonc
// views/git/manifest.jsonc
{
  "name": "git",
  "path": "/git",
  "title": "Git",
  "layout": "default",
  "auth": "user",
  "rail": { "label": "Git", "icon": "git-branch" }
}
```

Dynamic segments use a bracketed directory stem. The manifest `path` mirrors the
directory; the runtime pattern (next section) uses the `:name` form:

```jsonc
// apps/shell/src/studio/components/[name]/manifest.jsonc
{
  "name": "components-detail",
  "path": "/studio/components/[name]",
  "title": "Component",
  "layout": "default",
  "auth": "user"
}
```

## Dynamic routes — `defineRoutes` + `route.params`

`route` is a `RouteSignals` injected into every controller section's scope.
`route.params.<name>` and `route.query.<name>` are lazy `Signal<string |
undefined>`s; reading one (`route.params.id()`) inside a `derived`, an `effect`,
or a `<data live>` source thunk subscribes the surrounding scope. When
navigation changes the URL, the matched param signals fire and dependents re-run
— the prior SSE subscription disposes and a new one opens with the updated arg.

Param names are not declared at construction. They are matched against the
patterns registered via `defineRoutes`. A view that reads `route.params.id()`
**must** register the pattern that captures `id`, or the signal stays
`undefined` until a future navigation introduces a matching segment
(`/home/madvib/dev/Ship/sui/packages/sui/runtime/src/route/index.ts:98`, `:237`).

```ts
// views/agents/[id]/controller.ts
import { controller, derived, live, unwrap } from '@ship/sui-runtime'
import { defineRoutes } from '@ship/sui-runtime/route'
import type { Scope } from './page.sui'

// TOP-LEVEL — before the controller default export.
defineRoutes([{ pattern: '/agents/:id' }])

export default controller<Scope>({
  scope: ({ route, ship }) => {
    const id   = derived(() => route.params.id() ?? '')
    const ctx  = live(() => ship.agents.detail({ id: id() }))
    const data = derived(() => unwrap(ctx) ?? null)
    return { id, ctx, data }
  },
})
```

The single-source-of-truth example in the shell registers two patterns at once
— the shell route and the view-host standalone route — so the same controller
works popped out of the dashboard
(`apps/shell/src/studio/components/[name]/controller.ts`):

```ts
import { DECK_PREFIX } from '@ship/sdk-client'
defineRoutes([
  { pattern: '/studio/components/:name' },
  { pattern: `${DECK_PREFIX}/view/studio/components/:name` },
])
```

> **Hard rule.** Per-controller `defineRoutes` only works because every bundle
> shares the single `@ship/sui-runtime` instance. The shape registry and the
> global URL updater both live on `globalThis`
> (`/home/madvib/dev/Ship/sui/packages/sui/runtime/src/route/index.ts:72`, `:309`). A second runtime copy
> would register into a different proxy the shell never reads.

`:param` segments compile to `([^/]+)` and are captured by name
(`route/index.ts:218`). Query params come from `URL.searchParams` and need no
registration — `route.query.q()` works for any key.

## SPA navigation — `navigate()`

`navigate(path)` is the single entry point for changing the URL. It delegates to
the host's registered `RouterPort` when one exists (the shell owns matching +
mounting, so the view actually switches); without a port — a single-view host or
a test — it updates history and the `route.*` signals in place
(`/home/madvib/dev/Ship/sui/packages/sui/runtime/src/route/index.ts:192`).

```ts
import { navigate } from '@ship/sui-runtime/route'

navigate('/agents/42')
navigate('/login', { replace: true })   // replaceState instead of pushState
```

`navigate` and the browser back/forward `popstate` listener both fire the same
global updater, so every attached view re-evaluates from one navigation
(`route/index.ts:317`). Writing to `route.params` or `route.query` directly
throws — change the URL through `navigate`.

In a controller, `navigate` is also available on the handler argument:

```ts
on: {
  newAgent: ({ navigate }) => navigate('/agents/create'),
}
```

## Sub-routes vs in-page tabs

Two patterns, chosen by whether the URL should change.

**In-page tabs** (no URL change, no back-button history) use a signal:

```ts
// views/git/controller.ts
scope: () => ({ tab: signal<'changes' | 'history'>('changes') })
```

```html
<sui-segmented prop:value={tab()}>
  <sui-segmented-option value="changes" on:click={() => tab.set('changes')}>Changes</sui-segmented-option>
  <sui-segmented-option value="history" on:click={() => tab.set('history')}>History</sui-segmented-option>
</sui-segmented>

{#match tab()}
  {:case 'changes'}<section class="changes">…</section>
  {:case 'history'}<section class="history">…</section>
{/match}
```

**Sub-routes** (URL changes, back button works) use a bracketed child directory
plus `defineRoutes`:

```
views/<name>/
  page.sui                  # /<name>        — list
  manifest.jsonc            # { path: "/<name>" }
  [id]/
    page.sui                # /<name>/[id]  — detail
    manifest.jsonc          # { path: "/<name>/[id]" }
```

The `[id]/controller.ts` registers `'/<name>/:id'` and reads
`route.params.id()`, exactly as the dynamic-route example above. Worked
exemplar: `apps/shell/src/studio/components/[name]/controller.ts`.
