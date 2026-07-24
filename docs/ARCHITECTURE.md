# SAIL Architecture — the language / userland seam

Status: **governing**. This document is the explicit map: which things are
**language**, which are **standard library**, which are **userland**, and the
seams between them. When code and this document disagree, one of them is a
bug — file it.

Scope discipline: this file states rules the toolchain enforces (each one has
a row in the enforcement map) plus the element inventory. Rationale, decision
trails, status, and work tracking live elsewhere — nothing lands here that
cannot be checked against the compiler.

## The one-sentence architecture

SAIL is a UI language whose standard library IS the component set: every
element ships **behavior + published anatomy + a replaceable default skin**;
userland composes elements with typed props, tokens, and motion presets, and
never writes CSS.

## The four layers

```
L3  USERLAND        apps + product compositions (account-menu, nav-top, …)
                    typed props · tokens · motion presets · full-TS controllers
                    — no CSS, no new vocabulary, no new elements —
────────────────────────────────────────────────────────────── the seam ──
L2  STDLIB          compiler-known custom elements with runtime behavior
    COMPONENTS      (sail-dialog, sail-combobox, sail-tree-grid, sail-toast, …)
                    behavior machinery · anatomy manifest · default skin
L1  LANGUAGE        compile-time lowered, no runtime tag identity
    ELEMENTS        layout (row/column/grid/center/stack) · typography
                    (h1–h6, p, a, code, …) · controls (button, input,
                    checkbox, …) · content (icon, spinner, separator, …)
L0  TOKENS          primitive → semantic → component tiers; six themes today.
                    Values only. The skin (L1/L2 CSS) is the function that
                    consumes them.
```

**The naming seam.** A bare name (`<row>`, `<button>`, `<icon>`) is a
language element — it compiles away; a hyphenated name (`<sail-dialog>`) is a
stdlib component — it exists at runtime. Userland components are also
hyphenated but live outside the toolchain and are subject to L3 rules.

**LIGHT DOM IS THE DEFAULT — for every layer.** L1 elements were always
light; L2 components render into their own element, no shadow root. What
this buys: native form participation (the ElementInternals plumbing mostly
dies), cross-root ARIA works (`label[for]`, `aria-describedby` reach
everything), SSR is plain HTML (no declarative-shadow dance, no hydration
flash), one document cascade (no per-root sheet adoption), and honest
devtools/tests. What it obligates: compiler-managed slot projection (native
`<slot>` is shadow-only), compiler-generated unique IDs for ARIA wiring,
and style containment by construction — component rules compile into
`@scope (sail-x) to ([data-sail])` donuts so a component styles its own
subtree and stops at nested component boundaries. Never by accident, never
by encapsulation technology.

**Shadow DOM is a containment FEATURE, not the component model.** It is
declared per component for exactly one job: wrapping foreign content whose
styles/DOM we do not own (the tiptap wrapper, embedded third-party widgets,
the FFI boundary). A shadow-wrapped component pays the full isolation tax
(sheet adoption, slot mechanics, internals) knowingly — and the compiler
knows it from the manifest.

**`part` and `::part`/`:host` are not authoring surface.** In light DOM the
host is just the element (`sail-x[attr]`) and "parts" are the anatomy's
classes and attributes — published API in the manifest, styleable by skins
directly. The `<host>` directive and `part=` attributes are shadow-era
idioms and retire with it. The anatomy manifest is CEM — the standard
custom-elements.json schema, enriched with the anatomy fields (parts as
class contracts, states, slots-as-projection-points, layer) — so standard
tooling reads our contracts for free.

## The leverage gradient

Each layer up trades surface for leverage, and the budgets point in opposite
directions:

- **L0 tokens are deliberately RICH.** Expressiveness is the point — full
  families with sensible defaults, so a brand almost never needs skin
  authorship. When a real design decision surfaces, the default answer is
  "it becomes a token (or joins a family)", not "it becomes an API knob".
  Prefer one generalized family over per-element knobs: `--overlay-*`
  serving dialog/drawer/sheet beats `--dlg-*` + `--dr-*` + `--sheet-*`.
- **L1/L2 surfaces are deliberately LEAN.** Every element, prop, variant,
  part, and event must earn its place; two similar props are one prop too
  many. When in doubt, a token, a contract, or composition absorbs the
  decision — the API does not grow. Less is more, and it compounds: a lean
  L2 surface is what keeps L3 authoring (and agent authoring) decidable.

## The seam test

Where does a thing go? Ask in order:

