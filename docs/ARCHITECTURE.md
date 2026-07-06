# SUI Architecture — the language / userland seam

Status: **governing**. This document is the explicit map: which things are
**language**, which are **standard library**, which are **userland**, and the
seams between them. When code and this document disagree, one of them is a
bug — file it.

Scope discipline: this file states rules the toolchain enforces (each one has
a row in the enforcement map) plus the element inventory. Rationale, decision
trails, status, and work tracking live elsewhere — nothing lands here that
cannot be checked against the compiler.

## The one-sentence architecture

SUI is a UI language whose standard library IS the component set: every
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
    COMPONENTS      (sui-dialog, sui-combobox, sui-tree-grid, sui-toast, …)
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
language element — it compiles away; a hyphenated name (`<sui-dialog>`) is a
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
`@scope (sui-x) to ([data-sui])` donuts so a component styles its own
subtree and stops at nested component boundaries. Never by accident, never
by encapsulation technology.

**Shadow DOM is a containment FEATURE, not the component model.** It is
declared per component for exactly one job: wrapping foreign content whose
styles/DOM we do not own (the tiptap wrapper, embedded third-party widgets,
the FFI boundary). A shadow-wrapped component pays the full isolation tax
(sheet adoption, slot mechanics, internals) knowingly — and the compiler
knows it from the manifest.

**`part` and `::part`/`:host` are not authoring surface.** In light DOM the
host is just the element (`sui-x[attr]`) and "parts" are the anatomy's
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
| Extend vocabularies (sx props, token families, motion presets, theme channels) | yes | no |
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
| sui-stack, sui-cluster | `<stack>`/`<row>`/`<column>` | done (spike) | already deleted |
| sui-center | `<center>` | T1 | keyword exists; absorb the component's residue |
| sui-grid | `<grid>` | T1 | same |
| sui-button | `<button>` | T1 | states, transitions, loading spinner, icon-only — all element-owned |
| sui-input | `<input>` | T1 | + `<textarea>` on the same contract |
| sui-textarea | `<textarea>` | T1 | |
| sui-label | `<label>` | T1 | pairing, required marker, disabled ink |
| sui-checkbox | `<checkbox>` | T1 | shared glyph engine with radio/switch |
| sui-radio-group | `<radio>` + stdlib group wrapper | T1 | glyph is L1; group semantics L2 |
| sui-switch | `<switch>` | T1 | |
| sui-icon | `<icon>` | T2 | kills the ::slotted icon boilerplate |
| sui-spinner | `<spinner>` | T2 | one spin keyframe, language-owned |
| sui-separator | `<separator>` | T2 | |
| sui-avatar | `<img>` reset + stdlib avatar | T2 | image baseline L1; initials-fallback logic L2 |
| sui-scroll-area | `<scroll>` | T3 | scrollbar chrome is deliberately inexpressible elsewhere |
| sui-table | `<table>` family reset + stdlib table | T3 | visual baseline L1; sort/select behavior L2 |
| (new) | `<progress>`, `<visually-hidden>`, `<kbd>`, list resets | T2/T3 | no current dir; born language-level |

### → L2 stdlib components (custom elements, toolchain-shipped)

Overlay/popup: sui-dialog, sui-alert-dialog, sui-sheet, sui-drawer,
sui-popover, sui-tooltip, sui-hover-card, sui-context-menu,
sui-dropdown-menu, sui-toast (+host/container), sui-cmdk, sui-portal.
Selection/input: sui-select, sui-combobox, sui-autocomplete,
sui-toggle-group (+toggle), sui-slider, sui-color-picker, sui-date-picker,
sui-calendar, sui-file-upload, sui-field.
Disclosure/nav: sui-accordion, sui-collapsible, sui-tabs (+tab, tab-panel),
sui-navigation-menu, sui-pagination.
Data/content: sui-tree, sui-tree-view, sui-tree-grid, sui-json-view,
sui-diff, sui-markdown, sui-code, sui-timeline (+item).
Status/display: sui-alert, sui-badge, sui-chip, sui-card, sui-status-dot,
sui-skeleton, sui-error-boundary (runtime-backed).

These keep custom-element identity (behavior, shadow DOM, anatomy manifest)
but move **into the toolchain** and stop being a published userspace package.
Design-vocabulary components (badge, chip, card, alert) are deliberately L2,
not L1: they're design semantics, not platform behavior — but they are still
stdlib, because the stdlib is where taste lives.

### → L3 userland (move out of primitives; the permanent dogfood canary)

sui-account-chip, sui-account-menu, sui-brand-mark, sui-empty-state,
sui-error-state, sui-nav-top, sui-project-switcher, sui-theme-picker.

These encode product decisions. They are the proof the seam works: the day
one of them needs an L1/L2 privilege, the stdlib has a gap — and we find out
in our own code first.

## The seams in detail

