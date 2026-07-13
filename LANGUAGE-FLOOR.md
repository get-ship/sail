# The floor, and the dial

Status: proposal. This is the seam to ratify before anything else is built —
close the hatches first, and the design space is whatever is left.

## Why the list matters more than the rules

An agent's Tailwind prior is **right about the syntax and wrong about the scope**.
It will write `flex items-center gap-2` in one breath and `el.style.transform` the
moment sx refuses something. So the agent skill's most valuable section is not
what we allow — it is **what we are not**, said plainly:

> sx is Tailwind's syntax with a closed vocabulary. Layout is not in it. Raw
> values are not in it. There is no arbitrary-value escape. Styling from
> JavaScript does not exist.

Honest, and it converts a class of silent failures into a thing the agent simply
knows.

## THE FLOOR — no configuration relaxes these

These are the escape hatches. They are not a tier; they are the language.

1. **A value is a token or a CSS keyword.** Never a raw number, colour, or unit.
   No arbitrary-value bracket (`p-[13px]`), at any tier, ever. This is the law
   the entire design system rests on.
2. **`style=`** — raw CSS in markup.
3. **Imperative styling from JavaScript** — `el.style.x = …`,
   `el.classList.add/remove/toggle`, `setAttribute('style'|'class', …)`,
   `adoptedStyleSheets`, `new CSSStyleSheet`, appending a `<style>` element.
   *This is the hatch we left open.* We closed the template and forgot the
   controller, and it is exactly where a blocked agent goes next.
4. **`!important`** — there is no cascade to fight; sx attaches to one element.
5. **Undeclared stylesheets.** Every `.css` file that exists is named in
   `sui.config.json`. An undeclared companion sheet (`sui-board.css`) is a
   compile error — that is banned CSS living in a file the compiler never sees.

## THE DIAL — `sui.config.json`, a gradient, not two tiers

Strictness is right when it **points at an existing primitive**. That is how we
stay DRY. It is wrong when aimed at the layer that *builds* the primitives, which
is the one place the platform is legitimately needed — and that is what we got
backwards.

| capability | `compose` (apps) | `author` (our packs, L1/L1.5) |
|---|---|---|
| primitives + sx + layout + motion | yes | yes |
| raw containers (`<div>`, `<span>`) | **no** — compose a primitive | yes |
| `class=` | **no** | yes (internal; never in the CEM) |
| scoped `<style>` block | **no** | yes — tokens still enforced, no class selectors |
| the floor above | enforced | enforced |

Our packs are **built**, not compiled by consumers, so `author` is a repo-internal
build setting. No door opens that cannot be shut — which is the whole reason the
sequencing works: **ship closed, open later if ever.** sx can grow rows and ship
codemods forever; `<style>` cannot be un-shipped once it is in the wild.

## RULING: sx is static, forever. Runtime geometry is an action.

Four surfaces become three, and `style:` leaves the language.

| surface | owns |
|---|---|
| **sx** | static style. Always. No slots, no signals, no exceptions. |
| **`use:`** | DOM behaviour — including ALL runtime geometry (`draggable`, `resizable`, `virtualRow`, `rangeFill`, `parallax`). |
| **`motion:`** | choreography — springs, scroll timelines, staggers, FLIP. |
| ~~`style:`~~ | **deleted.** |

Signals in sx (`sx="translate-y-{n}"`) was the wrong answer, and it is worth
saying why, because the reasoning generalises:

- It re-opens arbitrary values. `w-{signal(300)}` is a hardcoded 300px in a
  signal costume, and the compiler cannot see through it.
- It is a **cognitive** escape hatch, which is worse than a technical one: *"I do
  not know the spacing token, I will compute it."* Invisible in review, because
  the output still renders.
- It punctures the one property that makes sx worth having — that an sx attribute
  TELLS YOU what the element looks like, and lowers to a shared, cacheable atomic
  class.

An action has none of those failure modes. Its body is framework code — written
once, reviewed, and the only thing in the system that touches `.style`. An app
author invokes `use:parallax={progress}` with typed params; there is no slot to
smuggle a raw value through, because there is no slot. It names the INTENT
(`use:parallax`) rather than the mechanism (`translate-y-{n}`), so it composes and
stays DRY instead of being re-derived in fifteen places. And it is a CATALOGUE —
addable and deprecatable — where syntax is forever.

The friction is the feature: `use:virtualRow={w.start}` is more to write than
`translate-y-{n}`, which is exactly why nobody reaches for it to dodge learning a
token.

**THE LINE, and it is one line: no generic action, ever.** `use:draggable` is a
primitive. `use:style={{…}}` is the hatch with a ribbon on it. That single rule
replaces a case-by-case argument about which properties count as "geometry" — we
never mint the general mechanism, so there is nothing to erode.

Geometry is not where design systems die; colour, spacing and type are. A one-off
217px column harms nothing. The reason to hold the line anyway is not that the
VALUE is dangerous — it is that the MECHANISM that permits it is the thing that
gets asked for next, for padding ("it is computed too"), and then there is no line
at all.

## The seam this replaces


Runtime geometry is real: a node dragged to 342px, a virtualizer's row offset, a
measured marquee duration. There is no token for those, because they are **data,
not design**. So:

> ~~`style:` sets custom properties only~~ — SUPERSEDED. `style:` is deleted;
> actions own runtime style writes. Kept here because the reasoning still holds
> for what an ACTION may do: a value that comes from measurement or from the user
> is DATA, not design, and it reaches CSS through a custom property the action
> sets.

```html
<!-- data in -->                        <!-- consumed as style -->
<row style:--row-y={w.start}            sx="absolute translate-y-(--row-y)">
```

