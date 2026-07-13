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

## The one seam that makes the floor possible

Runtime geometry is real: a node dragged to 342px, a virtualizer's row offset, a
measured marquee duration. There is no token for those, because they are **data,
not design**. So:

> **`style:` sets CUSTOM PROPERTIES ONLY, never CSS properties.**

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