1. **Is it a value?** → L0 token. (A color, a radius, a duration, a control
   height.)
2. **Is it structure/surface fully derivable at compile time — at most
   wrapping one native element?** → L1 language element. (A button is a styled
   native `<button>`; a row is a flex div; an icon is a sized `<svg>` slot.)
3. **Does it need runtime state or orchestration — focus traps, listbox
   state, virtualization, portals, timers?** → L2 stdlib component.
4. **Does it encode a product decision — what's in an account menu, what the
   brand mark is?** → L3 userland. The stdlib must never know your product.

And the styling analogue:

- Which token lands on which part in which state → **skin** (L1/L2, authored
  in the toolchain only, written against the anatomy contract).
- Which values the skin consumes → **tokens** (L0, the sanctioned axis of
  user expression; themes are token vectors).

## The charter — who may do what

| Capability                                        | Toolchain (L1/L2) | Userland (L3) |
| ------------------------------------------------- | ----------------- | ------------- |
| Authored CSS (style blocks, pseudo-elements, keyframes) | yes         | **never**     |
| Define new elements / tags                        | yes               | no (FFI only) |
| Extend vocabularies (sx props, token families, motion presets, shared tone/size prop unions) | yes | no |
| Interaction machinery (focus, anchoring, virtualization, form internals) | yes | no |
| Compose elements, typed props                     | yes               | yes           |
| Theming (all token tiers)                         | yes               | yes           |
| Motion presets by name                            | yes               | yes           |
| Controllers (full TypeScript behavior)            | yes               | yes — unrestricted |

The restraint is **visual, not behavioral**. Userland logic has no
framework-imposed limits.

**Knobs.** The compiler's policy knobs can re-open styling surface per
package. Flipping one is an explicit, manifest-visible declaration that
**voids the guarantee** (a11y floor, theme coherence, agent-safety) for that
package — Rust-`unsafe` semantics: same language, one permission bit, socially
expensive. Never a quiet setting, never a third authoring form.

**FFI.** Genuinely novel UI (canvas charts, editors) embeds a foreign custom
element inside a declared boundary: size contract in, theme-token bridge in,
nothing else out. The fence is stdlib-governed; inside it, anything.

## Element inventory — destiny of every current primitive

Classification of all 70 primitive dirs on `feat/next` (the baseline).
"Wave" is the absorption order: T1 controls → T2 content → T3 chrome →
interaction patterns.

### → L1 language elements (lowered; component dir eventually deleted)

| Today | Becomes | Wave | Notes |
| --- | --- | --- | --- |
| sail-stack, sail-cluster | `<stack>`/`<row>`/`<column>` | done (spike) | already deleted |
| sail-center | `<center>` | T1 | keyword exists; absorb the component's residue |
| sail-grid | `<grid>` | T1 | same |
| sail-button | `<button>` | T1 | states, transitions, loading spinner, icon-only — all element-owned |
| sail-input | `<input>` | T1 | + `<textarea>` on the same contract |
| sail-textarea | `<textarea>` | T1 | |
| sail-label | `<label>` | T1 | pairing, required marker, disabled ink |
| sail-checkbox | `<checkbox>` | T1 | shared glyph engine with radio/switch |
| sail-radio-group | `<radio>` + stdlib group wrapper | T1 | glyph is L1; group semantics L2 |
| sail-switch | `<switch>` | T1 | |
| sail-icon | `<icon>` | T2 | kills the ::slotted icon boilerplate |
| sail-spinner | `<spinner>` | T2 | one spin keyframe, language-owned |
| sail-separator | `<separator>` | T2 | |
| sail-avatar | `<img>` reset + stdlib avatar | T2 | image baseline L1; initials-fallback logic L2 |
| sail-badge | `<badge>` L1 element | done | live in `stdlib/badge.sail` (element mode); not dissolved |
| sail-link | bare `<a>` + sx | dissolved | JS-free; decoration/hover styling (no sx `decoration` prop) and the external-link sr-only text dropped as cosmetic — zero usages in-repo |
| sail-code | bare `<code>` (inline) / `<div>`+`<header>`+`<separator>`+`<pre>` (block) + sx | dissolved | JS-free; the block-mode header's border-bottom (sx has no per-edge border) is composed with the L1 `<separator>` element instead, and its uppercase transform (sx has no `text-transform`) moves into the template expression (`{lang().toUpperCase()}`) — both losslessly, no stdlib skin needed |
| sail-scroll-area | `<scroll>` | T3 | scrollbar chrome is deliberately inexpressible elsewhere |
| sail-table | `<table>` family reset + stdlib table | T3 | visual baseline L1; sort/select behavior L2 |
| (new) | `<progress>`, `<visually-hidden>`, `<kbd>`, list resets | T2/T3 | no current dir; born language-level |

