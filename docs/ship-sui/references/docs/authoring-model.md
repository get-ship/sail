---
group: Views
order: 1.5
title: The authoring model — pure-expression templates, controller logic
description: The line between a `.sui` template and its inline `<controller>` block. Templates carry declarative directives plus pure expressions; capabilities, statements, and declarations belong in the controller, reached through the ship SDK. SUI is the Svelte/Vue/Astro SFC lineage — deliberately not JSX.
---

# The authoring model

Every line you write in a view lands on one side of a single boundary: the
`.sui` template, or its inline `<controller>` block. Get the boundary right and the
rest of the framework — SSR, the LSP, route generation, the capability gate —
works for free. Get it wrong and you fight the renderer.

The boundary is **not** "heavy vs. light" or "much code vs. little code." A
200-line pure transform chain is fine in a template. The axis is **purity and
authority**, and it has two halves.

## The line: pure expressions, not capabilities

A `.sui` template expression must be a **pure function of scope signals** — safe
to re-run on every dependency change, and identical on the server and the
client.

**Allowed in `.sui`** — signal reads and pure transforms:

- `map` / `filter` / `reduce` / `slice` / `find` / `flatMap`
- arithmetic, string ops, `toFixed`, comparisons, ternaries
- member access (`u.name`, `overview().value?.branch`)

**Banned — promote to the `<controller>` block, reached through the ship SDK:**

| Category | Examples | Where it goes |
|---|---|---|
| I/O & authority | `fetch` · `XHR` · `WebSocket` · `eval` · `Function` · `import()` · `localStorage` | a `<data>` subscription or a controller `live` call / event handler |
| Non-determinism | `Date.now()` · `Math.random()` · `new Date()` | seed with a signal in the controller, assigned inside `mount` — or derive from data |
| Mutation of non-owned state | `arr.sort()` · `arr.reverse()` · `arr.splice()` | copy first: `[...arr].sort()`, inside a `derived` |

**Why the line is drawn here:**

1. The renderer **re-runs** every expression whenever a tracked dependency
   changes. An expression that fetches, mutates, or reads the clock is unsafe to
   re-run.
2. Authority flows **only** through the ship SDK — that is the auth gate. A
   template that reaches `fetch` directly has stepped around it.
3. Pure expressions **SSR identically** on the server and the client, so the
   server-rendered HTML and the client attach agree byte-for-byte.

The line is **enforced three ways**: a compile lint
(`controller_no_ambient_authority`, Error severity), a client SES sandbox
(`fetch`/`window` are `undefined` in template scope), and a server
deny-by-default loader. The template-subset lint — the pure-expression half made
explicit, with instant author feedback — is the same contract stated positively.

## Not JSX: expression-in-template, statement-in-controller

SUI is the Svelte/Vue/Astro single-file-component lineage, **deliberately not
JSX**. The split is by *kind of code*, not by *amount*:

- **Template (`.sui`):** declarative directives — `<prop>` / `<data>` /
  `<client>`, control flow `{#for}` / `{#if}` / `{#while-loading}` — plus pure
  expressions.
- **Controller (`.ts`):** full JavaScript — variable declarations, `signal` /
  `derived` / `live` / `effect`, and the auth-gated I/O.

There are **no `const`/`let`/statements in a `.sui` file.** Want a local
binding? **Promote it to a `derived`** in the controller. That promotion is not
a tax — the friction *is* the feature: it forces every named computation to be a
declared, typed, statically analyzable part of the view's scope.

Keeping the template to directives + expressions is what makes a view
**statically analyzable**: props, data dependencies, and routes are known
without running any JS, which is what powers the LSP, route generation, and prop
contracts. The cutover grows authoring power the right way — template
expressions compile to real TypeScript that `tsc` and the LSP type-check, **not**
by admitting statements into the template.

## Worked example — before / after

**Before** — the template reaches for I/O, a local `const`, the clock, and an
in-place sort. Every one of these belongs in the controller:

```html
<!-- ❌ page.sui — capabilities + statements in the template -->
{const res = fetch('/api/users')}                       <!-- I/O + declaration -->
{const sorted = res.items.sort((a, b) =>                <!-- declaration + non-owned mutation -->
  a.name.localeCompare(b.name))}
<ul>
  {#for u of sorted key={u.id}}
    <li>{u.name} · joined {Date.now()}</li>             <!-- non-determinism -->
  {/for}
</ul>
```

**After** — the data comes through a `<data>` subscription (the SDK is the auth
gate), the `const` becomes a `derived`, the sort copies first, and the template
holds only declarative directives and pure expressions:

```html
<!-- ✅ page.sui — declarative directives + pure expressions only -->
<data name="users" source="ship.users.list()" live />

{#if users().kind === 'loading'}
  <p class="muted">loading…</p>
{:else if users().kind === 'error'}
  <p class="err">{users().error.message}</p>
{:else}
  <ul>
    {#for u of sortedUsers() key={u.id}}
      <li>{u.name} · joined {u.joinedLabel}</li>
    {/for}
  </ul>
{/if}
```

```ts
<controller>
<!-- ✅ page.sui — inline <controller> block: declarations, logic, and the auth-gated read -->
import { derived } from '@ship/sui-runtime'

export default controller(({ users }) => {
  // The template's local `const` is now a declared scope member returned
  // by the controller. Copy before sort — never mutate the array the
  // signal owns. The "joined" label derives from data, not the wall clock.
  const sortedUsers = derived(() =>
    [...(users().value?.items ?? [])]
      .map((u) => ({ ...u, joinedLabel: new Date(u.joinedAt).toLocaleDateString() }))
      .sort((a, b) => a.name.localeCompare(b.name)))

  return { sortedUsers }
})
</controller>
```

The template still does real work — a `{#for}` over a sorted list, member
access, a loading/error/ready branch — but all of it is pure and re-run-safe.

## See also

- **[Architecture](./index.md)** — the three roles and the discipline this line
  enforces.
- **[The template surface](./template.md)** — every directive and expression a
  `.sui` file may hold.
- **[The controller surface](./controller.md)** — `signal` / `derived` / `live`
  / `on`, the scope, and mutation discipline.
- **ADR — SUI authoring model** (`docs/ship-adrs/references/docs/sui-authoring-model.md`)
  — the durable contract and its enforcement.
