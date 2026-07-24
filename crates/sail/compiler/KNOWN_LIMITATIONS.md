# Known limitations — `sail-compiler`

Honest inventory of what the compiler and the L1 language elements do **not**
do yet. Every entry cites the source that enforces the limit, the smallest
repro, and a status. If you hit something here it is a known gap, not a bug —
file against the `sail-language` subsystem.

Verified against `sail-compiler` at the `spike/sail-sx` line. Compiler test
suite is green and the build is warning-free.

## Language / compiler

### `motion:` accepts only a static object literal
- **What:** `motion:<trigger>={{ ... }}` must be a static object literal.
  A dynamic/computed config is rejected at parse time.
- **Repro:** `<button motion:press={cfg()}>` → compile error
  "dynamic configs are not yet supported".
- **Where:** `src/parse/template/attrs.rs:583` (`parse_motion_static_config`).
- **Status:** planned — a dynamic form needs a runtime bridge; static baking
  is milestone 1.

### `motion:` properties are an allow-list
- **What:** Only the supported motion properties bake; anything else errors.
- **Repro:** `<div motion:press={{ blur: 'sm' }}>` → "motion property `blur`
  is not supported".
- **Where:** `src/parse/template/attrs.rs:676`.
- **Status:** by-design today; new properties are added deliberately as presets.

### `motion:` size aliases (`sm`/`md`/`lg`) are not yet distinct
- **What:** `motion:scale:sm` and `motion:scale:md` currently animate to the
  same identity end-state; the size alias does not change the curve/magnitude.
- **Repro:** `motion:scale:sm` vs `motion:scale:md` produce identical output.
- **Where:** `src/emit/template_emit/dom_emit.rs:416`.
- **Status:** planned (future milestone) — aliases parse but are equivalent.

### Synthetic render (docs playground preview) can't model propful components
- **What:** The in-browser "synthesize a preview" path can't render a component
  that declares props in `<scope>` — it has no controller scope to source prop
  values. The real compile path handles `<scope>` props fine; this is preview-only.
- **Repro:** a lesson/component with a `<scope>` props block shown in the playground
  preview → "component `<scope>` prop declarations are not supported yet".
- **Where:** `src/emit/synthetic/mod.rs:191`.
- **Status:** planned (Phase 2: a props-parameterized scope factory).

## L1 language elements

### `<card>` header/footer bleed chrome is PE-md-sized
- **What:** Named header/footer slots carry fixed space-token bleed (`-mx-5`
  / `-mt-4` / … matching padding=md). Combining `padding="sm|lg|none"` with
  filled header/footer slots does not re-bleed to the new pad; body-only
  cards (no header/footer) track padding CVA correctly.
- **Repro:** `<card padding="sm"><h3 slot="header">…</h3>…</card>` header
  bleed is still md-width.
- **Where:** `stdlib/card.sail` structure sx on `.header` / `.footer`.
- **Status:** accepted trade for zero `--card-*` channel PE; revisit if
  padding×slot combinations become common.

### `<card interactive>` is visual only — no keyboard/role semantics
- **What:** `interactive` applies the full hover/active/focus-visible skin but
  ships no `role="button"` / `tabindex` / Enter-Space activation (that was
  runtime JS in the L2 baseline; element mode is static structure).
- **Repro:** `<card interactive>` is not keyboard-activatable on its own.
- **Where:** `stdlib/card.sail` (LANG-GAP note).
- **Status:** by-design — add `role`/`tabindex`/keydown at the use site, or
  slot a real `<button>`/`<a>`.

### `<icon>` has no tone prop paint
- **What:** L1 `<icon>` inks via `currentColor` only. Per-tone icon color
  needs a tones sheet shared across stdlib elements, which the stdlib compiler
  can't emit yet.
- **Repro:** `<icon tone="destructive">` does not tint; wrap in a
  tone-colored context or set `color` instead.
- **Where:** `stdlib/icon.sail` (LANG-GAP note).
- **Status:** planned — shared cross-element tones sheet.

### `<icon>` has no named-glyph lookup
- **What:** L1 `<icon>` only sizes/inks a slotted `<svg>`. `name` / `src` /
  `set` / `variant` (e.g. lucide fetch, tile frames) are L2 controller behavior
  and intentionally live in an L2 component built on top of `<icon>`.
- **Repro:** `<icon name="check" />` renders nothing; slot the SVG:
  `<icon><svg>…</svg></icon>`.
- **Where:** `stdlib/icon.sail` (header note).
- **Status:** by-design — named-glyph lookup is an L2 concern.
