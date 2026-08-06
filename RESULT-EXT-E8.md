# RESULT — EXT-1 E8 (`sail create` + `.deck` hygiene)

**Branch:** `job/ext-e8-create`  
**Worktree:** `/home/madvib/dev/Ship/worktrees/ext-e8-create`  
**Date:** 2026-08-06  
**Authority:** EXT-1-external-app-story.md · VERIFICATION-BOARD E8  

## Tip SHA

`f5c5e9a82d5db1568d1dd575b47ce3d616ebeb73`

## Goal

Land `sail create my-app` for the external-app 4-entry tree, ensure scaffold
`.gitignore` includes `.deck/`, and make `sail check` warn when `.deck/` is
git-tracked. **No registry publish** (operator gate).

## What landed

| Surface | Change |
|---------|--------|
| `sail create <name>` | New top-level CLI command; writes package.json, sail.config.jsonc, routes/page.sail, themes/ship.json (baked catalog), .gitignore (via `ensure_project_store`), tsconfig.json + `.sail/types.d.ts` |
| package.json deps | Registry-style `@ship/*` at `^0.3.0` only — no `file:` / `workspace:*` |
| `.gitignore` | Canonical store includes `.deck/` (already in `project_store::CANONICAL_GITIGNORE`; create path calls `ensure_project_store`) |
| `sail check` hygiene | Warns (does not fail) when any path under `<project>/.deck/` is git-tracked |

## Behavioral tests (fail when wire broken)

| Test | Pin |
|------|-----|
| `create_writes_four_entry_tree_and_check_is_green` | tmpdir `sail create` → 4-entry tree + `.deck/` in gitignore → `sail check` green |
| `check_warns_when_deck_is_tracked` | force-add `.deck/index.html` in git → check still exits 0, stderr warns |
| `sail_help_lists_mvp_subcommands` | help lists `create` |

## Tests run

```text
cargo test -p deck-cli --test cli_smoke -- \
  create_writes_four_entry_tree_and_check_is_green \
  check_warns_when_deck_is_tracked \
  sail_help_lists_mvp_subcommands
# → 3 passed, 0 failed

# Manual acceptance (tmpdir):
sail create my-app && sail check my-app
# → ACCEPT_OK (1 file ok)
```

## Acceptance map

| Criterion | Status |
|-----------|--------|
| `sail create my-app` writes 4-entry tree | **done** (package.json, sail.config.jsonc, routes/, themes/) |
| gitignore includes `.deck/` | **done** (`ensure_project_store` / CANONICAL_GITIGNORE) |
| `sail check` warns if `.deck` tracked | **done** (warning only; exit 0) |
| No registry publish | **done** (operator; scaffold uses registry-style dep specs only) |
| tmpdir create→check green | **done** |
| RESULT-EXT-E8.md | **this file** |
| Commit; never push | **done** (no push) |

## Out of scope (not this job)

- npm / registry publish of `@ship/*` packages (operator)
- Full external hello-world CI lane create→dev→build outside monorepo (needs publish)
- Migrating monorepo apps off `tokens/themes` (E4 residual)

## Self-score

**READY** — `sail create` product path + `.deck` hygiene warning landed; publish remains operator.
