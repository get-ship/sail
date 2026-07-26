# Action / DOM port inventory

**Branch:** `job/style-synthesis`  
**Scope:** `packages/sail/runtime/src/actions/**`, related runtime notes, residual `class=` in `crates/sail/compiler/stdlib/**/*.sail`  
**Date:** 2026-07-26  
**Question:** Does the **action port** contain DOM coupling that leaks paint, or only the intended CP / behavior seams?

## Executive answer

| Concern | Result |
| --- | --- |
| **Paint leaks** (imperative color / background / font / layout props via `.style`) | **None** in actions |
| **CP write port** (`setProperty('--*', …)` / `removeProperty('--*')`) | **Healthy** — 20 exported actions, all `--*` names |
| **Geometry / behavior DOM** (focus, popover, measure, listeners, createElement) | **Present and acceptable** under `sail-action-no-style` |
| **Anatomy class coupling** (`.sail-*` / `.hint` / `.code` / …) | **Yes — the real residual debt** |
| App-author `class=` | **Floor-banned** (`sail-no-class`) |
| Stdlib `class=` | **Allowed** — L1 `compile_element` **bypasses** the lint floor |

**Bottom line:** The action port is **not** a paint channel. It is a **CP bridge + behavior mix-in** layer that still **depends on class anatomy** for several L1 elements. That class dependence is the migration target, not a wave of style rewrites.

---

## Method

Grep targets under `packages/sail/runtime/src/actions/**/*.ts`:

- `el.style` / `.style.setProperty` / `.style.removeProperty` / `.style.* =`
- `querySelector` / `querySelectorAll` / `getElementById` / `closest` / `classList`
- free `document.` / `window.`

Stdlib: `class=` in `crates/sail/compiler/stdlib/**/*.sail`.

Classification per hit:

| Tag | Meaning |
| --- | --- |
| **CP write** | Named custom property; Style/sx/`(--name)` reads — good port |
| **geometry/behavior DOM** | focus, popover, measure, listeners, DOM create/query for structure — acceptable action |
| **paint leak** | Sets color/background/font/layout via plain style — **BAD** |
| **anatomy class coupling** | Depends on `.sail-*` / bare part classes (`.hint`, `.code`, …) |

---

## Counts by classification

### Style surface in actions

| Classification | Action symbols (exported) | Notes |
| --- | --- | --- |
| **CP write** | 20 | All writes are `--*`; lint-enforced by `sail-action-no-style` |
| **paint leak** | **0** | No `el.style.color` / `background` / `font*` / plain layout prop assigns |
| **geometry/behavior only** (no style) | ~25 files | Focus, ARIA, popover, RO/IO, listeners |
| **anatomy class coupling** | **13 actions** | Class selectors or `classList.contains` |

Exported symbols with CP writes (from `index.ts` + file exports):

`autosize`, `chromeOffset`, `spotlight`, `zoomPan`, `maxWidth`, `swatch`, `contextMenu`, `worldTransform`, `gridTracks`, `anchor`, `targetCursor`, `virtualRow`, `virtualSpacer`, `cellPad`, `tipDescribe`, `eventSpan`, `rect`, `pin`, `portEdge`, `rangeFill`.

### querySelector / document / window in actions

| Kind | Approx. hit files | Role |
| --- | --- | --- |
| `querySelector(All)` / `closest` / `getElementById` | 16 files | Child anatomy, option lists, overlay style inject |
| `document.` | 10 files | createElement overlays, activeElement, listeners, theme attr, getElementById |
| `window.` | 3 files | resize/scroll measure (`anchor`, `chromeOffset`, `contextMenu`) |

None of these are paint channels; they are structure/behavior.

### Residual `class=` in stdlib

| Metric | Count |
| --- | --- |
| Files with `class=` | **16** |
| `class=` attribute lines | **~208** |
| of which `theme-picker.sail` | **158** (swatch chrome: `tc-*` + `sail-theme-*`) |
| Non–theme-picker lines | **~50** |
| Unique class tokens | **~55** |

---

## CP write inventory (good port)

Every action style write uses `style.setProperty` / `removeProperty` on **`--*`** names. Companion sx/skin reads via `(--name)` / `var(--name)`.

