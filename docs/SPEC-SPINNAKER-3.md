# SPEC-SPINNAKER-3 — the style language, ratified atlas + full architecture

Status: operator-ratified through the 08-13 design sessions. Supersedes
SPEC-SPINNAKER-2 (its locked semantics survive: slot-merge, kind family
key, geometry-only size, duplicate-assignment errors, sx burial). This
is the complete job: language, theme integration, implementation,
testing, migration. Not additive. Not a proof of concept.

## The governing law

Every concern must WIN LEGIBILITY in the common case or die. No
organizing for tidiness. Nothing more verbose than what it replaced.
A grouping earns existence iff (a) multiple properties express one
intent commonly set together, or (b) a named composite carries reuse.
Single-property mechanics stay plain setters — that is the base case,
not a third way.

## 1. The ratified atlas

**Syntax model (dissolved, operator-ratified 08-13):** a style is a
FLAT CHAIN of concern/setter calls. Channels (box/paint/text) are
dissolved — the one vocabulary table derives each call's
classification for emit/clash/merge. The only keyed members are the
states/triggers axis: `when:`, `match:`, `animate:`. Declared and
inline forms are ONE language:

```sail
style navLink = {
  pad({ x: 3, y: 2 }).radius(md).type(label),
  when: { hover: bg(surface-2), current: bg(primary).fg(primary-foreground) },
}

<h2 style(type(heading-2))>Profile</h2>           // inline sugar
<p style(navLink)>                                // named application
<card style(card-chrome, elevation(xl))>          // composition
```

Override bodies are bare chains (`when: { hover: bg(primary).fg(…) }`);
slot-merge semantics unchanged (property/concern-keyed — never were
channel-keyed). `animate:` stays keyed — triggers are a different
axis, not a channel.

### Concerns (slot-object OR theme-registered named composite)

| Concern | Slots | Named examples |
|---|---|---|
| `bg` | `color, image, clip, glow` | `bg(surface)`, `bg(card)` |
| `fg` | `color, caret, accent` (absorbs SVG fill/stroke) | `fg(foreground)`, `fg(muted-foreground)` |
| `border` | `color, width, style, edge, kind` | `border(subtle)`, `border(card)` |
| `size` | `w, h, min-w/h, max-w/h, aspect` — geometry ONLY | `size(full)`, `size(icon-md)` |
| `type` | `family, size, weight, leading, tracking, align, case, wrap` | `type(body-sm)` — theme type table |
| `elevation` | `shadow` | `elevation(sm)` |
| `position` | `scheme, inset, t/b/l/r, z, order` | — (mechanics; usually slots) |
| `play` (in the `animate:` channel) | `preset` \| `props, duration, ease, delay` | `play(colors, fast)`, `play(fade-up)` |

border: `kind` = family key (stroke|outline|ring coexist; same-kind
slot-merges; same-kind twice = clash). `edge` = all|x|y|t|b|l|r|s|e.

### Axis-collapsed families (one name + slots; scalar form kept)

`pad(3)` / `pad({ x: 3, y: 1, t: 4 })`; `margin` (explicit — rare);
`overflow` / `overflow({ x: auto })`; `radius(lg)` /
`radius({ t: lg })`. `gap` stays a layout-element attr — one home.

### Plain setters (unchanged)

`display, grow, shrink, self, dir, opacity, cursor, select,
pointerEvents, z? (rides position), transforms (rotate, scale,
translateX/Y), filters (blur, brightness…), scrollbar faces` +
the marker verbs (srOnly, isolate, appearance, …).

### Value domains

Token-union only, EXCEPT ratified literal slots: `border.width`
(scale | px int), `grow/shrink` (int), `opacity` (pct int),
`z` (scale | int), `duration` (tokens | ms int), transforms
(deg/pct numbers). Enumerated per concern in the compiler table;
anything else literal = compile error.

### The positional rule

Positional args legal IFF args are type-distinct (misorder = compile
error). `animate(colors, fast)` ✓ (prop-set ≠ timing token).
Same-typed args = slot-object only. `animate` is the only current
citizen; additions require the same proof.

### Override semantics

`when:`/`match:` merge at SLOT level within a concern (partial slot
object overrides named slots only; base inherits). Duplicate slot or
overlapping longhand across a style (chain + overrides resolved) =
compile error naming both sites. The chain is the separator inside a
style (commas stay inside slot objects — one punctuation per role).

### Inline sugar (operator-ratified 08-13)

`style(…)` in tag position accepts ANY MIX of named-style refs and
concern/setter calls — the SAME flat chain as the declared form, one
language at two sizes:

```sail
style(type(heading-2))                    // one call
style(margin({ l: auto }))                // slot object
style(navLink)                            // named application
style(card-chrome, elevation(xl), pad(2)) // named style + overrides = composition
```