### → L2 stdlib components (custom elements, toolchain-shipped)

Overlay/popup: sail-dialog, sail-alert-dialog, sail-sheet, sail-drawer,
sail-popover, sail-tooltip, sail-hover-card, sail-context-menu,
sail-dropdown-menu, sail-toast (+host/container), sail-cmdk, sail-portal.
Selection/input: sail-select, sail-combobox, sail-autocomplete,
sail-toggle-group (+toggle), sail-slider, sail-color-picker, sail-date-picker,
sail-calendar, sail-file-upload, sail-field.
Disclosure/nav: sail-accordion, sail-collapsible, sail-tabs (+tab, tab-panel),
sail-navigation-menu, sail-pagination.
Data/content: sail-tree, sail-tree-view, sail-tree-grid, sail-json-view,
sail-diff, sail-markdown, sail-timeline (+item).
Status/display: sail-alert, sail-chip, sail-card, sail-status-dot,
sail-skeleton, sail-error-boundary (runtime-backed).

Historical destiny rows above are incomplete. Many of these are already
bare L1 tags in `stdlib.rs` (chip, alert, card, tabs, tooltip, …) with
light DOM as the author surface. Treat the registry as truth; rewrite
this section when absorption inventory is a feature doc, not a blog table.

### → L1 inventory note (live stdlib, not destiny table)

Compiler `stdlib.rs` currently registers ~42 bare L1 tags including
`badge`, `empty-state`, `error-state`, `theme-picker`, `navbar`, tabs family,
etc. **That registry is the truth**, not the historical feat/next destiny
rows above.

L1 admission bar: domain-neutral, atomic, useful to any app. Product/domain
nouns (account, project, billing, brand) stay out of the language and live
in consumer packs. `theme-picker` / empty / error in stdlib are domain-neutral
UI patterns under that bar (revisit if they grow product semantics).

### → Product / domain packs (userland; not language)

account-menu, nav-top, project-switcher, brand-mark, account-chip, and any
composition that encodes org product nouns.

## The seams in detail

**One authoring TECHNOLOGY for every layer (element mode).** L1 elements are
authored in `.sail` like everything else — scope/props, template, variants/sx
skin — compiled in **element mode**, which changes only the output targets:
the template becomes a use-site lowering (like the layout keywords) instead
of a registered module; the skin routes into the framework sheet keyed on
the element + attributes; the props block feeds the manifest AND derives the
bare-shorthand vocabulary. JS enhancement rides **stacked `use:<behavior>`
stamps** on the static structure, lowered to `runAction` at the use site by
the normal action pipeline. Element mode enforces the seam test on the
*template* as a compile error: element templates are static structure +
slots — no control flow, no dynamic attributes/text. A `<controller>` block
is rejected: that is the L2-component behavior channel. Without JS the
static HTML/CSS still ships (progressive enhancement); with JS the stamped
behaviors enhance ARIA/keyboard through the standard `use:` machinery.
Stdlib element sources compile at toolchain build and bake in; an element
moves between L1 and L2 by changing its compile mode, never by rewriting
its source.

**`use:` is a mix-in API** — and the L1 behavior channel. Generic node
behaviors (autosize, clickOutside, geometry bridges) and element interaction
alike ride `use:`; an L1 element stamps the reusable behaviors it needs on
its static structure (chip: `use:pressable use:pressed use:dismissible`) and
the use-site pipeline attaches them.

**One authoring API, two implementations (the layer-invisibility rule).**
The single biggest footgun in this architecture: L1 elements are lowered at
compile time and L2 components are custom elements — but the AUTHORED API
must be identical, or the layer split becomes a visible scar every author
trips on. The rule: **an element's contract (attributes, prop bindings,
events, states) is defined once, at the language level; the layer is an
implementation detail the author must never be able to observe.** Concretely:

- `tone="primary"`, `size={x()}`, `disabled`, `on:change`, `bind:value`
  behave identically on `<button>` (L1) and `<sail-dialog>` (L2) — same
  syntax, same dynamics, same reflection semantics.
- Contract styling is keyed on ATTRIBUTES, not compile-time classes
  (`button[tone='primary']`, `[tone='primary']` on light components), so a
  dynamic `tone={expr}` restyles at runtime on BOTH layers. A contract prop
  that lowers to a static class is a bug — that prop just went static-only
  on L1 while staying dynamic on L2. L1 paint is attribute-keyed on the
  element itself (light DOM); `:host` is not an L1 author or paint path.