| File | Symbol(s) | Custom properties | Consumer role |
| --- | --- | --- | --- |
| `autosize.ts` | `autosize` | `--autosize-h` | Measured textarea height |
| `chrome-offset.ts` | `chromeOffset` | `--chrome-offset` | Sticky chrome height → root |
| `spotlight.ts` | `spotlight` | `--spot-x`, `--spot-y`, `--spot-strength` | Pointer-lit radial glow (overlay node) |
| `zoom-pan.ts` | `zoomPan` | `--zoom-tx` | Composed translate+scale |
| `zoom-pan.ts` | `maxWidth` | `--max-w` | Runtime stage max-width |
| `zoom-pan.ts` | `swatch` | `--swatch-color` | User-picked color (not a design token) |
| `context-menu.ts` | `contextMenu` | `--anchor-left`, `--anchor-top` | Cursor-anchored popover |
| `world-transform.ts` | `worldTransform` | `--world-tx` | Canvas world matrix string |
| `grid-tracks.ts` | `gridTracks` | `--sail-grid-template` | Dynamic track list |
| `anchor.ts` | `anchor` | `--anchor-left`, `--anchor-top`, `--anchor-width` | Popover placement vs invoker |
| `target-cursor.ts` | `targetCursor` | `--tc-x`, `--tc-y` | Bracket/dot overlay position |
| `virtual-row.ts` | `virtualRow` | `--row-y` | Virtualizer row offset |
| `virtual-row.ts` | `virtualSpacer` | `--rows-h` | Virtual spacer height |
| `virtual-row.ts` | `cellPad` | `--cell-ps` | Tree indent padding-start |
| `tip-describe.ts` | `tipDescribe` | `--tt-*`, `--tt-arrow-*`, `--sx-translate-y` | Tooltip bubble + arrow geometry |
| `event-span.ts` | `eventSpan` | `--evt-top`, `--evt-h`, `--evt-left`, `--evt-w` | Calendar/canvas rect |
| `rect-box.ts` | `rect` | `--box-top`, `--box-left`, `--box-w`, `--box-h` | Absolute box |
| `rect-box.ts` | `pin` | `--pin-x`, `--pin-y` | Point pin |
| `port-edge.ts` | `portEdge` | `--port-top/left/right/bottom` | Edge-docked port |
| `range-fill.ts` | `rangeFill` | `--rng-pct` | Slider fill percent |

**Note on `swatch` / `--swatch-color`:** This *is* a paint value, but it crosses as a **named CP** that sx consumes (`bg-(--swatch-color)`). That is the documented runtime-geometry/user-input seam — **not** a paint leak (`el.style.background = …` is absent).

**Note on overlays (`spotlight`, `targetCursor`):** Companion `<style>` blocks key on `data-sail=…` (not classes). Per-frame motion is still CP-only. Overlay paint lives in injected CSS tokens (`var(--primary)`), not JS style assigns.

### Related non-action runtime (out of action port, for context)

| File | Style use | Classification |
| --- | --- | --- |
| `theme.ts` | `root().style.setProperty` / `removeProperty` on token names | Privileged theme seam (documented) |
| `bind.ts` | `el.style.display` show/hide | Framework bind (not action port) |
| `motion.ts` | `el.style.opacity` / `scale` / `translate` / `rotate` | Motion engine WAAPI-adjacent |
| `scroll-lock.ts` | `body.style.overflow` / `paddingRight` | Explicit body lock seam |
| Controllers | — | Stricter `sail-no-imperative-style` (no `.style` at all) |

These are **not** the action port; they do not change the paint-leak verdict for actions.

---

## Geometry / behavior DOM (acceptable)

Actions with **no** style writes that still touch the platform (allowed by ARCHITECTURE / `sail-action-no-style`):