Rules: stateless only — `when:`/`animate:`/`match:` need the declared
form (or a named style); duplicate property across the args = the
same clash error; arg order is irrelevant (no cascade inside a
style). Sugar never hides intent: the call name IS the intent.

## 2. Animation story

`animate:` keyed member, keyed by trigger: `transition` (reserved —
props animating on state change), `enter, exit, reveal, loop, scroll`.
The CONCERN inside is `play` — `animate: { enter: play(fade-up),
transition: play(colors, fast) }` — the key is named once; the
self-echo (`animate: { enter: animate(…) }`) is dead on arrival.
Slots: `preset` (theme keyframe preset) OR `props` (transition
property-set: colors, opacity, transform) + `duration, ease, delay`.
There is no `with`. Positional `play(fade-up, slow)` is legal under
the type-distinct rule. Keyframes are THEME DATA (preset tables,
studio-editable, mode-aware); no keyframe syntax in the language.
Transforms are static setters, not animation. Hover/press/focus
animate via `transition` + `when:`.

## 3. Theme integration (the real change)

Themes carry, as data: palette → semantic tokens → component tokens →
**concern composites** (`border.subtle`, `card.border`) → **named
spinnaker styles** (multi-concern compositions:
`card-chrome = { bg: surface, border: subtle, radius: lg, … }`)
→ **type table** → **animation table**. All studio-editable, all
mode-aware, all gated (schema, aliases, modes, contrast floors per
member).

**Policy** (`sail.config.jsonc`, existing policy registry):
`styling: open` | `styling: themed`. The three categories, spelled:
- LEGAL in themed: named composites (`border(subtle)`), named styles
  (`style(card-chrome)`), token-union scalars (`pad(3)`, `bg(surface)`,
  `type(body-sm)`) — the base vocabulary is not an invention.
- ILLEGAL in themed: slot objects (`bg({ color: primary })`) and
  local style consts — one-offs belong in the theme; the error
  points at studio.
Autocomplete is theme-first in both modes.

## 4. Compiler architecture

- **One vocabulary table, three row tags.** Every style name lives in
  ONE registry row tagged `concern | axis | plain`. Complete, clash,
  emit, LSP, skill, MCP all walk that table. There is no setters.rs
  "plain list" beside a concern registry — one schema, three tags,
  or we rebuilt the dual registry we came to kill.
- Concerns emit longhands DIRECTLY. Shared low-level emit helpers
  allowed; the legacy setter LAYER (definitions, dispatch, registry
  rows, `allow_legacy` flag, both code paths) is DELETED.
  Proof: grep-zero for legacy setter names in `crates/` — the death
  is in source, not just corpus.
- `z` is position-only. No standalone `z()` setter — the first
  dual-path question in the draft is answered by deletion.
- ONE-TIME CUTOVER, no tombstones (operator doctrine 08-13): the
  rewriter migrates the corpus; the 165 legacy setter names and the
  v2-atlas names (`place`, `fill`, `ink`, `flow`, `space`, `ground`)
  are DELETED from the vocabulary table — after the cut they fail as
  generic unknown vocabulary. Nothing here was ever an official
  version; no one will ever make these errors.
- Teaching budget goes to FOREIGN idioms (permanent, data-driven
  FOREIGN_IDIOMS table): CSS property habits (`border-width`,
  `font-size`, `padding`), tailwind patterns, HTML attrs (`class=`,
  `style=`) — each gets a smart pointer to the v3 form. That is
  teaching another language's users, not embalming our history.
- Slot-merge, clash, positional-typecheck per §1, all fixtured.

## 4b. Contracts the cut must not regress (this week's failures,
written down)

- **Host-attr contract.** Splice copies use-site attrs onto the
  snippet root; layout attrs (`inline`, gap, …) are CONSUMED by the
  host's layout, never inherited by children. Children see host
  layout state only via `group()` + `when: { group-* }`; `match:` is
  same-element only. (The `<theme-picker inline>` popover bug is the
  fixture.) No phase-4 "fix" via el.style — the contract is in the
  compiler, fixtured.
- **Themed paint resolution.** Tokens resolve against the node's
  theme scope (`theme=`, mode) — never global aliases. Substrate is
  compiler-private (vocabulary: `theme=`, `bind()`; data-*/var() are
  output only). The studio chain test asserts COMPUTED CSS per node,
  never class-name spelling.
- **`inline` face, designed or deleted.** An overlay-bearing element
  (theme-picker, popover, dropdown) may expose a sanctioned `inline`
  face = in-flow surface, no floating layer — a designed variant
  prop with documented semantics, not an attr that accidentally
  changes layout. Undesigned inline behavior is a compile error.

## 5. Surfaces (in the machine, not after)

