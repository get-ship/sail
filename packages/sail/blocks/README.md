# `@ship/sail-blocks`

Composition product. **Not shipping** a marketplace. Organized as if we are.

Two layers live here:

| Layer | What | Where | Test |
|---|---|---|---|
| **Block** | One named Mantine-style composition. No page chrome. | `src/blocks/<Name>/` | colocated sail-test (role / name) |
| **Template** | A page that *uses* blocks. | `routes/<id>/page.sail` | `test/<id>.test.ts` + Playwright |

If a composition needs `style()` on every L1 tag, the defaults are wrong or a primitive is missing. Host props (`tone`, `variant`, `bordered`) are fine.

## Folder contract (copy this)

```
src/blocks/<Name>/<Name>.sail      the composition (defaults only)
src/blocks/<Name>/<Name>.test.ts   @ship/sail-test (role / name / state)
src/catalog.ts                     one row: kind: 'block', status, mantineUrl, primitives
routes/lab/<Name>/page.sail        optional canvas — mount, do not market
routes/<id>/                       templates only
```

`<Name>` is PascalCase and **is** the catalog `id` (`InputWithButton`, `FaqSimple`). Templates keep kebab ids (`dashboard`) matching their route.

A `.sail` stem without a hyphen compiles as a view fragment (`renderRoute`), not a custom element. Do not rename the file to invent a tag. Do not `sail-compiler build` the block file as a CE — the path looks like `<tag>/<tag>.sail` but `InputWithButton` is not a custom-element name.

## Add the next block (Wave 5b)

1. Open the Mantine URL. Record headings, buttons, links, fields (role / name). That list is the test.
2. Copy `src/blocks/InputWithButton/`. Rename the folder and both files to `<Name>`.
3. Author `<Name>.sail` with L1 defaults. No navbar/footer. No `data-*`. No `component.*`.
4. Colocate `<Name>.test.ts`. Locate UI via `@ship/sail-test`. Assert the catalog row. Assert no per-tag `style()` on L1.
5. In `src/catalog.ts`, flip that row to `status: 'authored'` and set `primitives` to the L1 hosts you actually used.
6. Optional canvas: `routes/lab/<Name>/page.sail` — pad + the same composition. Not a marketing page.
7. Playwright the lab (or the Mantine URL vs this file). Do not claim a browser gate from the unit test alone.

```
bun test                          # this package; includes src/blocks/**/*.test.ts
bunx playwright test              # templates + lab; reuses :7893
bun e2e/probe.ts                  # visual evidence under web-qa/blocks/
```

Docs composition and takumi consume `src/catalog.ts`. Do not fork a second list into sail-site.

## Law

- Authored in `.sail`.
- Defaults only — host props are fine. Per-tag `style()` on L1 is a finding.
- Tests locate UI via `@ship/sail-test` (role / name / state). No `data-*`, no class atoms.
- New **block** = new `src/blocks/<Name>/` + catalog row + colocated test.
- New **template** = new `routes/<id>/` + catalog row + `test/<id>.test.ts`. Do not add templates until the first-20 blocks exist.

## Inventory

| id | Kind | Status | Tests |
|---|---|---|---|
| dashboard | template | authored | yes |
| pricing | template | authored | yes |
| settings | template | authored | yes |
| auth | template | planned | — |
| empty | template | planned | — |
| InputWithButton | block | authored | yes |
| AuthenticationTitle, CardWithStats, FaqSimple, FeaturesCards, FooterLinks, GetInTouchSimple, HeaderSimple, HeroText, HeroTitle, NavbarMinimal, StatsGroup, UserButton | block | authored | yes |
| *(remaining first-20 names)* | block | planned | — |

The full first-20 list, with `mantineUrl`, is in `src/catalog.ts`.
