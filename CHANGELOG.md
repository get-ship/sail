# Changelog

## 0.3.0 — Verdaccio stamp (2026-07-26)

Internal registry cut for `job/style-synthesis`.

**Published to** `http://localhost:4873` as `@ship/sail-*@0.3.0` and related packs:

- `@ship/tokens@0.3.0`
- `@ship/sail-compiler-wasm`, `sail-runtime`, `sail-dom`, `sail-lsp-browser`, `sail-tutorial`
- `@ship/sail-primitives`, `sail-canvas`, `sail-charts`, `sail-mermaid`, `sail-chat-elements`, `sail-media`, `sail-code-editor`, `sail-viewer`
- `@ship/deck-router@0.3.0`, `@ship/deck-host@0.3.0`

**Not republished:** legacy `@ship/sui-*` (marketing/helm/ship-apps stay on sui until explicit cutover — visual safety).

**Language gaps / non-language surface:** see `apps/sail-site/qa/language-gaps.md`.

## Unreleased — `job/style-synthesis` (2026-07-26)

Authoring and host-surface freeze for Sail. Apps still on SUI (`.sui`, `@ship/sui-*`, brace control-flow, `ctx.clients` / `ship`) need the migrations below when pointing at this tip.

### Breaking

**Control flow is elements, not brace blocks**

```sail
<!-- before -->
{#if open()}…{:else if x()}…{:else}…{/if}
{#for item of items() key={item.id}}…{/for}

<!-- after -->
<if cond={open()}>…<else-if cond={x()}>…</else-if><else>…</else></if>
<for as="item" of={items()} key={item.id}>…</for>
```

Also: `<match on={…}>` / `<case of={…}>` / `<default>`, `<while-loading of="sig">`, `<while-error as="err" of="sig">`.  
Brace `{#if}` / `{:else}` / `{/if}` compile-error with a migration hint.  
Codemod: `scripts/migrate-control-flow.py` (works on `.sail`; rename `.sui`→`.sail` first if needed).

**Public `fx:` removed**

Motion attaches via Style chain methods, same config body as the old directive:

```sail
sx={style.enter({ opacity: 'fade' }).exit({ opacity: 'fade-out' }).loop({ scale: 'pulse' })}
```

Triggers as methods: `.enter` / `.exit` / `.loop` / `.reveal` / `.scroll` / `.press` / `.hover` / `.focus` / `.focus-visible`.  
Runtime attach IR unchanged (exit-hold, stagger, reduced-motion).

**`data()` removed**

Reads are `query()` / `queryKeyed()`; writes `mutation()`. SSE LiveSource still works through query.

**Host inject is flat — no `ship`, no `clients` bag**

```ts
declare module '@ship/sail-runtime' {
  interface Host { ops: Ops; api: Api }
}

mount(el, payload, { ops, api, route, navigate })
// controller: ctx.ops…  template: bare ops… if host spreads providers
<form mutation="ops.createThing">
```

- `ctx.ship` / `opts.ship` / `ctx.clients` / `opts.clients` gone  
- Mutation path is `<provider>.<method>` (no `ship.` prefix)  
- Types: augment `interface Host { … }`, not `Clients`

**Package / file renames (SUI → Sail)**

| Before | After |
|--------|--------|
| `@ship/sui-runtime` | `@ship/sail-runtime` |
| `@ship/sui-dom` | `@ship/sail-dom` |
| `@ship/sui-primitives` etc. | `@ship/sail-primitives` etc. |
| `.sui` sources | `.sail` |
| `SUI_REPO` → deck | this monorepo tip (`orch` / `sui` worktree) |

**Public runtime export pruned**

Language surface on `@ship/sail-runtime`. Compiler ABI (`bindFor`, `runAction`, `__tpl`, …) only on `@ship/sail-runtime/internal`.

**Platform CSS**

- Document floor asset: `sail-base.css` (`contracts.css` deprecated alias)  
- Production CSS minify (lightningcss) on deck sail.css / asset CSS  

### Not breaking (but know)

- L1 still stamps host classes for CSS; actions discover via `data-sail-host`  
- Ambient PE: `style.animate(spin)` still CSS keyframe catalog (rename later)  
- Bundle: L1 library delta ~**7.3 kB gzip** (was stale 1.3 kB claim)  
- Public `use:` catalog remains 12 names; stdlib stamps more internally  

### Upgrade checklist (polyrepo)

1. Point deck/compiler at this tip (`SUI_REPO` / path overrides → orch worktree).  
2. Rename deps `@ship/sui-*` → `@ship/sail-*`.  
3. Rename sources `.sui` → `.sail` (and any imports/paths).  
4. Run control-flow codemod.  
5. Replace `fx:…` with Style motion methods.  
6. Flatten inject: `ctx.clients.X` → `ctx.X` / destructure from setup; `mutation="ship…"` → `mutation="client.method"`.  
7. Remove `data()`; use `query`.  
8. Rebuild; fix compile errors (compiler messages name the legal surface).  

### Commits (selected, this branch)

- `a90b4df4` / `eced9872` — element control-flow  
- `a4afcc42` — flat host inject  
- `5d60b355` — Style motion; retire public `fx:`  
- `f1906813` — remove `data()`  
- `875a6c04` — public export prune  
- `5fe0e2a5` — sail-base + CSS minify  
- `93cbff02` — purge ship-set branding  
- `b1483b76` — bundle-cost harness  
- `f3a5ba51` — `data-sail-host` for actions  
