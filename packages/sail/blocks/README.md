# `@ship/sail-blocks`

Composition product. **Not shipping** a marketplace. Organized as if we are.

A **block** is a page-shaped composition of L1 atoms (dashboard, pricing, settings, auth). It is the test that defaults work together. If a block needs `style()` on every tag, the defaults are wrong or a primitive is missing.

## Layout

```
routes/<id>/page.sail     the block (defaults only)
src/catalog.ts            inventory + intended primitives
test/<id>.test.ts         @ship/sail-test (role / name / state)
```

Docs composition page and takumi consume these same files. Do not fork a second dashboard into sail-site.

## Law

- Authored in `.sail`.
- Defaults only — host props are fine (`bordered`, `tone="info"`). Per-tag `style()` is a finding.
- Tests locate UI via `@ship/sail-test` (role / name / state). No `data-*`, no class atoms.
- New block = new `routes/<id>/` + catalog row + test. Review is a batch across blocks.

## Intended set (not all authored)

| id | Status | Tests |
|---|---|---|
| dashboard | authored | yes |
| pricing | planned | — |
| settings | planned | — |
| auth | planned | — |
| empty | planned | — |
