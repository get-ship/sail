# sx syntax v0 (lead B)

**Identity:** closed property set + token values from the registry + one IR.  
Not “Tailwind minus delta.” Short property names are fine (`p`, `px`, `bg`, `color`, …).

**Primary surface in `.sail`:** string attribute. JSON later is the same IR.

## Forms (dual-parse)

Both forms may appear in one `sx="…"` string. They lower to the same IR.

### Structured (preferred for new code)

```text
prop: value
prop:value
hover:prop: value
tablet:px: 4
```

Examples:

```sail
<div sx="bg: primary px: 2 rounded: sm hover:bg: muted" />
<div sx="bg: color.primary px: space.2 rounded: radius.sm" />
<span sx="color: tone-fg font-size: xs font-weight: semibold" />
```

### Legacy atoms (still accepted)

```text
prop-value
hover:prop-value
```

```sail
<div sx="bg-primary px-2 rounded-sm hover:bg-muted" />
```

## Values

| Kind | Shape | Example |
|------|--------|---------|
| **Token** | Registry cssName or dotted path | `primary`, `space-2`, `space.2`, `radius.sm` |
| **Keyword** | CSS physics (display/position/…) | `center`, `solid`, `hidden` (bare) |
| **Var ref** | `(--name)` | `w: (--box-w)` |
| **Raw** | `raw:…` only | `bg: raw:#3366ff` |

Rules:

- Token values only by default (registry names/paths).
- Dual resolve: dotted path → kebab cssName, or last segment / scale suffix (`space.2` → `2` on `px`).
- **Raw colors/lengths without `raw:` are errors** (`#fff`, `16px`, `rgb(…)`).
- Bare legacy atoms (`hidden`, `truncate`, `italic`, `relative`) still work.

## Properties

Closed set = utility heads in `sx-vocabulary.json` plus lead-B aliases:

| Alias | Maps to |
|-------|---------|
| `color` | `text` (color scale) |
| `font-size` | `text` (type scale) |
| `font-weight` | `font` (weight) |
| `font-family` | `font` (family) |
| `radius` | `rounded` |

Full table: agent schema (`sx_ir::agent_schema` / `sx-schema.gen.json`) and `sx_vocab::vocabulary()`.

**Not sx properties:** `tone`, `size` (context attrs / variant axes — not paint props).

## Prefixes (one dialect)

Existing prefixes map cleanly into IR:

- **State:** `hover:`, `focus:`, `active:`, `disabled:`, `checked:`, `pressed:`, …
- **Condition / breakpoint:** `mobile:`, `tablet:`, `desktop:`, `dark:`, `motion-reduce:`, …
- **Pseudo:** `before:`, `after:`, `empty:`
- **Group:** bare `group`; `group-hover:`, `group-<state>:`

Structured: `hover:bg: primary`  
Legacy: `hover:bg-primary`

## Compile-time only

`sx` is static. Dynamic `sx={…}` is a hard error. Conditional paint → prop/variant or `{#if}` with static strings.

## Lowering

```text
string → IR (SxDecl*) → atom + prefixes → sx_vocab/scales → atomic class + CSS
```

Legacy and structured share the emit path.

## Agent contract (minimal)

```
Properties: closed table only — do not invent utilities.
Values: registry token (path or cssName) | keyword | raw:… | (--var)
Prefixes: hover:/focus:/… / tablet:/desktop: / before: / group-*
No tone:/size: as sx props.
```
