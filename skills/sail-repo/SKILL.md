---
name: sail-repo
description: >
  Workplace rules for the Sail monorepo. Use on any task in this repo —
  compile, test, hygiene, emit, scripts, fixtures, agents.md, conventions.
  Triggers: /sail-repo, "where does this go", "can I add a script",
  output homes, test locations, fixtures, emitters, agent antipatterns.
---

# Sail repo — agent rules

Gates beat sermons. If a rule can be a test, write the test.
`tests/hygiene/emitters.json` and `fixture_law.rs` are the living lists.
This file is for the smells tests have not caught yet.

## Output homes (do not invent a fourth)

| Home | What | Who writes |
|---|---|---|
| `<project>/.deck/` | App SPA / deck build | `sail build` / `sail dev` |
| `<repo>/.deck/<src-rel>/` | Compiler mirror | `sail-compiler build` (no `--out`) |
| `<project>/node_modules/.sail/` | Types, kind tsconfigs, `test-map` | `sail types` / `sail test` / build |
| `<project>/.sail/` | Ops **task store** (gitignored). Not types. Not Studio. | `sail op` / `ensure_project_store` |
| Adjacent `*.sail.d.ts` | Type companion | compiler |

Forbidden as product out: `.sail-out`, root `.sail`, root `tsconfig.json`, root `test-results`. Hygiene fails if they reappear.

`node_modules/.sail` is **not** `.sail/`. Do not document them as one home.

Repo root is not a Sail project. `sail types` without `--project` must refuse.

## Generated files

Every tracked generated file has a row in `tests/hygiene/emitters.json`
(path, emitter, gate). New `*.gen.ts` without a row is a third state — fail.

Untrack **or** gate. Never "committed and stale."

Do not run kitchen-sink `cd apps/sail-site && bun run gen` to fix one dump.
Use the named emitter. `--check` before write when the script has it.

`prop-registry` / `language-surface.gen.ts` / `primitives.json` /
`element-skins.css` are stdlib-coupled. Do not regen them while
`job/release-integrate` is moving stdlib.

## Scripts

A one-off script that emits product is a sin. Prefer a `sail` /
`sail-compiler` verb.

Allowed in `scripts/` and `apps/*/scripts/`: thin wrappers around an
existing verb, or `--check` until the fold lands.

`apps/sail-site/scripts/gen-runtime-surface.ts` and
`apps/sail-site/scripts/gen-tutorial.ts` are the two remaining
load-bearing site scripts. They have `--check`. Fold them into
`packages/sail/runtime` and the compiler/docs pipeline. Do not add a third.

## Tests — where and how

| Layer | Home | Query |
|---|---|---|
| Author / app | `@ship/sail-test` | role / name / state only |
| PW locator | `@ship/sail-test/contract` (`getByContract`) | not package root (mocks) |
| Vitest adapter | `@ship/test-setup` | not the public identity |
| Compiler / crate | `crates/*/tests` + `tests/fixtures/**` | structure OK |
| Hygiene | `tests/hygiene` | bun |
| Owned mini-app | `crates/sail/compiler/tests/fixtures/lab` | `sail test` |

One-way law: crate tests never path-join `apps/sail-site` or `apps/studio`
as fixtures. `type_gate_tsc_parity.rs` is the only named product gate.

App suites: no `data-sail-host`, `data-*`, class, `s-*`, testid as
primary locator. Framework suites (hydrate, morph, canary) may use structure.

Do not treat the live site or studio as a crate fixture. Do not copy
the site tree "to own it."

Site/studio Playwright rewrite is operator-coordinated. Pattern lives
in `packages/sail/sail-test/test/app-surface.test.ts`.

## Collision / isolation

- `job/release-integrate` owns stdlib, L1 skins, Spinnaker language,
  studio chrome. Stay off unless explicit DEFECT.
- Do not edit `preprocess.rs` `lift_tag_style` (void-slash / `keep_sep`).
- Do not invent fixtures under `tests/fixtures/l1-matrix/`.
- Local commits only on `job/*` unless the operator says push.

## Banned agent moves

- Skip-because-red. Tombstone tests without TTL.
- Denylist-string theater (`json.contains("s-")`) when an allowlist exists.
- `bun run gen` / one-off `.mjs` to paper over a missing verb.
- Regenerating stdlib-coupled dumps to "make CI green" on this lane.
- Using `apps/sail-site` as a test corpus.
- Writing exploits, or attacking systems. (org default)

## Quality

Verify the command that names the law (`bun test tests/hygiene`,
`cargo test -p … --test fixture_law`). Do not claim a gate you did not run.
If Sail typecheck is red on files you touched, fix it.