- Events: the author writes `on:change` everywhere; the compiler wires it to
  whatever the layer emits (native event on L1, CustomEvent on L2). Event
  payload shapes are part of the published contract.
- The anatomy manifest records the contract for BOTH layers in one format —
  a layer move (e.g. a component graduating from L2 to L1) changes the
  manifest's `layer` field and nothing an author wrote.

Enforced by a **cross-layer conformance matrix** in the toolchain tests: for
every shared contract surface, the same template fragment runs against an L1
element and an L2 component and must produce identical observable behavior
(attribute reflection, dynamic updates, event delivery, disabled/focus
semantics). A contract feature that cannot pass on both layers does not ship
on either.

**Anatomy contract (the skin seam).** Every L1/L2 element publishes parts,
state attributes, and slots as versioned public API, emitted by the compiler
(extends the existing `custom-elements.json`). The default skin is the
reference implementation of that contract; alternative skins are stylesheets
written against it. A skin cannot add parts or states — only restyle them.
Skin conformance (focus-visible present, disabled states present, contrast,
reduced-motion honored) is toolchain-validated; passing it is what keeps the
guarantee alive under full visual replacement.

**Token tiers (the theme seam).** primitive (`--space-3`) → semantic
(`--surface-2`, `--tone-primary-bg`) → component (`--button-hover-lift`,
`--control-transition`). Rule of thumb: every skin decision that a brand
might want different goes in the component tier; every skin decision that
defines what the element *is* stays in the skin. The six existing themes must
stay expressible as pure token vectors at all times.

**Vocabulary closure (the sx seam).** sx validates against the closed token
registry and sx property set only. **One variance pattern:** CVA-in-props
axes (`variant` / `size` / `tone` as ordinary key→sx maps) + optional
`compound:`. Shared `tone`/`size` unions are stdlib convention (ordinary
props). `<theme>` is hard-refused (unknown element; never widens what
validates). Userland never gains vocabulary by authoring; it gains
vocabulary when the stdlib grows. See mission plan
`m-deck-standalone/scratch/FANOUT-design-system-poc.md` (locked product
rules).

**Fidelity gate (the absorption seam).** A component is absorbed into L1/L2
only at computed-style equivalence with the `feat/next` baseline across
parts × states × variants × sizes, plus green behavior tests. A port that
cannot pass files a language gap; nothing ships degraded.

## `use:` — author mix-in API (not the L1 brain)

```
use:<name> / use:<name>={param}

Attaches a mix-in to the mounted host node.
name resolves to an Action in scope (import or local).
Action: (node, param) => { update?(param); destroy?() } | void
```

- **Is:** a node-lifetime mix-in API authors extend. Geometry that cannot be
  a token (drag offset, virtualizer start, measured height) crosses JS by
  writing a **`--*` custom property** the skin/sx reads (`(--row-y)`,
  `left: var(--anchor-left)`, …).
- **Is not:** app state; not a styling/layout channel. Element-owned
  interaction is expressed as stacked, reusable behaviors the element stamps
  — never a per-element private `use:<tag>` mix-in.
- **Registry (`action_manifest`):** curated builtins/examples that ship with
  the runtime — same Action API and the same floor as author Actions, not a
  privilege tier.
- **No-JS honesty:** an Action simply does not run; static lowered HTML/CSS
  still ships. Same progressive-enhancement reality as any controller.

**Action body floor** (`sail-action-no-style` on
`packages/sail/runtime/src/actions/**`):

| Refuse | Allow |
| --- | --- |
| `.classList` mutation, `.className`, `set/removeAttribute('class'\|'style')` | listeners, focus, measure (RO/IO), DOM queries |
| `.style.*` except `setProperty`/`removeProperty` on `--*` names | `style.setProperty('--x', …)` / `removeProperty('--x')` |
| plain CSS property names in `setProperty('height', …)` | ARIA / `role` updates |

Bench harness sources under `tests/perf/` are path-exempt (interop may need
`class=`). Controllers remain on the stricter `sail-no-imperative-style`
(no `.style` at all — reflect via signals + `(--x)`).

## `style:` is dead — the `(--x)` binding replaced it

