# sx — the design

Status: settled in principle, unbuilt. The vocabulary lives in
`crates/sui/lint/sx-vocabulary.json`; this is the argument for it.

## Identity

Bootstrap improves the defaults. Flutter replaces the platform. Tailwind gives
you the platform and re-decides everything, per element, forever.

**SUI is the web platform with a closed vocabulary.** You write HTML. Every value
is a system decision, every structure is a primitive, and the compiler refuses
anything that isn't. Not a new abstraction — it's HTML. Not merely better
defaults — you cannot opt out.

The knob is the product: how much of the platform is in reach is a compiler
setting, per layer. Nobody else has that. Tailwind can't be narrowed. Panda can't
stop you writing CSS beside it.

## The observation the whole thing rests on

We had a full-featured component library and agents did not reach for it. They
wrote more CSS.

That is not a design opinion, it is a measurement, and it outranks every argument
about principled seams. **Agents compose when composing is the only door.** Every
open hatch is a hatch they take. So the doors close — and the diagnostic points
at the door that is open, by name. The mission is never "find another way." It is
always "use this."

## What went wrong, and it wasn't strictness

We aimed the strictest policy at the layer that *builds* the primitives — the one
layer that legitimately needs the platform, because somebody has to author the
thing everyone else composes. Then the framework quietly exempted itself in
`contracts.css` (250 lines of exactly the CSS a component gets an error for
writing) because it had no choice.

The proof is in the gap list from the five-pack migration. Go down it —
`object-fit`, `transform-origin`, `touch-action`, a 3px rail, 48px thumbnails,
`opacity: 0.85`, a ±1deg tilt, `font-variant-numeric`, `:empty`. **Not one is
something an app author would ever hit.** An L3 agent composing
`<media-image fit="cover">` never touches `object-fit`. Every single gap was a
primitive-authoring gap.

The language wasn't failing at its job. We were pointing it at the wrong job —
and migrating pre-existing, CSS-shaped components is the hardest and least
representative test there is. A component designed in CSS fights anything that
isn't CSS, forever. We proved nothing about greenfield authoring, which is the
actual thesis, and tested none of it.

## The layers

Our packs are **built**, not compiled by consumers. Nobody outside this repo ever
runs the compiler over `sui-primitives`. So the relaxed policy is a repo-internal
build setting, not a public surface — and no door is opened that can't be shut.

- **`compose`** — apps. Where agents live. Doors welded: primitives, sx, layout
  elements, `motion:`. No CSS, no `class`, no `style=`, no raw containers. This is
  where beautiful-by-default and token-efficiency come from, and closing the doors
  is *how* you get it, because the only path left is composition.

- **`author`** — L1/L1.5: the primitives themselves, and the framework sheet.
  The platform is available: scoped `<style>`, internal classes, the full property
  space. Still enforced: **tokens only**, no global CSS, no exported hooks. Small,
  rare, reviewed.

Most of what L1 was escaping to turns out to be vocabulary we simply never wrote
down. Once the table below exists, the honest remainder for `author` is small:
`@keyframes`, `:empty`, `::slotted`, and selectors over semantics.

## The rule

> **Every sx value is a design token or a plain CSS keyword.
> Never a raw number, never a colour, never a unit.**

Two kinds of utility, and the distinction is the whole language:

**Scale-valued.** The value IS a design decision — colour, spacing, size, radius,
type, weight, shadow, duration, easing, z-index, border-width, opacity, angle.
A token from that scale, and nothing else. `16px` is an error. `#3366ff` is an
error. `var(--i-invented-this)` is an error.

**Keyword-valued.** The value is physics, with no design opinion in it —
`display`, `position`, `overflow`, `object-fit`, `white-space`, `cursor`,
`touch-action`, `transform-origin`, `user-select`. The CSS keywords. All of them.
No allowlist.

**The allowlist was the mistake. The value discipline is the point.** Banning
`object-fit: cover` protected nothing — there is no design decision inside it,
nothing to tokenize, no way for two authors to diverge. It bought zero
consistency and cost a component.

## The names are Tailwind's

```html
<div sx="p-4 bg-surface-2 rounded-md object-cover
         hover:bg-surface-3 tablet:p-6">
```