| File | Symbol | DOM role |
| --- | --- | --- |
| `click-outside.ts` | `clickOutside` | Capture-phase `document` pointerdown |
| `auto-dismiss.ts` | `autoDismiss` | Timer + hover/focus pause |
| `details-toggle.ts` | `detailsToggle` | `<details>` open sync |
| `details-disabled.ts` | `detailsDisabled` | Block open when disabled |
| `pressable.ts` / `pressed.ts` | press/ARIA | Pointer + `aria-pressed` |
| `dismissible.ts` | `dismissible` | Escape / `.remove` click → dismiss |
| `dismiss-control.ts` | `dismissControl` | Close control selectors |
| `dismiss-self.ts` | `dismissSelf` | Event + `el.remove()` |
| `live-region.ts` | `liveRegion` | Live region attrs |
| `media-fallback.ts` | `mediaFallback` | Error → fallback attribute |
| `icon.ts` | `icon` | Fetch/inject SVG (no class stamp) |
| `variant-live.ts` | `variantLive` | Attr sync for live variants |
| `highlight.ts` | (highlight) | Text/range highlight behavior |
| `hover-surface.ts` | `hoverSurface` | Popover open on invoker hover |
| `combobox.ts` | `combobox` | input + listbox/option + popover |
| `menu.ts` | `menu` | Roving focus on items |
| `tabs.ts` | `tabs` | Tab selection + ARIA |
| `radio-group.ts` | `radioGroup` | Group + hidden input + ARIA |
| `pagination.ts` | `pagination` | Page current + click |
| `toggle-group.ts` | `toggleGroup` | Roving tabindex + pressed |
| `dropzone.ts` | `dropzone` | file input + `[data-dz-files]` |
| `syntax-highlight.ts` | `syntaxHighlight` | `pre` text paint via highlighter API |
| `catch-render.ts` | `catchRender` | Error boundary handle + retry |
| `recovery-surface.ts` | `recoverySurface` | Code chip + details toggle + focus |
| `described-control.ts` | `describedControl` | aria-describedby wiring |
| `marquee.ts` | `marquee` | Track animation duration from host |
| `theme-select.ts` | `themeSelect` | Theme option list + document theme |

`document` / `window` uses are measure, global listeners, overlay mount, or theme read — not style mutation.

---

## Anatomy class coupling (the residual debt)

### Action → class selectors

| Action | Selectors / class reads | Stdlib stamp | Severity |
| --- | --- | --- | --- |
| `describedControl` | `.hint`, `.error`, `.control`, `.sail-switch` | `field.sail` | **High** — a11y wiring |
| `recoverySurface` | `.code`, `.details`, `.details-toggle`, `.actions` | `error-state.sail` | **High** |
| `catchRender` | `.message`, `.retry` | `error-boundary.sail` | **High** |
| `tipDescribe` | `.bubble` | `tooltip.sail` | **High** |
| `dismissible` | `.remove` | `chip.sail` | Medium |
| `dismissControl` | `button.close`, **`classList.contains('sail-toast')`** | `alert.sail` / toast host stamp | Medium |
| `menu` | `.sail-menu-item` | host stamp on `menu-item` | Medium (L1 family) |
| `tabs` | `.sail-tab`, `.sail-tab-panel`, `.sail-tab-list` | host stamps | Medium |
| `toggleGroup` | `.sail-toggle` | host stamp | Medium |
| `pagination` | `.sail-page` | host stamp | Medium |
| `radioGroup` | `.sail-radio` | host stamp | Medium |
| `themeSelect` | `.sail-theme-trigger`, `.sail-theme-panel`, `.sail-theme-option` | `theme-picker.sail` | Medium |
| `marquee` | `.sail-ticker` (closest) | host stamp on ticker | Low |

**Better patterns already present (no class):**

- `dropzone` → `input[type=file]`, `[data-dz-files]`
- `spotlight` / `targetCursor` → `data-sail=…`
- `combobox` → tag/`option` / popover (still DOM, not class anatomy)
- `hoverSurface` → `[popovertarget="…"]`

### Two shapes of class coupling

1. **Host identity classes** (compiler-stamped `.sail-<element>` on use site / structure root)  
   Used so a parent action can find children (`tabs` → `.sail-tab`). These are **framework-emitted** at stamp time for app authors, but **stdlib still authors** some (`sail-theme-*`, `sail-nav-*`).

2. **Internal part classes** (`.hint`, `.bubble`, `.code`, `.remove`, `.message`, …)  
   Authored only inside L1 element sources; actions hard-code the same strings. CEM even collects these as published anatomy (`emit/cem.rs`: “classes ARE the anatomy in the light-DOM”). That collides with the app-facing `sail-no-class` floor narrative (“anatomy = role / aria / tag”).

---

## Residual `class=` in stdlib (every file)

### Per file (line counts)