`style:prop={expr}` is gone: the parser refuses it outright, with a
diagnostic that teaches the replacement. Runtime geometry — a value that
cannot be a token *by definition* (a drag position, a virtualizer offset, a
user-picked color) — crosses JS through a named, typed `use:` action
(`use:virtualRow`, `use:eventSpan`, `use:rect`, `use:pin`, `use:zoomPan`,
`use:worldTransform`, `use:maxWidth`, `use:swatch`, …), which writes ONE
custom property; sx reads it back with `(--name)`
(`translate-y-(--row-y)`, `top-(--evt-top)`).

Ownership stays where it already lives — no new grammar:

- **Scope owns PROPS** (parent-granted config). No state block in scope;
  state is code, not a declaration table.
- **The controller owns state** — `const hue = signal(220)` in `setup()`,
  returned.
- **An action owns its runtime-geometry property** — `--row-y`,
  `--evt-top`, and the rest are fixed names a `use:` action writes; they
  are neither a prop nor a signal of the component that reads them.

**Resolution law for `(--x)` in sx**: global token vars (including the
action-owned runtime-geometry names, `sail_lint::tokens::CONVENTION_TOKENS`)
→ declared props → setup-returned signals → refusal, with a did-you-mean
(`check::sx_vars`). A name never read via `(--x)` gets no CSS wiring at
all — the compiler only wires names actually referenced.

When `(--x)` matches a declared prop or a setup-returned signal, the
compiler emits a CSS `@property` registration (typed from the prop's
declared type / the signal's inferred type — a `number` gets
`syntax: '<number>'`, so a CSS transition on it tweens natively with zero
JS) and reflects the value onto the host as an inline custom property,
seeded at mount and kept live on every write
(`emit::styles::register_properties_install_js`, the runtime's
`registerCustomProperties`). `el.style` stays closed — actions gain no
styling powers beyond the one property they own.

## No shims

The migration doctrine: **either we want the code or we remove it.**
Transitional constructs die when their last user converts; lowering code is
deleted the same day — not deprecated, not shimmed, not aliased. No
compatibility layers; no "legacy mode"; git history is the just-in-case. A
trim pass runs at every milestone: code that no longer earns its place is
removed in the same breath that adds its successor.

## Refused constructs (not author surface)

| Construct | Status |
| --- | --- |
| `<theme>` | **refused** — unknown element; never widens sx; tone/size are ordinary props / CVA axes |
| author `:host` / `::part` / `part=` | **refused** as author surface (light DOM: host is the element; anatomy is classes/attrs) |
| `<prop …/>` tags, `prop:` / `attr:` directives | **refused** — plain `name={expr}` field binding |
| `<variants>` tags under strict policy | **refused** — CVA-in-props on `props:` (gating policy is config surface) |
| `<host>` | **refused** — element is the host (light DOM) |

Unknown `<scope>` members fail generically (not a memorial list of former fields).

Emit may still dual-target some CSS selectors internally; that is not author surface.

## Enforcement map

| Seam | Mechanism | Status |
| --- | --- | --- |
| No authored CSS in userland | policy floor (`sail-no-style-block`, `sail-no-inline-style`) | exists (spike) |
| No class/style in Action bodies | `sail-action-no-style` on `runtime/src/actions/**` (`--*` setProperty only); `tests/perf/**` exempt | exists |
| No raw containers | `sail-no-raw-container` → layout keywords | exists (spike) |
| sx closed vocabulary | `sail-sx` lint + token registry single-sourced from tokens.css; variance = CVA-in-props + compound only | exists |
| `<theme>` hard-refuse | unknown element (preprocess does not revive it); never widens sx | exists |
| Knobs are marked declarations | manifest key + build-output banner + guarantee flag | planned |
| Anatomy as public API | compiler-emitted anatomy manifest | planned (Phase 2) |
| Naming seam | bare tag = lowered L1; hyphenated = custom element | by construction |
| Layer invisibility (one authoring API) | cross-layer conformance matrix in toolchain tests; contracts attribute-keyed | planned (gates T1 absorption) |
| Fidelity gate | computed-style equivalence harness | planned (Phase 3 entry) |
| Skin conformance | toolchain skin validator | planned (Phase 4) |

## Why the stdlib is the language

Component libraries exist on the web because nobody owns the platform
vocabulary — HTML froze, so every team rebuilds the missing patterns in
userspace and then governs that middle tier forever. SAIL owns its compiler,
lint, LSP, and runtime, so the middle tier is removed instead of governed:
interaction patterns are standard library, taste ships as a replaceable
default skin over published anatomy, and userland is composition. Anyone who
wants different taste changes tokens or skins — never component internals.
