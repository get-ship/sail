# `@ship/sail-l1`

Generated **consumer catalog** for Sail L1.

## What this is

A TypeScript projection of `crates/sail/compiler/stdlib`:

- components (directory = component = docs hub host)
- tags, props, axes, slots (accepts children)
- **concerns** each host's Style program spends (`border`, `fg`, `bg`, …)

Docs, Studio, and the matrix probe **import this package**. They do not copy `L1_CATALOG` or hand-author `contract.ts`.

## What this is not

| Not | Lives here |
|---|---|
| L1 component definitions | `crates/sail/compiler/stdlib/**/*.sail` |
| Queries | `@ship/sail-test` (`getByRole` / `getByText` / `click`) |
| Theme × mode × variant matrix | `crates/sail/compiler/tests/l1-matrix` + fixture `tests/fixtures/l1-matrix` |
| Behavioral edge cases | `crates/sail/compiler/stdlib/<tag>/<tag>.test.ts` |
| Theme token catalog | `@ship/tokens` |
| Composition recipes | `@ship/sail-blocks` |

Regenerate:

```
cargo run -p sail-compiler -- l1-surface
```

Authored sidecar (`src/copy.ts`) is blurbs and category only. Do not put axes or tokens there.

Theme Studio may later overlay the **concerns** listed here using the style language. Persistence of those overlays is parked. This package does not invent `component.<tag>.*` tokens.