`style:transform={…}` is refused; `style:--x={n}` is the way. The property stays
in sx where it is auditable and tokenized; only the *number* comes from JS. This
requires sx to read a declared custom property — `translate-y-(--row-y)`, which
is Tailwind v4's own syntax, so the prior transfers.

**With that seam, `el.style` can be closed completely, with nothing lost.** It is
the last piece of the floor.

## What is NOT allowed — the skill's honest list

**Styling**
- raw values of any kind (`13px`, `#3366ff`, `1.35`, `50%`) — spend a token
- the arbitrary-value bracket — it does not exist
- `<style>`, `style=`, `class=`, `!important`
- styling from JavaScript (see the floor)
- selectors of any kind — descendant, child, sibling, pseudo-element
- `::before` / `::after` / `content` — a thing you can see is an element
- `@media` by hand — the condition prefixes (`tablet:`, `motion-reduce:`)
- `@keyframes` in a component — that is `motion:`

**Layout — this is the boundary the Tailwind prior fights hardest**
- `flex`, `grid`, `inline-flex`, `inline-grid`
- `flex-row`, `flex-col`, `flex-wrap`
- `items-*`, `justify-*`, `place-*`
- `gap-*`, `grid-cols-*`, `grid-rows-*`
- `float`, `clear`

  → `<row>`, `<column>`, `<grid>`, `<stack>`, `<center>`. The parent decides
  arrangement; the child decides only its own participation (`self-*`, `grow-*`,
  `basis-*`, `order-*`, `absolute` — those ARE sx).

  `flex items-center gap-2` is one breath of muscle memory and three refusals.
  It must therefore produce **one** diagnostic showing the whole replacement —
  `<row align="center" gap="2">` — not three rejections. The highest-friction
  boundary in the language has to be the place it teaches best.

**Markup**
- a raw `<div>`/`<span>` as a container (at `compose`) — compose a primitive
- `part=` — retired; light DOM has no shadow parts

## Motion is not hover effects

The line, and it is teachable in one sentence:

> **A property easing when it changes is sx. A choreography is `motion:`.**

**sx** — `transition-colors duration-fast ease-out`. Hover backgrounds, focus
rings, an `aria-selected` border, a chevron under `[open]`. Tailwind names them,
the values are tokens, and this is the great majority of UI motion. It needs no
new concept and it is why `motion:` does not have to be a CSS-transition
compiler.

**`motion:`** — the things CSS cannot do, and the things an agent cannot one-shot:
- keyframe animation from a named catalogue (a marquee, a shimmer, a typing dot)
- spring physics
- orchestrated sequences and staggers
- scroll-linked timelines
- FLIP / shared-element transitions
- gesture-driven values
- animating **to a signal** (zoom, pan, a runtime-measured duration)

The runtime is then the **single reduced-motion authority** — no CSS kill switch,
no per-controller `matchMedia`. It creates every animation, so it can honour the
preference once, for all of them.

Buy the runtime (motion.dev's `motion` — vanilla, a few kB, MIT, and a *runtime*
dependency, so it never touches the wasm-in-browser compiler). Do not hand-roll
WAAPI orchestration; we already are, badly, inside `<sui-band>`'s controller.

## Order of work

1. **Close the floor.** The JS-side hatch lint, `style:` restricted to custom
   properties, sx reading `(--var)`, declared stylesheets. Nothing else is
   trustworthy until this is done.
2. **The compound layout refusal** — one diagnostic, the whole replacement line.
3. **The agent skill**, generated from the table, leading with what we are not.
4. **Align the space ramp to Tailwind's** (ours matches through step 6 and
   silently diverges at 7 and 8 — the worst kind of divergence) and extend it
   upward, which also supplies the missing `size` scale.
5. **`motion:` on motion.dev**, with a keyframe catalogue.
6. **The greenfield eval** at `compose`. The metric is not final quality — it is
   **first-attempt success rate and round-trips-to-green**. That is the number
   that says whether an agent is composing or thrashing.


## The payoff nobody else can have: static layout

`taffy` (Rust flexbox/grid/block, wasm-clean — Zed and Dioxus use it) can compute
our box tree AT COMPILE TIME, with no browser. And it works precisely BECAUSE of
the restrictions we have been agonising over:

- sx is atomic and selector-free → **there is no cascade to resolve.** The
  computed style of every element is already known.
- values are tokens → they are real pixels.
- layout is `<row>`/`<column>`/`<grid>` → the layout intent is DECLARED, not
  inferred from four flex properties.
- the tree is the template.

Tailwind cannot do this (arbitrary values, cascade). React cannot (unknown tree).
It falls out of the closed vocabulary as a side effect — an argument for the
constraints that has nothing to do with moralising about DRY.

**What it catches is the class of bug that actually plagues agent-authored UI:**
not "does it compile" but *"it renders and it looks broken."* Zero-height flex
children, unconstrained growth, overflow, 0×0 boxes, a reading measure blown past
its column. A real example from this repo: a tutorial pane grew to **1779px inside
a 900px viewport** because `<row>` wraps by default, so `align="stretch"` never
constrained the pane height and `overflow:hidden` clipped it. That cost hours. The
compiler could have said it in a sentence.

It changes what the compiler IS. Today it says "that is not a token." With this it
says **"your left pane computes to 1779px in a 900px viewport"** — semantic
feedback about the visual result, on every edit, no screenshot, no Chromium. That
is the loop that beats a one-shot React app, and it is the honest answer to
"award-winning capable".

The hard part is TEXT: taffy does the box model, not shaping. Leaf measurement
needs font metrics (cosmic-text / fontdue; Zed does exactly this). Without it we
still catch fixed-size overflow, collapsed boxes and unconstrained growth — most
of the value. Dynamic content computes against representative or min/max bounds.

Additive, and therefore the reversible half: it ships ambitious.
