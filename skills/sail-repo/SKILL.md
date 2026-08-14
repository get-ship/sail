---
name: sail-repo
description: >
  Sail monorepo conventions. Rules only. Use when placing output, tests,
  scripts, generated files, or fixtures. Triggers: /sail-repo, "where does
  this belong", scripts/, .sail, .deck, fixtures, test conventions.
---

# Sail repo conventions

If a rule can be a test, it is a test. Do not restate hygiene here.

## Homes

- **Build out:** `<project>/.deck/`. One name. No `.sail-out`. No second default.
- **Project store:** `<project>/.sail/`. One name. Tasks, types, and test
  artifacts do not get a second directory also called sail.
- **Source companions:** `*.sail.d.ts` sit next to the `.sail`.
- Do not invent a new cache/out/store name.
- Do not treat a hygiene ban on a leftover path as the design.

## Generated

- Untrack or gate. No third state (tracked and stale).
- Registry: `tests/hygiene/emitters.json`. A new tracked gen file without a
  row fails.
- Do not add an app script to emit product.

## Scripts

- Product emit is a `sail` / `sail-compiler` verb.
- `scripts/` is a wrapper around an existing verb, or it does not emit.
- An app that cannot be produced by `sail build` is a defect, not a
  `scripts/` folder.

## Tests

- Crate tests do not open `apps/sail-site` or `apps/studio` as fixtures.
- App tests locate UI by role / name / state. Not `data-*`, class, or testid.
- Framework tests of the compiler/runtime may use structure.
- New fixtures are owned trees under `crates/*/tests/fixtures` or
  `packages/*/test`. Not a copy of a live app.

## Agents

- Do not skip a red gate.
- Do not add a denylist-string test when the law is a shape or an allowlist.
- Do not explain a smell as two valid homes.
- Stay out of stdlib / L1 / Spinnaker language unless the operator names a
  DEFECT.
- Local `job/*` commits only, unless told to push.