| File | `class=` lines | Action hooks? | Notes |
| --- | --- | --- | --- |
| `theme-picker.sail` | 158 | **Yes** — `sail-theme-{trigger,panel,option}` | Bulk `tc-*` is **swatch chrome only** (no action query) — avoidable / cosmetic |
| `error-state.sail` | 8 | **Yes** — code, details, details-toggle, actions | + icon/copy/title/desc layout |
| `error-boundary.sail` | 6 | **Yes** — message, retry | + content/fallback layout gates |
| `alert.sail` | 6 | Partial — `close` via `button.close` | ico/title/body/actions mostly layout/`empty:` |
| `field.sail` | 5 | **Yes** — hint, error, control | label/error-gate/msg also |
| `empty-state.sail` | 5 | No action query | icon/copy/title/desc/actions (skin/layout) |
| `navbar.sail` | 4 | No action query | `sail-nav-*` skin slots |
| `ticker.sail` | 3 | Indirect — marquee on track; host is `.sail-ticker` stamp | track/segment structure |
| `spinner.sail` | 2 | No | spin/ring skin |
| `combobox.sail` | 2 | Prefer tags | `input`/`listbox` redundant with querySelector tags |
| `collapsible.sail` | 2 | No action | trigger-wrap / content-wrap skin |
| `chip.sail` | 2 | **Yes** — `.remove` | leading slot |
| `card.sail` | 2 | No | header/footer empty-gate |
| `tooltip.sail` | 1 | **Yes** — `.bubble` | |
| `radio.sail` | 1 | No (group uses `.sail-radio` host stamp) | `rlabel` |
| `label.sail` | 1 | No | `req` required marker |

### Unique class tokens (grouped)

**Action anatomy hooks (must migrate carefully):**  
`hint`, `error`, `control`, `bubble`, `code`, `details`, `details-toggle`, `actions`, `message`, `retry`, `remove`, `close` (as `button.close`),  
`sail-theme-trigger`, `sail-theme-panel`, `sail-theme-option`,  
(+ framework host stamps queried from actions: `sail-tab`, `sail-tab-list`, `sail-tab-panel`, `sail-toggle`, `sail-page`, `sail-radio`, `sail-menu-item`, `sail-ticker`, `sail-switch`, `sail-toast`, `sail-icon` — many emitted by stamp, not always authored in the `.sail` body).

**Skin / layout only (no action query found):**  
`tc-overlay`, `tc-card`, `tc-eyebrow`, `tc-heading`, `tc-typo`, `tc-body`, `tc-mono`, `tc-controls`, `tc-cta`, `tc-ghost`, `tc-chip`, `sail-theme-swatch`,  
`sail-nav-brand`, `sail-nav-links`, `sail-nav-center`, `sail-nav-actions`,  
`icon`, `copy`, `title`, `desc`, `header`, `footer`, `spin`, `ring`, `track`, `segment`, `leading`, `req`, `rlabel`, `label`, `msg`, `error-gate`, `content`, `fallback`, `fallback-icon`, `fallback-body`, `body`, `ico`, `input`, `listbox`, `trigger-wrap`, `content-wrap`, `content-inner`.

Theme-picker alone accounts for **12 × 12** repeated `tc-*` / swatch tokens (~144 of 158 lines).

---

## Can app authors use `class=`?

**No — floor-banned.**

| Layer | Rule | Effect |
| --- | --- | --- |
| App / pack / route `.sail` | `sail-no-class` in `FLOOR_RULES` | Hard Error; overrides cannot soften |
| Lint copy | `crates/sail/lint/src/rules/no_class.rs` | class is a global CSS back door in light DOM |
| Tests | `floor_unconditional.rs`, `check_all.rs` | Compile refuses `class="x"` |
| **L1 stdlib** | `compile_element` **does not** run `enforce_floor` / lint gate | **Privileged exception** — “they ARE the styling layer” (`check/lint_overrides.rs`, `check/mod.rs`) |
| Perf harness | path-exempt | Interop may need Bootstrap `class=` |

Compiler still **emits** its own classes (`sail-root`, `sail-<element>`, `sx-*` atoms). Authors must not write `class=` / `class:`; anatomy for apps is tag / role / aria / `data-*` / `bind:this` / `use:`.

---

## Paint leaks

### Actions: **none**

