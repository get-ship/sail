# @ship/sail-tutorial

The SUI tutorial, co-located with the framework so it can't go stale.

Nine lessons, each a paired `starter/` (the reader's working copy) and
`solved/` (the reference implementation). Every lesson is a real, small
screen written on the current language surface — compiled by the in-repo
`sail-compiler`, mounted live by the docs site.

## Why nine

One lesson per decision surface the language actually kept, plus the
loop the language exists for:

| # | lesson | decision surface |
|---|--------|------------------|
| 01 | a-page-is-markup | structure = layout keywords + component props (zero JS) |
| 02 | styling-is-sx | the one styling attribute; closed token vocabulary; states as prefixes |
| 03 | state-is-a-controller | signal / derived / effect; one owner for state |
| 04 | templates-react | zero-arg reads, modifiers, `{#if}` / `{#for key}` / `{#match}` |
| 05 | data-and-forms | `live()`'s three explicit states; `bind:`; refetch = bump a tracked signal |
| 06 | motion-is-a-directive | triggers × presets; exit-hold; stagger; reduced motion |
| 07 | themes-are-one-attribute | tokens + the tone contract; `applyTheme`; `data-theme-preview` |
| 08 | the-compiler-talks-back | the agent loop: run, read the rejection, fix, run again |
| 09 | review-like-an-owner | the human loop: five flaws that compile, and the checklist that catches them |

08 and 09 are the north star spelled out: **agents author** (the
compiler's closed vocabulary makes their output reviewable-by-build)
and **humans review** (intent — the part no lint can gate).

## Layout

```
src/lessons/
  01-a-page-is-markup/
    starter/    # the reader's working copy (lesson 08's starter fails the build on purpose)
      tut-01-settings-page-starter.sui
    solved/     # reference impl — mounted live by the docs site
      tut-01-settings-page.sui
  03-state-is-a-controller/
    starter/
      tut-03-stopwatch-starter.sui   # inline <controller> block, no sibling file
    solved/
      tut-03-stopwatch.sui           # inline <controller> block, no sibling file
  …
src/prose/
  01-a-page-is-markup.md    # flat files — frontmatter: title/description/order/slug/prev/next/chapter
  …
  09-review-like-an-owner.md
```

Naming is the custom-element tag rule: a component `.sui`'s stem IS its
tag (`tut-03-stopwatch` → `<tut-03-stopwatch>`), so stems are
hyphenated, unique across the workspace, and prefixed `tut-`. Starter
stems carry a `-starter` suffix — two files may not claim one tag. A
lesson has an inline `<controller>` block only when it has state;
lessons 01 and 02 are deliberately zero-JS. There is no sibling
`.controller.ts` file — a controller is always authored inline in the
`.sui` file, and a sibling `<stem>.controller.ts` is a compile error.

## Compile check

Every template must compile standalone:

```
cargo build -q -p sail-compiler
target/debug/sail-compiler build --mode component <lesson>.sui
```

The one exception is `08-the-compiler-talks-back/starter/` — that file
FAILS the build by design (four planted defects); the lesson's prose
walks the fix loop with the compiler's verbatim output. If compiler
error text changes shape, re-run the loop and refresh the prose.

## Use

The docs site (`apps/sail-site`) mounts each solved lesson as a custom
element — the deck bundler resolves any `<tut-*>` tag used in a route
to its source `.sui` here and compiles it into the view bundle. Prose +
lesson sources are baked into the site by
`apps/sail-site/scripts/gen-tutorial.mjs`, which emits
`apps/sail-site/src/tutorial-data.ts`; regenerate after editing lessons
or prose:

```
cd apps/sail-site && node scripts/gen-tutorial.mjs
```

## Scope type companion

Each controller does `import type { Scope } from './<stem>.sui'`,
resolved by `src/sail-ambient.d.ts` (`Scope = Record<string, unknown>`)
so `typecheck` needs no compiler run; the lesson controllers that need
the type annotate their setup param `(ctx: Scope) => {…}` explicitly.
The one setup-function idiom — `controller((ctx) => {…})`, with
`Ctx`/`SC` inferred and a real compiler-emitted `.sui.d.ts` — is what
the prose teaches for real projects.

## Scripts

- `bun run typecheck` — `tsc -p tsconfig.json --noEmit`

## Editing policy

The lesson `.sui` files (inline `<controller>` block and all) are the
single source of truth for the code; `src/prose/*.md` for the words.
The site's `tutorial-data.ts` is generated — never edit it by hand.
