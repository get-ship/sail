# Style setter inventory (call counts + demotion map)

**Date:** 2026-07-26  
**Branch:** `job/style-synthesis`  
**SSOT:** [`crates/sail/lint/src/style/setters.rs`](../../../crates/sail/lint/src/style/setters.rs) — `SetterSpec` table via `setters()`  
**Corpus:** 273 `*.sail` files (excluding `node_modules` / `target` / build artifacts)  
**Method:** walk every `style.…` chain in `.sail` sources; count each known setter method on the chain; nested `style.…` inside `when` / `match` / `compound` args counted separately (so `style.colorMix(…).when({ hover: style.colorMix(…) })` = 2).

---

## Executive summary

| Bucket | Setters | Used (≥1) | Call sites |
|--------|--------:|----------:|-----------:|
| **pure-paint** | 98 | 79 | **8,111** |
| **layout-ish** | 24 | 19 | **1,247** |
| **motion-adjacent** | 11 | 10 | **62** |
| **composite-recipe** | 14 | 11 | **61** |
| **Total** | **147** | **119** | **9,481** |

- **28 setters have zero product call sites** (including brand-new Tier-2 chrome: `controlFrame` / `controlLift` / `press`, and layout host-paint `gap` / `gapX` / `gapY`).
- Style is overwhelmingly paint: ~86% of calls are pure-paint.
- Composites are rare in absolute counts but high-leverage in stdlib (soft/solid tone recipes).
- Generated skill **SKILL.md is stale by one setter**: `colorMix` is missing (see § Documentation coverage).

---

## Classification rules

Each setter is in **exactly one** bucket:

| Bucket | Criteria |
|--------|----------|
| **pure-paint** | Color, type, border, radius, shadow, opacity, padding/margin, overflow, cursor, a11y chrome, single-concern CSS longhands, host markers (`group`) |
| **layout-ish** | Size (`w`/`h`/`min*`/`max*`/`size`), flex item (`grow`/`shrink`/`basis`/`self`/`order`/`justifySelf`), container flex/gap (`flexDir`/`gap*`), `display`, `position`, inset axes |
| **motion-adjacent** | `transition` / `duration` / `ease` / `translate*` / `rotate` / `scale` / `animate` / `transform` / `willChange` / `origin` |
| **composite-recipe** | Multi-slot packs, two-arg color recipes, or component-specific chrome (`colorMix`, `colorDeepen`, control chrome, `textGradient`, `bgGlow`, media packs, `truncate`, `srOnly`, card `backdrop`/`borderImage`) |

Padding/margin stay **pure-paint** (box chrome), not layout-ish — layout policy already steers axis/gap to layout elements; pad is the dominant Style spacing channel.

---

## Top call-site offenders (files)

| Calls | File |
|------:|------|
| 736 | `crates/deck/deck-conformance/app/routes/gallery-sail-primitives/page.sail` |
| 394 | `crates/deck/deck-conformance/app/routes/gallery-ship-code/page.sail` |
| 345 | `crates/deck/deck-conformance/app/routes/gallery-ship-arcade/page.sail` |
| 343 | `crates/deck/deck-conformance/app/routes/gallery-sail-chat-elements/page.sail` |
| 290 | `crates/deck/deck-conformance/app/routes/gallery-sail-charts/page.sail` |
| 240 | `crates/sail/compiler/stdlib/theme-picker.sail` |
| 237 | `crates/deck/deck-conformance/app/routes/gallery-sail-viewer/page.sail` |
| 224 | `crates/deck/deck-conformance/app/routes/gallery-ship-markdown/page.sail` |
| 206 | `apps/sail-site/routes/page.sail` |
| 199 | `crates/deck/deck-conformance/app/routes/gallery-sail-media/page.sail` |
| 196 | `crates/sail/compiler/stdlib/chip.sail` |
| 190 | `crates/deck/deck-conformance/app/routes/gallery-ship-authority/page.sail` |

Gallery/demo pages dominate volume; stdlib + sail-site marketing pages are the secondary cluster. Policy work should weight **stdlib recipes** higher than gallery noise.

### Top setters overall (all buckets)

