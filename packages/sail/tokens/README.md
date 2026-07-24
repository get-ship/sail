# @ship/tokens

Ship design tokens — OKLCH palette, semantic surfaces, tone ramps, role tokens,
motion/density channels, and the theme catalog.

## Source of truth (grouped DTCG)

**Edit** Design Tokens Format (DTCG) JSON under [`model/`](./model/):

| Path | Role |
|------|------|
| `model/base.json` | Base `:root` tokens — **grouped** (`color`, `space`, `radius`, `typography`, `shadow`, `motion`, `control`, …) |
| `model/dark.json` | `.dark` overrides (same group shape) |
| `model/density-*.json` | Density channel overrides |
| `model/reduced-motion.json` | Reduced-motion duration overrides |
| `model/themes/*.json` | Per-theme token overrides (`data-theme` / preview) |
| `model/chrome.css` | Structural CSS for base (body overlays, scrollbars) — not token values |
| `model/themes-chrome.css` | Theme structural flourishes (body textures, keyframes) — not token values |
| `model/compat-aliases.json` | Legacy CSS name → current cssName dual-emit map |

### Groups + public CSS names

Leaves live under groups (e.g. `space.4`, `color.primary.default`,
`color.accent.dashboard`). Public CSS custom properties stay stable via
`$extensions.ship.cssName` on each leaf:

```json
"space": {
  "$type": "dimension",
  "4": {
    "$value": "16px",
    "$extensions": { "ship": { "cssName": "space-4" } }
  }
}
```

→ emits `--space-4: 16px` and registry `{ path: "space.4", cssName: "space-4", … }`.

Domain accents are normal color tokens under `color.accent.*` with cssNames
`accent-*` (no load-bearing `app-` prefix). Legacy `--app-accent-*` names are
dual-emitted as `var(--accent-*)` for PE compatibility.

**Do not hand-edit** generated files:

- `src/tokens.css` — web delivery (`/* GENERATED FILE */`)
- `src/themes.css` — theme overrides + chrome concat
- `src/token-registry.json` — lint / sx registry (paths + colors)

## Generate

```bash
pnpm --filter @ship/tokens generate
# or
node packages/sail/tokens/scripts/generate.mjs
```

Aliases in DTCG use `{css-name}` or `{group.path}` and emit as
`var(--css-name)` so theme override chains stay intact.

## Themes

Themes are **DTCG override data** under `model/themes/<id>.json`. The generate
step emits `src/themes.css` (token blocks + `themes-chrome.css`). This is the
model a future theme studio would edit — not hand-maintained CSS property
lists.

Catalog metadata (id, label, dark, swatches) for pickers remains in
`src/themes.ts`.

## Lint / sx

`crates/sail/lint` loads `src/token-registry.json` as the token **name + color**
SSOT (not a CSS scrape). Registry v2 fields used by tooling:

- `names` / `colors` — cssNames (incl. legacy aliases)
- `tokens[cssName].path` / `.cssName` / `.type` / `.value`
- `paths` — path → cssName map

After model edits, regenerate before running lint tests.

## Workflow

```bash
# 1. Edit SSOT (groups / themes)
$EDITOR packages/sail/tokens/model/base.json
$EDITOR packages/sail/tokens/model/themes/paper.json

# 2. Emit CSS + registry
node packages/sail/tokens/scripts/generate.mjs

# 3. Verify
cd packages/sail/tokens && ./node_modules/.bin/vitest run
cargo test -p sail-lint --lib
```