Ambient d.ts, skill tables, MCP `sail_surface`, LSP autocomplete/
hover — generated from the v3 table ONLY. Retired names are
UNOFFERABLE, not merely rejected. Studio: theme type/animation/
named-style tables editable; the chain test (studio edit → registry
→ CSS → static screenshot) covers them.

## 6. Testing

Per concern: slot-object, named composite, override-merge, clash,
literal-domain (accepted + rejected). Foreign-idiom pointers: CSS
property names, tailwind habits, HTML attrs → v3 forms. Generic
unknown-vocabulary error for deleted names (no tombstone fixtures).
Positional rule: both type-distinct cases + same-type rejection.
Policy: `themed` mode fixtures (anonymous rejected, registered
resolves). Animation: every trigger, preset resolution, delay slots.
Drift pins: zero legacy identifiers corpus-wide, zero `sx=`, zero
v2 names. Visual parity: declaration-equivalent vs the NAMED
baseline — release-integrate HEAD after the residual reset (the
green tree the v3 cut starts from). Taste changes ride separate
commits, never the cut.

## 7. Migration plan (full corpus, one machine)

Phases on release-integrate (spinnaker's home), merge after:
1. **Ratify + feel test** — five hand-written components (form,
   overlay, nav, data display, motion-heavy) in the v3 syntax;
   operator approves the READ before compiler work.
2. **Compiler** — v3 table, native emitters, legacy layer deleted,
   foreign-idiom pointers to v3 forms.
3. **Catalog + themes** — composites, named styles, type/animation
   tables; theme JSONs migrate; gate fixtures.
4. **Rewriter v3** — corpus (stdlib, site, studio, components,
   tutorial, fixtures) from v2 form → v3 (renames/slot moves);
   grep-zero proof.
5. **Surfaces + studio** — per §5.
6. **Gates** — three builds, suites, hygiene, parity, all fixtures.
Then: merge release→spike, full battery, archaeology sweep on the
converged tree.

## 8. Completeness addendum (ratification session 08-13)

### Lost-expression coverage (every gap named, none silent)
- **`bind()` arg domain** — the geometry/`use:` crossing is an
  explicit slot-value domain: `position({ t: bind(tt-top) })`. Vocabulary,
  not var(). Any slot may accept a bind where the atlas declares it.
- **Restored plain setters**: `align` (vertical-align), `list`
  (list-style); residual internals `resize`, `contain`, `willChange`,
  `visibility` — plain setters, stdlib-leaning, in the ONE table.
- **Scroll-snap**: `overflow({ x: auto, snap: x })` — L2 carousel's
  prerequisite, in the atlas now or carousel becomes the next *\_MAP.
- **Gradients**: `bg({ image: <gradient-token> })`, `fg` text-gradient
  slot — gradient tokens are catalog data.
- **One-off keyframes**: theme-only by design (accepted loss; the
  filing path is a theme table entry).

### New expressiveness (free because we own the layer)
- **Container `when:`** — `when: { container: md }` component-adaptive
  states (viewport `when:` breakpoints unchanged).
- **Stateful named styles** — theme-registered styles carry `when:`
  variants as theme data; the design system ships interaction.
- **`style(name, partial)` composition** — application with slot-merge
  overrides: `style(card-chrome, { paint: elevation(xl) })`.
- **Correct-by-default emit**: `when: hover` emits hover-capable
  media (no sticky touch hover); `play` presets honor reduced-motion
  automatically. Stated as emit rules, fixtured.

### DTCG stance
DTCG 2025 composite types (border, typography, shadow, transition,
gradient) are the base — the industry landed where we did. Sail
extensions under `$extensions`: `$type: "sail.style"` (named
multi-concern styles), `$type: "sail.keyframes"` (animation presets).
One JSON shape for studio, MCP, gate, generate.

### Studio surfaces (rolled into this spec)
Studio = the visual editor for theme data, not an app builder.
1. **Tables** — type table + animation table with live previews.
2. **Named-style composer** — concern assembly with live preview
   INCLUDING states (edit hover beside rest).
3. **Live gate feedback** — contrast floors while editing, mode
   completeness, alias resolution.
4. **L1 component tokens** — editable per element (l1_manifest SSOT),
   composites as single units; invisible in .sail as ruled.
5. **Publish loop** — draft paint → commit → effective ids → app
   autocomplete; `themed`-mode flow: app compile error → "create in
   theme" → studio scaffolded from the rejected anonymous form.

### Default theme
Ships the COMPLETE named catalog (every composite, type, animation
preset, base named styles). Named forms resolve out of the box;
themes subtract/override from a full set, never build from zero.

## 9. Non-goals

New layout vocabulary; L1/L2 surface decisions (separate session);
raw keyframe authoring in .sail (theme data only); `styling: themed`
as default (open stays default at launch — closure is opt-in).