| # | Setter | Calls | Bucket |
|--:|--------|------:|--------|
| 1 | `fg` | 1186 | pure-paint |
| 2 | `fontSize` | 915 | pure-paint |
| 3 | `bg` | 736 | pure-paint |
| 4 | `border` | 564 | pure-paint |
| 5 | `radius` | 509 | pure-paint |
| 6 | `pad` | 409 | pure-paint |
| 7 | `fontWeight` | 398 | pure-paint |
| 8 | `borderWidth` | 396 | pure-paint |
| 9 | `fontFamily` | 385 | pure-paint |
| 10 | `tracking` | 284 | pure-paint |
| 11 | `textCase` | 250 | pure-paint |
| 12 | `w` | 210 | layout-ish |
| 13 | `padX` | 203 | pure-paint |
| 14 | `maxW` | 164 | layout-ish |
| 15 | `mb` | 162 | pure-paint |

---

## Top 10 composites by usage

| Rank | Setter | Calls | Files | Primary homes |
|-----:|--------|------:|------:|---------------|
| 1 | **`colorMix`** | **26** | 4 | `stdlib/button.sail` (12), `stdlib/anchor.sail` (12), docs pages (2) |
| 2 | **`truncate`** | **16** | 12 | primitives (tree-grid, calendar, cmdk, …), chat-elements, demos |
| 3 | **`colorDeepen`** | **7** | 1 | `stdlib/anchor.sail` solid hover only |
| 4 | **`srOnly`** | **5** | 5 | chart components (line/area/bar/donut/heatmap) |
| 5 | `avatarMedia` | 1 | 1 | `stdlib/avatar.sail` |
| 6 | `backdrop` | 1 | 1 | `stdlib/card.sail` |
| 7 | `bgGlow` | 1 | 1 | `apps/sail-site/routes/page.sail` |
| 8 | `borderImage` | 1 | 1 | `stdlib/card.sail` |
| 9 | `glyphFill` | 1 | 1 | `stdlib/icon.sail` |
| 10 | `scrollbarThin` | 1 | 1 | `stdlib/scroll-area.sail` |

Honorable mentions (tied at 1 or zero): `textGradient` (1, hero page); **`controlFrame` / `controlLift` / `press` = 0** (implemented, unused in `.sail` — still owned by contracts for L1 buttons per button header comment).

---

## Deep dive: `colorMix`

### What it is

Two-arg paint composite (not a scale token):

```text
style.colorMix(token, pct)   // pct ∈ 5..95 step 5, or bind
```

SSOT comment in `setters.rs`:

> Soft wash primitive: color-mix hue into transparent by percent  
> (restores the deleted **tone-\*-bg** ramp without minting design tokens).

### Lowering

From `lower_color_mix_arg` in `setters.rs`:

```css
background-color: color-mix(in oklch, {token} {pct}%, transparent);
```

Where `{token}` is `var(--{bare})` (or keyword `transparent` / `current` / `currentColor`, or `var(--{bind})`).

Compare siblings:

| Setter | CSS shape | Intent |
|--------|-----------|--------|
| `colorMix(token, pct)` | `color-mix(in oklch, token pct%, transparent)` | Soft tinted fill |
| `colorDeepen(token, pct)` | `color-mix(in oklch, token, var(--foreground) pct%)` | Solid hover deepen toward foreground |
| legacy `tone-*-bg` | e.g. `color-mix(…, var(--primary) 14%, transparent)` in theme CSS | Fixed percent per tone channel (deleted as design tokens for Style) |

### Call sites (executable, not comments)

| Location | Pattern |
|----------|---------|
| `crates/sail/compiler/stdlib/button.sail` | `compound({ variant: soft, tone: T }, style.colorMix(T, 15)….when({ hover: style.colorMix(T, 25) }))` for primary/info/success/warning/destructive/accent |
| `crates/sail/compiler/stdlib/anchor.sail` | Same soft recipe as named consts (`softPrimary`, …) |
| `apps/sail-site/routes/sail/language/theming/page.sail` | Doc example in prose |
| `apps/sail-site/routes/sail/language/styling/page.sail` | Doc example in prose |

Note: button **default soft** (no tone) still uses `style.bg(hover-bg)`, not `colorMix`. Washes are the **toned soft** path only.

### Would a `soft-*` token replace it?

**Recommendation: no — keep parametric `colorMix`.**

