# Agent / contributor rules

## Retirement is a scrub (pre-launch)

**Scope:** pre-launch practice. No public/versioned sail consumers exist off
this machine. Once sail is a public versioned surface, retirement follows
deprecation windows instead of instant scrub.

1. **Retirement = scrub, then delete.** A retired syntax gets: (a) the one-time
   migration (codemod/pointer error) while the corpus transitions, (b) a corpus
   scrub proving zero occurrences, (c) **deletion** of the tombstone — the
   manifest row, the pointer error, the migration test. After that the syntax
   simply does not exist; the generic unknown-vocabulary error handles it.

2. **Tombstones for our own retired APIs are temporary.** Embalming retired
   APIs forever with pointer errors and migration tests is not acceptable.
   After scrub, own-API ghosts fail as generic unknown vocabulary (from the
   manifests), not "this was retired in version X."

3. **Foreign-idiom guidance is permanent and different.** An author reaching
   for an HTML/CSS/JS habit (`class=`, `style=`, `document.querySelector` in a
   controller, layout-by-`<div>`) gets a smart pointer to the sail way. That is
   teaching a foreign language's users, not embalming our history. One
   data-driven table (`FOREIGN_IDIOMS` in the compiler directive manifest).

4. **Scrub tests are a temporary migration harness, segregated from real
   suites.** They live under `tests/scrub/` with their own runner, are wired
   only for the migration window, and are deleted with the tombstone. They
   never intermix with legitimate behavioral suites.

Do not revive `ARCHITECTURE.md`. Doctrine for agents lives here until internal
tools land.
