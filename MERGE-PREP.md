# MERGE PREP — framework-doover ← job/release-integrate

State at writing: ours = 57 ahead / 39 behind off base 93b053ce7. Tree clean @ e437bfe59.
Dry-run: `git merge-tree --write-tree HEAD job/release-integrate` → **15 conflicted files**.

## Conflict list + resolution strategy

| File | Strategy |
|---|---|
| `crates/sail/compiler/src/action_manifest.rs` | **OURS wholesale.** RI still carries the old 36-action table (menu, liveRegion, passwordReveal, tabs, icon, radioGroup, …). Ours is the culled 19. RI's only manifest change worth keeping: none found — verify `danger`-related summaries if any. |
| `crates/sail/compiler/stdlib/combobox.sail` | **OURS** (controller cutover) + scan RI diff for token renames to re-apply. |
| `crates/sail/compiler/stdlib/dialog.sail` | **Union**: our structure + RI's `panel-*`→`overlay-*` rename (max-w/vh/min-w). |
| `crates/sail/compiler/stdlib/hover-card.sail` | **OURS** (controller + role=dialog) + overlay rename. |
| `crates/sail/compiler/stdlib/switch.sail` | **Union**: keep our type-union fix (`checkbox|radio|file|color|date|button`), take RI's `bg(knob)`→`bg(inverse(surface))`. |
| `crates/sail/compiler/stdlib/theme-picker.sail` | **OURS ENTIRELY** — controller cutover supersedes RI's 2-line anchorLeft/overlay tweak. Re-apply overlay-max-w rename to the new panel style. |
| `crates/sail/compiler/tests/sx_host_paint_probe.rs` | **THEIRS** then re-add any of our asserts (sx_*→style_* rename was RI-side; file deleted in dry-run means check both). |
| `apps/sail-site/src/component-docs/alert/meta.ts` | Inspect manually; likely union of copy edits. |
| `packages/sail/l1/src/copy.ts` | Union — prefer controller-era descriptions (ours), accept RI copy where no conflict. |
| `packages/sail/l1/src/generated/catalog.ts` | **REGENERATE** post-merge (`bun run generate`). Never hand-merge. |
| `packages/sail/runtime/src/actions/live-region.ts` | RI modified; we deleted it earlier. **Keep deleted**, delete again with `git rm`. |
| `packages/sail/runtime/test/alert-behaviors.test.ts` | RI updated for their alert shape; reconcile with our controller-era alert (dismiss fn, no liveRegion action). |
| `packages/sail/tokens/model/base.json` | **THEIRS** — token model SSOT is theirs by design. Formatting changed (flat→nested); take whole file. |

## Token vocabulary map (RI renames — apply to OUR stdlib files during resolution)

| Old (ours) | New (RI) | Consumers to fix |
|---|---|---|
| `--backdrop` | `--scrim` | element-skins.css dialog ::backdrop ×2 |
| `panel-max-w-*` / `panel-vh-*` / `panel-min-w-*` | `overlay-max-w-*` / `overlay-vh-*` | drawer, dialog, theme-picker, hover-card (+ element-skins) |
| `--divider` | dropped (alias of border-subtle) | zero template consumers — safe |
| `--focus-ring` | dropped | zero consumers — safe |
| `--hue-*` | inlined into `--syntax-*` values | zero template consumers (`--syntax-number` kept) |

Plus ~99 new `--inverse-*` mirror vars — additive, nothing migrates onto them yet.

## Known pre-existing lint failure after merge
`sail-site-header.sail` productLink/githubLink `fg(canvas)` — RI's own header
(e6a872afb) fixed these to `fg(foreground)` on their side; the merge brings that
fix in automatically since the file auto-merges.

## Post-merge gate (same as every wave)
1. `cargo build -p sail-compiler-napi` + `git checkout -- Cargo.lock`
2. `cd packages/sail/l1 && bun vitest run` — expect 365+ (RI adds tests)
3. `cargo test -p sail-compiler --test element_controller --test chip_dissolve_probe --test abi_boundary`
4. Regenerate catalog if stdlib drifted
5. Expect the 14 pre-existing lib reds to persist or shift — triage against
   `/tmp/before.txt` baseline (preprocess/splice cluster).

## Merge order recommendation
1. Resolve conflicts per matrix.
2. Regenerate catalog + skins from merged token model.
3. Full gate.
4. Do NOT push; local job/* only.

## Deferred smells (documented, not scheduled)
- `swatch`/`liveBox`: migrate to `bind(signal)` geometry writers +
  `theme.preview/commit`; needs compiler confirmation that geometry binds
  accept number signals end-to-end. 22 swatch + 11 liveBox callers.
- resizable stays internal (verdict c).