| Option | Pros | Cons |
|--------|------|------|
| **`style.colorMix(hue, pct)` (current)** | One primitive covers rest(15) + hover(25) + any bind; no token explosion; matches “no tone-\* design tokens” direction | Two-arg API; agents must know the pct ramp |
| **`soft-primary` / `soft-primary-hover` tokens** | `style.bg(soft-primary)` ergonomics | Re-mints the deleted tone-\*-bg family; rest vs hover needs two tokens × N hues; binds lose flexibility |
| **Fixed `tone-primary-bg` only** | Matches existing theme CSS vars still present in site skins (`--tone-primary-bg` ≈ 14%) | Style product deliberately left these; percent locked; hover deepen still needs a second token or `colorDeepen`/`colorMix` |

Theme CSS still ships `--tone-*-bg` for non-Style chrome (prose, dropzone, mark). Style’s soft path intentionally **does not** spend those tokens — it recomputes via `color-mix` so soft fills track live hue tokens (`primary`, `info`, …).

**Demotion home for `colorMix`:** **stay Style** (composite paint recipe). Not layout, not motion, not contracts. Optional future: L1 soft-variant default could bake wash into platform CSS, but product authors still need the primitive for custom soft surfaces.

---

## Demotion map

Homes:

- **stay Style** — product paint surface
- **layout element attrs / `box=`** — row/column/grid axis & child placement
- **`fx:` / fx** — enter/exit/loop and intentional motion
- **token / L1 default** — design token or platform control default
- **contracts (platform only)** — `contracts.css` L1 chrome not authored per instance

### Composite-recipe

| Setter | Calls | Proposed home | Rationale |
|--------|------:|---------------|-----------|
| `colorMix` | 26 | **stay Style** | Parametric soft fill; replaces tone-\*-bg without tokens |
| `truncate` | 16 | **stay Style** (or text utility pack) | Multi-slot text pack; ubiquitous in data UIs |
| `colorDeepen` | 7 | **stay Style** | Solid hover deepen; stdlib-only today |
| `srOnly` | 5 | **stay Style** or a11y helper | Multi-slot clip pack; charts-only today |
| `avatarMedia` | 1 | **stay Style** *or* component-private | Avatar-only multi-slot; could freeze as avatar internal |
| `glyphFill` | 1 | **stay Style** *or* icon-private | Icon SVG child pack |
| `scrollbarThin` | 1 | **stay Style** *or* scroll-area default | Could become scroll-area L1 default + optional override |
| `textGradient` | 1 | **stay Style** | Marketing/hero gradient text |
| `bgGlow` | 1 | **stay Style** *or* token + layout | Emits position/inset/background — half layout; rare |
| `borderImage` | 1 | **token / contracts** for card; setter optional | Card corner-image channel |
| `backdrop` | 1 | **token / contracts** for card | `--card-backdrop-filter` |
| `controlFrame` | 0 | **contracts (platform)** primary; Style bridge OK | Lowers to `var(--control-frame)`; button still contracts-owned |
| `controlLift` | 0 | **contracts** + motion-ish | `var(--control-lift)` translate |
| `press` | 0 | **contracts** / **fx:** | Multi-slot press chrome (`--press-translate/scale/shadow`) |

### Motion-adjacent

| Setter | Calls | Proposed home | Rationale |
|--------|------:|---------------|-----------|
| `transition` | 13 | **stay Style** for micro-PE; prefer **`fx:`** for enter/exit/loop | CSS transition property still valid host paint |
| `duration` | 13 | same | Pairs with transition; fx: has own duration |
| `ease` | 12 | same | |
| `rotate` | 9 | **stay Style** for static pose; **`fx:`** for animated | Icons/chevrons use static rotate |
| `translateX` / `translateY` | 5+5 | **stay Style** micro-nudge; avoid for layout | Layout should not fake offset with translate |
| `transform` | 2 | **stay Style** (bind/canvas) | Full transform value / bind tracks |
| `animate` | 1 | Prefer **`fx:`**; keep `animate(spin)` CSS PE | Lint already treats bare `animate` as motion domain |
| `scale` | 1 | **stay Style** static; **`fx:`** animated | |
| `origin` | 1 | **stay Style** | transform-origin |
| `willChange` | 0 | **stay Style** (escape hatch) or drop | Zero use; performance hint |

### Layout-ish

