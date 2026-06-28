# @ship/sui-tutorial

The SUI tutorial, co-located with the framework so it can't go stale.

25 lessons, each a paired `starter/` (the reader's blank slate) and `solved/`
(the reference implementation). Every lesson is a complete `@ship/sui-runtime`
controller — a `controller.ts` paired with a `template.sui` — that the
marketing site (or any host) can mount directly.

The original source lives in `ship-marketing/apps/site/tutorial-examples/` and
is mirrored here. Edit the source there first, then re-mirror — or, better,
edit here, since this is the canonical home.

## Layout

```
src/lessons/
  01-hello-signal/
    starter/    # reader-facing scaffold
      controller.ts
      template.sui
    solved/     # reference impl
      controller.ts
      template.sui
  02-derived/
  …
  25-going-further/
```

Numbering matches the docs' lesson ordering; do not renumber without
updating the marketing site too.

## Use

A host (the marketing site, a docs page, an embedded playground) imports a
lesson by path and mounts it. The `.sui` template is a static asset at
runtime — only `controller.ts` runs through the host's bundler.

```ts
import controller from '@ship/sui-tutorial/lessons/01-hello-signal/solved/controller'
// mount via the host's runtime — see @ship/sui-runtime `mount()`.
```

The package does not ship a runtime of its own; it depends on
`@ship/sui-runtime@0.1.0` (published sibling) and uses the same
`controller({ scope, on, mount })` / `template.sui` contract as the
rest of the framework. (See "Scope type companion" below for the one
tutorial-specific deviation: the lesson `.ts` files parameterize
`controller<Scope>({...})` explicitly because the tutorial does not
run the sui-compiler and falls back to an ambient `*.sui` declaration
for portability. The canonical no-arg form is taught in the prose.)

## Scope type companion

Each `controller.ts` does

```ts
import type { Scope } from './template.sui'
```

The sui-compiler normally emits a `<stem>.sui.d.ts` adjacent to every
`.sui` (see `scripts/gen-component-types.mjs`). The tutorial is a docs
asset, not a component pack, and the compiler binary isn't required for
typecheck — `src/sui-ambient.d.ts` resolves `*.sui` to
`Scope = Record<string, unknown>` instead, which is precise enough for
the controllers' legacy `controller<Scope>(…)` parameterization — the
ambient `*.sui` declaration is too loose to satisfy the no-arg
`controller({…})` form's TS2742 portability requirement without
compiler-emitted `<stem>.sui.d.ts` files. The
`template.sui` is a SUI source file, not a TS module; it never executes
on the host.

## Scripts

- `bun run typecheck` — `tsc -p tsconfig.json --noEmit`

## Editing policy

Mirror from the marketing site is the upstream source of truth for the
tutorial's prose. The 25 lessons' `controller.ts` + `template.sui` pairs
are the single source of truth for the code. Don't re-author lesson
code in two places.