Because that is the prior every agent already has, and because it is terse — an
agent pays for every character, and we halved our component files with it.

Not CSS property names (`padding: space-4`), even though the CSS prior is more
*exact*: it costs 6× the characters, and Tailwind has already named the entire
mechanic space (`object-cover`, `select-none`, `origin-top-left`, `touch-none`,
`whitespace-nowrap`) — every "gap" I catalogued, Tailwind already has a word for.

The failure mode — `p-4` meaning Tailwind's 1rem rather than ours — is weak: it
still lands on **our scale**, so it stays system-consistent. The agent may be a
step off from its intent; the design system absorbs that. A wrong step on a scale
is nothing like `#3366ff`.

And it is not irreversible in the way `<style>` is. We own the compiler: sx can
grow rows, rename fields, ship codemods. `<style>` cannot be un-shipped once it
is in the wild. **That asymmetry decides the sequencing: ship closed.**

## The scales that don't exist — and are why every wall was a wall

`size-*` above ~40px (both media components wanted 48px; the canvas wanted 180 and
320; thumbnails are unsized in `main` right now). `border-*` (every border lowers
to 1px; a 3px category stripe is unsayable). `angle-*` (`rotate` speaks
0/45/90/180/270; a 1deg tilt is dead). `opacity-*` steps finer than 25.

Token-registry work, not language work. But **a closed vocabulary is only fair if
the vocabulary is complete** — without a ramp to spend, every miss is a wall, and
the author goes looking for a hatch.

## Transitions are sx. `motion:` is GSAP.

Tailwind already names them, the values are tokens (`transition-colors
duration-fast ease-out`), and a property easing when it changes is the single most
common motion in real UI. Putting it in sx fixes that hole with **zero new
concepts**.

Which frees `motion:` to be what it was always meant to be: the thing CSS
*cannot* do — spring physics, orchestrated sequences and staggers, scroll-linked
timelines, FLIP and shared-element transitions, animating to a signal. That is
also where an agent is helpless: it can one-shot a card, it cannot one-shot scroll
choreography. **Buy the runtime** (motion.dev's `motion` — vanilla, a few kB, MIT,
and a *runtime* dep, so it never touches the wasm-in-browser compiler) rather than
hand-rolling WAAPI, which we are already doing badly inside `<sui-band>`.

## Not Panda, not Sprinkles

The compiler runs as **wasm in the browser** — the tutorial playground compiles
`.sui` in the reader's tab. Both need a Node build, so adopting either forks the
pipeline. They are also TS function calls, and our surface is a template attribute
parsed by Rust. And they give atoms, not law: Panda won't stop anyone writing
`style=` beside it, and its conditions are open (`_whatever: '[data-foo] &'`),
which hands the back door straight back.

Steal the one thing they get right: **the vocabulary is a config, not code.**
Today `sx.rs` hand-writes every atom in four places, which is why forty gaps felt
like forty projects. One table, and the lint, the TS types, the docs, the LSP
completions and the agent SKILL.md all read it. Adding a property becomes one row.

## What ships, and in what order

The rule that decides everything under a deadline:

> **Irreversible things ship minimal. Reversible things ship ambitious.**

Syntax is irreversible: sx, the layout elements, the SFC model, one `motion:`
shape, the policy names. Get those right and small. Primitives, presets, token
scales and docs are freely addable — that is where the ambition goes, and where
motion.dev/reactbits-level actually lives. Those aren't languages; their value
*is* the catalogue.

1. **Table-driven sx, Tailwind names.** The one irreversible piece. ~3–5 days: the
   engine change is a day (the scanner, prefixes, atomic emission and dedup all
   already work); the table is largely transcription.
2. **Diagnostics that redirect.** Every closed door names the open one. This is
   the highest-value thing we can do for an agent, and it is small.
3. **The layered policy.** Unbreaks the four designs we broke, with zero public
   surface.
4. **`motion:` on motion.dev.** One directive shape; grow the catalogue freely.
5. **The greenfield eval at `compose`** — the test we never ran, and the only one
   that tells us whether the thesis is true.

Not before launch: don't open `<style>`, don't chase contracts-sheet-in-sx, and
don't invent mechanisms. (`channels` was CSS custom properties wearing a costume.
Withdrawn.)