| Setter | Calls | Proposed home | Rationale |
|--------|------:|---------------|-----------|
| `w` / `h` / `size` | 210 / 132 / 34 | **stay Style** for paint boxes; layout children use parent axis | Intrinsic box size is host paint |
| `minW` / `maxW` / `minH` / `maxH` | 149 / 164 / 77 / 7 | **stay Style** (panels, controls) | Control heights (`control-h-*`) are Style size tokens |
| `grow` / `shrink` / `basis` | 102 / 28 / 18 | Prefer **layout child attrs** when parent is row/column; Style OK on non-layout hosts | Flex item props |
| `self` / `justifySelf` / `order` | 10 / 0 / 0 | **layout element attrs** | Alignment of item in parent |
| `display` | 86 | **stay Style** (show/hide, inline-flex hosts) | Not parent layout |
| `position` + inset (`top`/`left`/…) | 87 + axes | **stay Style** for overlays; discourage for document flow | Absolute/sticky chrome |
| `flexDir` | 3 | **layout element** (`row`/`column`) preferred | Host flex direction on non-layout tags only |
| `gap` / `gapX` / `gapY` | 0 | **layout element `gap=`** primary; Style host-paint fallback | Explicitly dual-pathed in skill; zero Style use → layout winning |

---

## Full inventory by bucket

### pure-paint (98) — 8,111 calls

| Setter | Calls | Setter | Calls | Setter | Calls |
|--------|------:|--------|------:|--------|------:|
| `fg` | 1186 | `fontSize` | 915 | `bg` | 736 |
| `border` | 564 | `radius` | 509 | `pad` | 409 |
| `fontWeight` | 398 | `borderWidth` | 396 | `fontFamily` | 385 |
| `tracking` | 284 | `textCase` | 250 | `padX` | 203 |
| `mb` | 162 | `leading` | 150 | `padY` | 145 |
| `shadow` | 108 | `outline` | 102 | `borderBWidth` | 101 |
| `borderB` | 92 | `mt` | 85 | `padB` | 78 |
| `mx` | 74 | `cursor` | 65 | `padT` | 63 |
| `opacity` | 56 | `overflow` | 48 | `borderTWidth` | 45 |
| `borderT` | 44 | `whitespace` | 41 | `ms` | 39 |
| `textAlign` | 32 | `z` | 24 | `outlineOffset` | 21 |
| `borderL` / `borderLWidth` | 19 | `group` | 19 | `pointerEvents` | 19 |
| `m` | 18 | `overflowY` | 17 | `overflowX` | 15 |
| `select` | 12 | `bgImage` | 11 | `borderS` | 10 |
| `fill` / `stroke` / `list` | 9 | `me` | 9 | `break` | 8 |
| `padS` | 7 | `borderEWidth` / `ml` / `my` / `radiusT` | 6 | `align` / `borderR*` / `content` / `tapHighlight` / `visibility` | 5 |
| `italic` / `padL` / `underline` | 4 | `nums` | 3 | `box` / `isolate` / `overscroll` / `padR` / `saturate` / `scrollbarThumb` | 2 |
| `appearance` / `aspect` / `borderSWidth` / `mixBlend` / `mr` / `radiusB/L/R` / `scrollbarTrack` / `tab` | 1 | | | | |
| **zero:** `accent`, `antialiased`, `bgClip`, `borderE`, `borderX`, `borderY`, `caret`, `color`, `contain`, `decoration`, `indent`, `objectFit`, `objectPosition`, `padE`, `resize`, `ring`, `scroll`, `touch`, `underlineOffset` | | | | | |

### layout-ish (24) — 1,247 calls

| Setter | Calls | Setter | Calls |
|--------|------:|--------|------:|
| `w` | 210 | `maxW` | 164 |
| `minW` | 149 | `h` | 132 |
| `grow` | 102 | `position` | 87 |
| `display` | 86 | `minH` | 77 |
| `top` | 50 | `left` | 44 |
| `size` | 34 | `shrink` | 28 |
| `bottom` | 20 | `right` | 19 |
| `basis` | 18 | `self` | 10 |
| `inset` | 7 | `maxH` | 7 |
| `flexDir` | 3 | `gap` / `gapX` / `gapY` / `justifySelf` / `order` | 0 |

### motion-adjacent (11) — 62 calls

| Setter | Calls |
|--------|------:|
| `duration` | 13 |
| `transition` | 13 |
| `ease` | 12 |
| `rotate` | 9 |
| `translateX` | 5 |
| `translateY` | 5 |
| `transform` | 2 |
| `animate` | 1 |
| `origin` | 1 |
| `scale` | 1 |
| `willChange` | 0 |

### composite-recipe (14) — 61 calls

