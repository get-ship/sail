---
group: Views
order: 13
title: Authoring a component pack
description: deck scaffold pack, the package.json/vitest conventions, the compile gate, the grouped-directory convention, CEM emission, and design-token rules.
---

# Authoring a component pack

A pack is a workspace package that registers a family of `<sui-*>` (or
`<your-prefix-*>`) custom elements. A single `import '@ship/<name>'` at a
consumer pulls in every tag the pack defines. This page covers scaffolding one,
the build gate each component passes, and the token rules its CSS must follow.

## Scaffold — `deck scaffold pack`

```sh
deck scaffold pack <name>
```

`<name>` is lowercase + hyphens and becomes `@ship/<name>`. The scaffold writes,
under `packages/components/<name>/`
(`crates/deck/deck-cli/src/scaffold.rs:225`):

- `package.json` — `@ship/<name>`, `type: module`, `main`/`types` →
  `src/index.ts`, `workspace:*` deps on `@ship/sui-dom` + `@ship/sui-runtime`,
  a `vitest` devDependency, and `test` / `test:watch` scripts.
- `tsconfig.json` — strict, with a `rootDirs` entry pointing at the compiler's
  mirror tree (`.ship/sui-out/packages/components/<name>/src`) so `tsc` resolves
  imports from the generated `.sui.d.ts` files.
- `src/index.ts` — the umbrella. Side-effect-imports each component's
  `.compiled.js` so importing the package registers every tag globally.
- `src/<name>-example/<name>-example.sui` + `.controller.ts` — a starter
  component to replace.

After scaffolding, run `bun install` to wire the new workspace package.

> The CLI scaffolds `package.json`, `tsconfig.json`, the umbrella, and the
> example component. It does not emit a `sui.config.ts` or `vitest.config.ts` —
> add a `vitest.config.ts` yourself (next section) when you write tests.

## The grouped-directory convention

One component per directory; the directory stem is the component's hyphenated
tag, and the `.sui` filename matches it. A pack's `src/` is a flat list of these
directories:

```
packages/components/sui/primitives/src/
  sui-button/
    sui-button.sui
    sui-button.controller.ts   (optional)
    sui-button.css             (optional)
  sui-card/
    sui-card.sui
  index.ts                     (umbrella: side-effect imports each .compiled.js)
```

The hyphenated stem is the custom-element tag — `sui-button.sui` compiles to the
`<sui-button>` element. Components carry no `manifest.jsonc` (that is a view-only
file).

## The compile gate — `sui-compiler build --component-name`

A `.sui` is compiled in component mode by passing `--component-name <tag>`,
which implies component mode (`crates/sui/compiler/src/bin/sui-compiler.rs:104`):

```sh
sui-compiler build packages/components/<name>/src/<tag>/<tag>.sui \
  --component-name <tag> \
  --styles packages/components/<name>/src/<tag>/<tag>.css
```

Without `--out`, outputs land in the mirror tree at
`.ship/sui-out/<source-rel>/` and the module is named `<tag>.compiled.js`
(`sui-compiler.rs:71`). `--styles` prepends a CSS file as a `<style>` block and
is only valid for a single component file. In the monorepo this is driven for
you by `just build` (`_sui-compiler-lib` + the per-pack rebuild) — author tags,
run the build, and the umbrella picks them up.

## CEM emission

Every component build also emits a Custom Elements Manifest beside the module:
`<tag>.custom-elements.json` (`crates/sui/compiler/src/bin/sui-compiler.rs:364`).
It is generated from the component's declared `<prop>`s and serves tooling — the
component viewer's prop knobs, catalog generation, IDE metadata. You don't write
it; declare props honestly in the template and the manifest follows.

## Vitest — `suiComponentVitestConfig`

Component tests use one factory rather than a hand-rolled config. Add a
`vitest.config.ts`:

```ts
// packages/components/<name>/vitest.config.ts
import { suiComponentVitestConfig } from '@ship/test-setup/vitest'

export default suiComponentVitestConfig({ environment: 'happy-dom' })
```

The factory (`packages/test-setup/src/vitest.ts`) is location-independent: it
walks up to the `ship-monorepo` root and resolves everything by package name or
absolute path, so a pack can move anywhere under `packages/` with no config
edit. It compiles `.sui` sources through the repo's `sui-compiler` binary into
the `.ship/sui-out` mirror, caps fork count (each happy-dom/jsdom fork holds
~500 MB), and prepends an honesty guard (`expect.hasAssertions()`) so a test
that asserts nothing fails instead of passing green. Real consumers:
`packages/components/sui/{viewer,charts,chat-elements}/vitest.config.ts`.

## Design-token rules

A pack's CSS consumes the Ship token API; it never hardcodes color, depth, or
motion. The canonical tokens live in `packages/sui/tokens/src/tokens.css`, and
themes override them in `packages/sui/tokens/src/themes.css`. Read the **highest
layer that fits**:

| Layer | Examples | Use for |
|---|---|---|
| 1. Semantic surfaces & intents | `--surface-0..3`, `--backdrop`, `--hover-bg`, `--selected-bg` | Depth hierarchy, interaction feel |
| 2. Tone ramps | `--tone-{success,warning,destructive,…}-{fg,bg,border,ring}` | Semantic status color |
| 3. Role tokens | `--heading-font`, `--body-font`, `--ease-out`, `--shadow-hover`, `--density-row-py` | Typography, motion, density, shadow intent |
| 4. Primitives | `--space-1..8`, `--radius-sm..xl`, `--text-xs..4xl`, `--duration-fast` | Raw scale — rarely the right override point |

Anti-smells (the compiler rejects literals in themed properties; the rest are
enforced by review):

- **No raw color in shipped CSS** — no hex, `rgba(…)`, or bare `oklch(0 0 0 / …)`.
  Ship is OKLCH-native; use `var(--backdrop)` / `var(--scrim)` for dims.
- **No `color-mix` for tones** — read `var(--tone-success-bg)`; the formula lives
  in `tokens.css` once.
- **Prefer the surface layer over the palette** — reach for `var(--surface-1)`
  before `var(--card)` so a theme that raises card depth only touches surfaces.
- **No hardcoded font families** — read `--heading-font` / `--body-font` /
  `--label-font` / `--mono-font`; a theme like `paper` remaps them.
- **No `var(--token, fallback)`** — tokens are guaranteed loaded at the document
  root; a fallback creates a parallel, stale system.

Full authoring detail lives in the `ship-design-system` skill; this is the
public-relevant subset for pack authors.
