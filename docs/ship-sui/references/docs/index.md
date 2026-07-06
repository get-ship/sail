---
group: Views
order: 1
title: Architecture
description: The three-role split, MVVM mapping, and the discipline that holds the framework's shape together. Read this before any other doc in this skill.
---

# Architecture

SUI is a single-file template DSL (`.sui`) compiled AOT to ESM that runs against a small signals runtime. Templates render server-side from `shipd`; a client runtime attaches reactive bindings, event handlers, and SSE subscriptions to the rendered DOM. Custom elements are the unit of composition; **light DOM is the default** — every `sui-*` renders into its own element with no shadow root, and components opt into a shadow boundary only when they wrap foreign content (declared in the manifest, paid for knowingly). The daemon is the data source.

## The three roles

Every authoring decision lives in exactly one of three roles:

| Role | File | Job |
|---|---|---|
| **Template** | `page.sui` | Structure + declarative wiring |
| **Controller** | `controller.ts` (optional) | Behavior — state, derived, handlers, lifecycle |
| **Data subscription** | `<data>` element in template | Bridge to the daemon's reactive surface |

The three do not overlap. There is no fourth role, and there will not be a fourth role until a real edge case forces a deliberate widening of the framework. Adding a fourth role to accommodate a one-off case is the easiest way to lose the coherence the three-role split buys.

## MVVM mapping

| MVVM | SUI | Notes |
|---|---|---|
| **Model** | The Ship daemon (`shipd`) | Out of band. Not a framework concept. The SDK on the other side of `<data>`. |
| **View** | The template (`.sui`) | Markup + declarative wiring. Per Vue/Svelte usage, "template" is more honest than "view" because it carries wiring, not pure structure — but it occupies the V slot. |
| **ViewModel** | The controller (`.ts`) | State, derived, handlers, composable hooks, lifecycle. The classic VM definition. |

Two things this mapping makes explicit:

1. The framework does not own the model. `shipd` is its own process; the SDK is the seam. `<data>` declarations are *bridges*, not models.
2. Templates carry wiring. `<data>`, `<use-*>`, `on:click`, `bind:value` are all view-side concerns even though they're declarative. Authors who treat `.sui` as pure markup will write controllers that try to take over wiring and end up in mud.

## The shared view scope

Each view has a single scope. Both the template and the controller contribute identifiers to it; both can read from it.

**Scope contributors**:

- Template `<data name="agents" src="..." live />` → `agents` as a live signal
- Controller `scope:` factory → returned signals / derived / hooks / shared providers (the **only** site for reactive component state)
- Controller `derived` section → declared keys as memos
- Controller `on` section → declared keys as handlers
- Controller `use:` section → declared keys as hook namespaces
- Template `<use-* as="X" />` → `X` as a hook-contributed identifier

**Scope consumers**:

- Template expressions (`{count() + 1}`, `{a.name}`, `{shell.theme}`)
- Controller `derived` sections (each is invoked with the scope as its argument)
- Controller `use:` sections (each hook factory receives the scope)
- Controller `on` handlers (each receives a state mutator + the scope)

The runtime calls each controller section with `{ ship, ...templateDeclaredSignals, ...stateSignals, ...derivedSignals, ...hookContributions }`. Each section destructures what it needs.

The mental model is **"this view has a scope; everything in this view contributes to it."** No imports between template and controller. No sharing rule beyond the scope. If you can see an identifier in the scope, you can read it.

## The discipline

Three rules that hold the architecture together. They are non-negotiable.

1. **Templates declare; controllers do.** Anything declarative (subscriptions, structure, wiring) goes in the template. Anything imperative (computation, async work, third-party libs) goes in the controller.
2. **Mutation is locked to handlers.** Templates and `derived.*` sections receive read-only views of state. Only `on.*` handlers can write. Statically enforced where possible, lint-enforced where not.
3. **Three roles, no fourth.** When an edge case arises, ask: does it fit one of the three roles, or does it fit a *combination* of two? If neither, it's a deliberate-widening question, not an accommodation question. The default answer is "express it with the three you have."

## What this is not

- **Not React.** No virtual DOM, no reconciler, no render functions. Templates compile to attach instructions over server-rendered DOM.
- **Not Vue.** No `<script setup>`, no SFC, no template-and-script-and-style in one file. Three sibling files (template + optional controller + optional CSS), paired by filename.
- **Not Svelte.** Similar template syntax; different reactivity model (no `$state` rune); different controller convention (no per-component `<script>` block); different SSR shape (Rust IR walker, not Node SSR).
- **Not Datastar.** Compatible philosophy (server-driven HTML + tiny client runtime + SSE), but SUI has its own template DSL, its own signals primitive, and its own marker shape. We do not ship Datastar's `data-*` attributes verbatim.
- **Not Hotwire/LiveView.** No server-pushed HTML fragments + idiomorph as the *only* update path. Local state is fully client-side; `data live` is the SSE channel for server-owned state. The two coexist.

## What's unique

Six edges SUI has over comparable frameworks:

1. **`data live` as a language verb.** Server-pushed reactivity is a first-class directive, not a library bolted on top.
2. **Single binary, no asset pipeline.** AOT compiler is invisible; `shipd` serves rendered HTML directly.
3. **Typed end-to-end.** Rust schema → TS types → template references → controller signatures. Same identity all the way through the stack.
4. **Agent-first ergonomics.** The framework is designed to be writable by an LLM. Buckets, naming, and rails exist partly so the eval rig has something to grade against.
5. **MCP-native.** Controllers can be exposed as MCP tools. A view becomes both a UI surface and an agent-callable surface.
6. **Mesh / multi-daemon.** When ship daemons mesh, controllers can subscribe to live signals from another daemon. Distribution is a framework primitive.

## See also

- **[The authoring model](./authoring-model/)** — the pure-expression-vs-capability line and the not-JSX split: what goes in `.sui` vs. `controller.ts`, and why.
- **[The template surface](./template/)** — directives, control flow, bindings, projection, expressions.
- **[The controller surface](./controller/)** — three tiers, scope rules, mutation discipline.
- **[Hooks](./hooks/)** — template vs controller hooks, when to reach for which.
- **[Data subscriptions](./data-subscriptions/)** — the canonical bridge to the daemon.
- **[Effect scopes](./effect-scope/)** — cleanup ownership on unmount.
- **[SSR and the attach runtime](./ssr-hydration/)** — server-rendered HTML, client attach.
- **[Canonical examples](./examples/)** — three view shapes that anchor the discipline.
- **[Agent surfaces](./agent-surfaces/)** — every MCP tool, LSP CLI subcommand, `@ship/sui-eval` entrypoint, and `ship://` resource an authoring agent uses.
- **[E2E test suite](./e2e-testing/)** — Playwright coverage and interpretation.
- **[Implementation status](./status/)** — what's stable, what's in flight.