| Setter | Calls | Slots / lowering sketch |
|--------|------:|-------------------------|
| `colorMix` | 26 | `background-color: color-mix(… token pct%, transparent)` |
| `truncate` | 16 | overflow + text-overflow + white-space |
| `colorDeepen` | 7 | `color-mix(… token, foreground pct%)` |
| `srOnly` | 5 | a11y absolute clip pack |
| `avatarMedia` | 1 | block + 100% + object-fit contain + pad 10% |
| `glyphFill` | 1 | block + 100% width/height |
| `scrollbarThin` | 1 | scrollbar-width thin + scrollbar-color vars |
| `textGradient` | 1 | gradient bg + clip text + transparent color |
| `bgGlow` | 1 | absolute + inset + glow background + PE none |
| `borderImage` | 1 | border-image via corner-image token |
| `backdrop` | 1 | backdrop-filter (+ webkit) via card token |
| `controlFrame` | 0 | `box-shadow: var(--control-frame)` |
| `controlLift` | 0 | `translate: var(--control-lift)` |
| `press` | 0 | press translate + scale + shadow vars |

---

## Documentation coverage (generated skill)

**Skill path:** [`.claude/skills/sail-authoring/SKILL.md`](../../../.claude/skills/sail-authoring/SKILL.md)  
**Generator:** `sail-compiler skill` → iterates `style::setters::setters()` (see `crates/sail/compiler/src/bin/sail-compiler.rs`).

| Check | Result |
|-------|--------|
| Setters in table | 147 |
| `style.NAME(…)` lines in SKILL.md | 146 |
| **Missing from SKILL** | **`colorMix` only** |
| Extra in SKILL not in table | none |

`colorDeepen`, `controlFrame`, `controlLift`, and `press` are listed; **`colorMix` was inserted into the table after the skill was last regenerated** (order in SSOT is `colorDeepen` → `colorMix` → control chrome; skill jumps from `colorDeepen` to `controlFrame`).

**Action:** regenerate skill (`sail-compiler skill`) so `colorMix` appears with arity notes. Optional improvement: skill generator should call out two-arg composites (`colorDeepen` / `colorMix`) explicitly — today it only prints vocab/scale/keywords, so arity is invisible to agents.

---

## Zero-usage setters (28)

```
accent, antialiased, bgClip, borderE, borderX, borderY, caret, color,
contain, controlFrame, controlLift, decoration, gap, gapX, gapY, indent,
justifySelf, objectFit, objectPosition, order, padE, press, resize, ring,
scroll, touch, underlineOffset, willChange
```

Clusters:

1. **Long-tail text/form chrome** (`caret`, `accent`, `decoration`, `indent`, …) — keep for completeness; low cost.
2. **Directional border/pad rarely needed** (`borderX/Y/E`, `padE`) — axis variants exist; product uses T/B/L more.
3. **Layout dual-path winners** (`gap*`, `order`, `justifySelf`) — layout elements absorbed usage.
4. **Tier-2 control chrome not yet adopted in Style** (`controlFrame` / `controlLift` / `press`) — contracts still own L1.

---

## Synthesis notes (for style-synthesis job)

1. **Do not demote pure-paint volume** — Style is working as the paint SSOT; top 10 are all paint/type.
2. **Composites to keep as Style primitives:** `colorMix`, `colorDeepen`, `truncate`, `srOnly`.
3. **Composites that can stay private to one component:** `avatarMedia`, `glyphFill`, `scrollbarThin`, card `backdrop`/`borderImage`.
4. **Wire or drop control chrome:** either migrate button contracts → `controlFrame`/`controlLift`/`press` Style calls, or stop advertising them until used.
5. **Motion boundary is soft:** ~62 Style motion calls vs growing `fx:` usage — document “static pose on Style / temporal motion on fx:”.
6. **Regenerate SKILL.md** — single known doc gap (`colorMix`).

---

## Reproduce

```bash
# Extract setter names
rg -o 'name: "[^"]+"' crates/sail/lint/src/style/setters.rs

# Rough call counts (style.NAME form only; undercounts chained .NAME after first)
rg -o 'style\.[A-Za-z][A-Za-z0-9]*\(' --glob '*.sail' -g '!node_modules' -g '!target' | \
  sed 's/.*style\.//;s/($//' | sort | uniq -c | sort -rn
```

Full chain-aware counts used for this report live in the analysis pass that produced this file (nested `style.` inside `when`/`match` included).
