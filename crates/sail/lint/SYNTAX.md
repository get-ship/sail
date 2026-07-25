# sx / Style syntax (product)

**Identity:** closed Style setter set + token values from the registry + one IR.  
Not “Tailwind minus delta.” Not utility-string atoms.

**Primary surface in `.sail`:** Style expressions on `sx={…}`:

```sail
<div sx={style.bg(primary).padX(2).radius(sm).when({ hover: style.bg(muted) })} />
<span sx={style.fg(tone-fg).fontSize(xs).fontWeight(semibold)} />
<button sx={when({ pressed: style.bg(foreground).fg(background) })}>…</button>
```

Also: scope Style consts (`const filled = style.bg(…)`) referenced as `sx={filled}`,  
lists `[ match(…), when(…), style.… ]`, `match(prop, { key: style.… })`, `compound(…)`,  
and `bind(name)` for runtime custom properties.

## Residual (refused)

Utility-string product is a **hard error** at parse + lint:

```text
sx="bg-primary px-2"          ✗ residual
sx="bg: primary px: 2"        ✗ residual
sx={style.bg(primary).padX(2)} ✓ product
```

Message points authors at Style chains / `match` / `when` / `compound`.

## Values

| Kind | Shape | Example |
|------|--------|---------|
| **Token** | Registry cssName | `primary`, `space` ramp `2`, `radius` `sm` |
| **Keyword** | CSS physics (display/position/…) | `center`, `solid`, `hidden` |
| **Bind** | `bind(name)` | `style.w(bind(boxW))` → `var(--box-w)` |
| **Raw** | only via residual diagnostics (not product) | — |

Rules:

- Token values only by default (registry names).
- **Raw colors/lengths are errors** (`#fff`, `16px`, `rgb(…)`).
- Bare composites via Style: `style.truncate()`, `style.position(relative)`, `style.display(none)`, `style.italic(true)`.

## Setters

Closed product table in `style::setters` — heads re-home the vocab utilities:

| Setter | Vocab head |
|--------|------------|
| `bg` | `bg` |
| `fg` / `color` | `text` (color) |
| `fontSize` | `text` (type scale) |
| `fontWeight` | `font` (weight) |
| `fontFamily` | `font` (family) |
| `radius` | `rounded` |
| `pad` / `padX` / … | `p` / `px` / … |
| `border` / `borderWidth` | color / width disambiguation |

Full table: `style::setters::setters()` and agent skill (Style section).

**Not Style paint channels:** do not reintroduce tone/size as paint props — they stay ordinary props / CVA axes when used.

## Conditions (`when`)

```sail
style.pad(2).when({
  hover: style.bg(muted),
  pressed: style.bg(primary),
  tablet: style.pad(4),
  motionReduce: style.shadow(none),
  groupHover: style.fg(primary),
  stateLeft: style.left(bind(portLeft)),
})
```

Keys map to state pseudo/attrs, design-system breakpoints, motion prefs, `group-*`, and `state-*` the same way residual string prefixes did — SSOT is `style::conditions` / `when_target`.

## Compile-time only

`sx` is static Style. Dynamic `sx={jsExpr}` (ternary, call) is a hard error. Conditional paint → `when` / `match` / prop / `{#if}` with static Style.

## Lowering

```text
Style expr → style::parse → IR → style::emit → atomic class + CSS
```

Internal leaf emit still uses `sx_ir` / vocab for paint ops; authors never write utility strings.

## Agent contract (minimal)

```
Form: sx={style.…} | when({…}) | match(…) | compound | Style const ref
Setters: closed table only — do not invent utilities.
Values: registry token | keyword | bind(name)
when keys: hover/focus/… / tablet/desktop / before / group-* / state-*
No utility-string atoms. No tone:/size: as Style paint props.
```
