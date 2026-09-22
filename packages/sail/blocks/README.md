# `@ship/sail-blocks`

Composition product. **Not shipping** a marketplace. Organized as if we are.

Two layers live here:

| Layer | What | Where | Test |
|---|---|---|---|
| **Block** | One named composition. No page chrome. | `src/blocks/<kebab>/` | colocated sail-test (role / name) |
| **Template** | A page that *uses* blocks. | `routes/<id>/page.sail` | `test/<id>.test.ts` + Playwright |

If a composition needs `style()` on every L1 tag, the defaults are wrong or a primitive is missing. Host props (`tone`, `variant`, `bordered`) are fine.

## Folder contract (copy this)

```
src/blocks/<kebab>/<kebab>.sail    the composition (defaults only). Stem is the tag.
src/blocks/<kebab>/<kebab>.test.ts @ship/sail-test (role / name / state)
src/catalog.ts                     one row: kind: 'block', id stays PascalCase (Mantine)
routes/lab/<Name>/page.sail        canvas — mounts the tag. No Style on the route.
routes/<id>/                       templates only
```

Catalog `id` stays PascalCase (`InputWithButton`, `FaqSimple`) so the Mantine name is stable. The file stem is kebab (`input-with-button`) because a custom-element tag must contain a hyphen. Lab is Privilege None: it mounts `<faq-simple>`, it does not fork the body.

## Add the next block

1. Record headings, buttons, links, fields (role / name). That list is the test.
2. Copy `src/blocks/input-with-button/`. Rename the folder and both files to the kebab tag.
3. Author `<kebab>.sail` with L1 defaults. No navbar/footer. No `data-*`. No `component.*`.
4. Colocate `<kebab>.test.ts`. Locate UI via `@ship/sail-test`. Assert the catalog row. Assert no per-tag `style()` on L1.
5. In `src/catalog.ts`, flip that row to `status: 'authored'` and set `primitives` to the L1 hosts you actually used.
6. Lab canvas mounts the tag. Playwright visits `/lab/<PascalId>`. Do not claim a browser gate from the unit test alone.

```
bun test                          # this package; includes src/blocks/**/*.test.ts
bunx playwright test              # templates + every authored lab
```

Docs composition and takumi consume `src/catalog.ts`. Do not fork a second list into sail-site.

## Law

- Authored in `.sail`.
- Defaults only — host props are fine. Per-tag `style()` on L1 is a finding.
- Tests locate UI via `@ship/sail-test` (role / name / state). No `data-*`, no class atoms.
- New **block** = new `src/blocks/<Name>/` + catalog row + colocated test + lab.
- New **template** = new `routes/<id>/` + catalog row + `test/<id>.test.ts`. Do not add templates until the first-20 blocks exist.
