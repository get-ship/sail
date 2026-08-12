# Operator pairing queue — scaffold_ready

**Updated:** 2026-08-11  
**Branch tip:** `job/release-integrate` (local scaffold residual)

Tags below are **scaffold_ready** / **operator_pending**: contracts, first-class docs knobs, visual `bun run visual <tag>` green. Operator taste / multi-theme approval still open — **not** `done`.

## Queue (B0 + B1 + B2 + B3 + B4 + B5 + B6 + residual)

| Tag | Docs host | Contract | Visual | Notes for pairing |
|-----|-----------|----------|--------|-------------------|
| button | `doc-button` | yes | green | Pilot; disabled knob docsSkip |
| badge | `doc-badge` | yes | green | Rest tone path |
| fieldset | `doc-fieldset` | yes | green | Legend nested only |
| chip | `doc-chip` | yes | green | selected / removable |
| alert | `doc-alert` | yes | green | dismissible remount |
| separator | `doc-separator` | yes | green | orientation × variant × tone |
| spinner | `doc-spinner` | yes | green | rest fg/track |
| skeleton | `doc-skeleton` | yes | green | shape × size |
| icon | `doc-icon` | yes | green | no component.icon leaves |
| progress | `doc-progress` | yes | green | omit value = indeterminate |
| meter | `doc-meter` | yes | green | low/high/optimum bands fixed in stage |
| **label** | `doc-label` | yes | green | size × required; for= specimen |
| **field** | `doc-field` | yes | green | invalid only; hint/error slots |
| **radio** | `doc-radio` | yes | green | value × disabled; grouping = radio-group |
| **radio-group** | `doc-radio-group` | yes | green | no component.radio-group.*; layout host |
| **switch** | `doc-switch` | yes | green | size × tone × checked × disabled |
| **slider** | `doc-slider` | yes | green | single thumb; no range invent |
| **input** | `doc-input` | yes | green | type matrix · size xs–lg · field faces |
| **textarea** | `doc-textarea` | yes | green | size · rows · shared field faces |
| **select** | `doc-select` | yes | green | size · multiple · shared field faces |
| **a** | `doc-a` | yes | green | dual mode bare/button-link; no component.a.* |
| **kbd** | `doc-kbd` | yes | green | zero axes; faces only; operator keycap taste |
| **prose** | `doc-prose` | yes | green | scope host; component.prose.fg; type brand taste |
| **code-block** | `doc-code-block` | yes | green | code · lang; syntaxHighlight |
| **empty-state** | `doc-empty-state` | yes | green | variant × size × optional tone |
| **error-state** | `doc-error-state` | yes | green | variant × size · code chip |
| **error-boundary** | `doc-error-boundary` | yes | green | message + catchRender events |
| **scroll-area** | `doc-scroll-area` | yes | green | orientation only; thumb faces |
| **ticker** | `doc-ticker` | yes | green | duration × reverse; no component.ticker.* |
| **table** | `doc-table` | yes | green | dense · striped · hoverable · bordered |
| **pagination** | `doc-pagination` | yes | green | value + onPageChange; page cells not hub |
| **toggle-group** | `doc-toggle-group` | yes | green | segmented host; no standalone toggle package |
| **tabs** | `doc-tabs` | yes | green | active · orientation; FACE_MERGE tab*/tab-list* |
| **navbar** | `doc-navbar` | yes | green | placement · sticky · bordered · autohide · offset |
| **card** | `doc-card` | yes | green | default\|outline only; no soft/elevated/ghost invent |
| **avatar** | `doc-avatar` | yes | green | size · shape · tone; name/initials |
| **breadcrumb** | `doc-breadcrumb` | yes | green | zero PE props; trail composition only |
| **dialog** | `doc-dialog` | yes | green | size · backdrop · side · dismissible; invoker open |
| **popover** | `doc-popover` | yes | green | placement string; native popover |
| **tooltip** | `doc-tooltip` | yes | green | placement · content |
| **dropdown** | `doc-dropdown` | yes | green | placement · onSelect |
| **hover-card** | `doc-hover-card` | yes | green | placement; open/close dwell |
| **context-menu** | `doc-context-menu` | yes | green | for= target; onSelect |
| **toast** | `doc-toast` | yes | green | tone · duration; onDismiss callback |
| **accordion** | `doc-accordion` | yes | green | open · disabled · name · multiple · onToggle |
| **combobox** | `doc-combobox` | yes | green | placeholder · value · onSelect; list faces combobox.* |
| **theme-picker** | `doc-theme-picker` | yes | green | value · inline · modeOnly; behavior catalog depth operator |

## Alias residual (not first-class host)

| Tag | Kind | Notes |
|-----|------|-------|
| checkbox | alias | No CE; use `<input type="checkbox">` → input detail |

## Explicitly not in queue yet

- Remaining legacy: dropzone (kind action), layout (kind keywords), pack CEs
- Product invent work: card ladder, slider range, loading, avatar/breadcrumb variants, motion doctrine
- Overlay behavior depth (pointer precision, multi-trigger edge cases)
- theme-picker live catalog/behavior (localStorage + effective ids) — scaffold anatomy only
- Playwright knobs→paint

## Checker

```bash
bun apps/sail-site/scripts/check-l1-surface-contract.mjs
# or: cd apps/sail-site && bun run check:l1-surface
# expect 47 contracts after residual peel (3 new hosts)
```