No assignment to plain CSS properties via `.style` in `packages/sail/runtime/src/actions/**`.  
No `setProperty('color'|…)` / non-`--*` layout props detected.

**Worst paint leaks:** *n/a — inventory is clean on this axis.*

Closest “paint-adjacent” CP ports (still legal):

1. **`swatch` → `--swatch-color`** — user color into cascade via sx  
2. **`spotlight` / `targetCursor`** — inject companion CSS using design tokens; per-frame vars only  
3. **Non-action:** `motion.ts`, `bind.ts` display, `scroll-lock.ts` body — outside action floor by design

---

## Recommendation: kill residual `class=` (migration path)

Do **not** mass-rewrite stdlib skins first. Order by **action coupling**, then cosmetic.

### Preferred target: data parts + compiler stamps

1. **Part attributes instead of part classes**  
   Replace action hooks with stable `data-sail-part="hint|error|control|bubble|…"` (or a single `data-part` enum).  
   Actions query `[data-sail-part=hint]` — same DOM cost, **no CSS global class name**, no collision with `sail-no-class` doctrine.  
   Aligns with overlay precedent (`data-sail='spotlight-glow'`, `data-sail='tc-bracket'`).

2. **Host identity without author class**  
   Keep compiler-stamped `.sail-tab` etc. for skins if needed, **or** migrate actions to:
   - tag name (`tab`, `menu-item`) after light-DOM custom elements, or  
   - `data-sail-host="tab"` stamped only by the compiler.  
   Parent-group actions (`tabs`, `menu`, `toggleGroup`) should key off **stamped host identity**, not hand-authored classes in child templates.

3. **CEM anatomy**  
   Today CEM publishes **classes** as anatomy (`emit/cem.rs`). Publish **parts** (data attributes / future part names) instead so reskins and actions share one contract without reopening author `class=`.

4. **Avoid `::part` / shadow**  
   ARCHITECTURE already refuses author `:host` / `::part` as surface. Stay light-DOM: data attributes + stamped host selectors.

5. **Quick wins (optional, tiny)**  
   - `dismissControl`: drop `classList.contains('sail-toast')` in favor of tag/`data-sail-host` or existing `duration` attr (already partially there).  
   - `combobox.sail`: drop redundant `class="input"|"listbox"` if nothing in CSS needs them (action already uses tags).  
   - `theme-picker` `tc-*`: pure chrome; either leave under L1 privilege or convert to sx-only structure (no action dependency).  
   - `alert` close: already partially data-driven (`[data-dismiss]`); prefer that over `button.close`.

### Migration phases

| Phase | Work | Risk |
| --- | --- | --- |
| **A** | Document inventory (this file); keep CP port frozen | None |
| **B** | Dual-write: stamp `data-sail-part` **and** keep class; actions accept either | Low |
| **C** | Actions query data-only; delete class selectors | Medium (tests/CEM) |
| **D** | Strip non-hook classes from stdlib; shrink theme-picker `tc-*` if skin allows | Low–medium |
| **E** | CEM/parts manifest; ban new anatomy classes in element review | Governance |

**Do not:** reintroduce author `class=` for apps, or widen `compile_element` bypass to pack sources.

---

## Port health summary

```
Action port
├── CP write .............. CLEAN (good)
├── paint leak ............ NONE
├── geometry/behavior DOM . EXPECTED (focus, measure, popover, overlays)
└── anatomy class ......... COUPLED (13 actions; migrate to data-part / stamps)
```

**Verdict:** The action port **contains DOM coupling**, but it is **behavior + anatomy**, not paint. Residual `class=` is **stdlib/L1 privilege** powering those anatomy queries; app authors remain floor-banned. Next work is **part identity migration**, not style rewrite.

---

## Source anchors

- Action floor: `docs/ARCHITECTURE.md` (Action body floor table), `crates/sail/lint/src/rules/action_no_style.rs`
- Class floor: `crates/sail/lint/src/rules/no_class.rs`, `crates/sail/compiler/src/check/policy.rs` `FLOOR_RULES`
- L1 floor bypass: `crates/sail/compiler/src/check/mod.rs` (“L1 element sources bypass…”), `compile_element` in `crates/sail/compiler/src/lib.rs`
- Registry: `packages/sail/runtime/src/actions/index.ts`
)
