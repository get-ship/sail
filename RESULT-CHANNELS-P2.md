# RESULT — CHANNELS Phase 2a

## Status

**FIX applied** (hard-review must-fix). Ready for quarantine re-review.

Base tip (pre-FIX): `b81e7aed` — `feat(sail): CHANNELS-1 Phase 2a — dual-accept style({ box, paint, text })`

## Blocking must-fix (from REVIEW)

### 1. Regenerate framework CSS — `element-skins.css` matches `boxSizing` class mint

**Done.**

```bash
cargo run -p sail-compiler -- framework-css
# wrote packages/sail/tokens/src/element-skins.css
```

Diff (material):

| Before | After |
|--------|-------|
| `.sx-boxborder{box-sizing:border-box}` (switch skin) | `.sx-boxSizingborder{box-sizing:border-box}` |

Root cause: Phase 2a renamed setter `box` → `boxSizing` in `switch.sail` + setters registry, which changes the class atom mint (`sx-box`+`border` → `sx-boxSizing`+`border`), but the committed framework half was not regenerated. Live `skins_css()` already emitted the new atom; the static file lagged.

No sibling gen artifacts rewritten — `framework-css` only materializes `packages/sail/tokens/src/element-skins.css`.

### 2. Prove `element_skins_css_matches_registry` green

**Done.**

```text
cargo test -p sail-compiler --test element_mode
test result: ok. 13 passed; 0 failed
  including: element_skins_css_matches_registry ... ok
```

### 3. Harden probe so switch box-sizing class/CSS cannot drift silent again

**Done** — minimal asserts on `switch_host_track_paint_on_host_sx` in
`crates/sail/compiler/tests/sx_host_paint_probe.rs`:

- source uses `boxSizing(border)` (not retired `style.box`)
- live compile stamps `sx-boxSizingborder` / emits `box-sizing:border-box`
- retired `sx-boxborder` absent from structure + skin
- **committed** `element-skins.css` contains `.sx-boxSizingborder{…}` and not `.sx-boxborder{`

Message points at regen command when the file half drifts.

```text
cargo test -p sail-compiler --test sx_host_paint_probe
test result: ok. 27 passed; 0 failed
  including: switch_host_track_paint_on_host_sx ... ok
```

## Leftover grep

| Pattern | Result |
|---------|--------|
| `sx-boxborder` (call sites / CSS) | **gone** from shipped CSS; only probe anti-asserts + comments |
| `style.box(` active call sites | **none** — only rename pointer diagnostics in `sail-style` parse + unit test exercising the pointer |
| Call sites using `boxSizing` | `switch.sail`, `sail-board.sail`, style unit tests |

Pointer diagnostics for `style.box(…)` → `style.boxSizing(…)` intentionally retained (temporary own-API rename pointer while corpus transitions).

## Out of scope (not touched)

- No tip land / no push
- No Phase 2b (animate channel)
- No mass codemod beyond the prior Phase 2a boxSizing renames

## Suites run (FIX)

| Suite | Result |
|-------|--------|
| `cargo test -p sail-compiler --test element_mode` | 13/13 ok |
| `cargo test -p sail-compiler --test sx_host_paint_probe` | 27/27 ok |
| `cargo test -p sail-style box_sizing` | 1/1 ok (`box_sizing_rename_and_alias_error`) |

## Files changed (FIX)

1. `packages/sail/tokens/src/element-skins.css` — regen (switch atom only material delta)
2. `crates/sail/compiler/tests/sx_host_paint_probe.rs` — boxSizing drift probe
3. `RESULT-CHANNELS-P2.md` — this evidence

## Self-score (quarantine re-review)

| Criterion | Score | Notes |
|-----------|-------|-------|
| Blocking fix complete | 10/10 | CSS regen + equality green |
| Drift cannot recur silent | 9/10 | probe + full equality dual-guard; still relies on suite being run |
| Scope discipline | 10/10 | no 2b, no land, no push |
| Evidence quality | 9/10 | suites listed; leftover grep clean |

**Overall: 9.5 / 10** — ready for re-review; only residual risk is process (not re-running element_mode after future setter renames), which the probe now fails loudly on.