**One authoring TECHNOLOGY for every layer (element mode).** L1 elements are
authored in `.sui` like everything else — `<component>{ props }`, template,
variants/sx skin — compiled in **element mode**, which changes only the
output targets: the template becomes a use-site lowering (like the layout
keywords) instead of a registered module; the skin routes into the framework
sheet keyed on the element + attributes; the props block feeds the manifest
AND derives the bare-shorthand vocabulary. Element mode enforces the seam
test as a compile error: element templates are static structure + slots —
no control flow, no `<controller>`. Needing either means the element is a
component. Stdlib element sources compile at toolchain build and bake in;
an element moves between L1 and L2 by changing its compile mode, never by
rewriting its source.

**One authoring API, two implementations (the layer-invisibility rule).**
The single biggest footgun in this architecture: L1 elements are lowered at
compile time and L2 components are custom elements — but the AUTHORED API
must be identical, or the layer split becomes a visible scar every author
trips on. The rule: **an element's contract (attributes, prop bindings,
events, states) is defined once, at the language level; the layer is an
implementation detail the author must never be able to observe.** Concretely:

- `tone="primary"`, `size={x()}`, `disabled`, `on:change`, `bind:value`
  behave identically on `<button>` (L1) and `<sui-dialog>` (L2) — same
  syntax, same dynamics, same reflection semantics.
- Contract styling is keyed on ATTRIBUTES, not compile-time classes
  (`button[tone='primary']`, `:host([tone='primary'])`), so a dynamic
  `tone={expr}` restyles at runtime on BOTH layers. A contract prop that
  lowers to a static class is a bug — that prop just went static-only on L1
  while staying dynamic on L2.
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

**Vocabulary closure (the sx seam).** sx validates against framework
contracts only (`tone-*`, `size-*` channels + the token registry). A
`<theme>` declaration must NOT widen what validates — the `source_channels`
back door closes in Phase 1, and the 72 ad-hoc channels it admitted become
compile errors (that error list = the re-port work list). Userland never
gains vocabulary by authoring; it gains vocabulary when the stdlib grows.

**Fidelity gate (the absorption seam).** A component is absorbed into L1/L2
only at computed-style equivalence with the `feat/next` baseline across
parts × states × variants × sizes, plus green behavior tests. A port that
cannot pass files a language gap; nothing ships degraded.

## The `style:` directive ruling

`style:prop={expr}` (single-property dynamic binding) is **toolchain-
privileged, for data-driven values only** — a user-picked color, a computed
hue, a drag position: values that cannot be tokens *by definition*. It is
not a styling escape: static values in `style:` are a lint error (spend a
token), and userland never gets the directive at all. The lint rides the
policy floor.

## Transitional constructs (compiled today, dying by design)

These still compile so the un-migrated tree keeps building; each dies
per-component at re-port, and its lowering code is deleted after the last
user. None are part of the taggable authoring surface:

| Construct | Successor |
| --- | --- |
| `<prop …/>` XML tags | `<component>{ props }` typed block |
| `<theme prop channels>` | framework contracts + component-token families |
| `<variants part="…">` | `<variants class="…">` / dotted keys (v2) |
| line-DSL `<variants>` bodies | the TS-object form |
| `<host>` directive | the element is the host (light DOM) |
| `part=` attributes | anatomy classes (CEM-published) |
| shadow-by-default emission | `--light` (flips to default at absorption end) |

## Enforcement map

| Seam | Mechanism | Status |
| --- | --- | --- |
| No authored CSS in userland | policy floor (`sui-no-style-block`, `sui-no-inline-style`) | exists (spike) |
| No raw containers | `sui-no-raw-container` → layout keywords | exists (spike) |
| sx closed vocabulary | `sui-sx` lint + token registry single-sourced from tokens.css | exists; channel back door **to close** |
| `<theme>` retirement from userland | compile error outside toolchain | planned (Phase 1) |
| Knobs are marked declarations | manifest key + build-output banner + guarantee flag | planned |
| Anatomy as public API | compiler-emitted anatomy manifest | planned (Phase 2) |
| Naming seam | bare tag = lowered L1; hyphenated = custom element | by construction |
| Layer invisibility (one authoring API) | cross-layer conformance matrix in toolchain tests; contracts attribute-keyed | planned (gates T1 absorption) |
| Fidelity gate | computed-style equivalence harness | planned (Phase 3 entry) |
| Skin conformance | toolchain skin validator | planned (Phase 4) |

## Why the stdlib is the language

Component libraries exist on the web because nobody owns the platform
vocabulary — HTML froze, so every team rebuilds the missing patterns in
userspace and then governs that middle tier forever. SUI owns its compiler,
lint, LSP, and runtime, so the middle tier is removed instead of governed:
interaction patterns are standard library, taste ships as a replaceable
default skin over published anatomy, and userland is composition. Anyone who
wants different taste changes tokens or skins — never component internals.
