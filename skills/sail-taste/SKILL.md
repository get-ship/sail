---
name: sail-taste
description: Anti-slop design judgement for sail interfaces. Brief inference, design dials, and the mechanical pre-flight rules that keep agent output from reading as templated defaults. Use when authoring or reviewing any sail page or surface.
---

# Sail Taste

Every rule is contextual. Read the brief first; pull only what fits. The
compiler enforces vocabulary — this skill enforces judgement.

## 0. Design read (before any code)

Infer: page kind (landing / app shell / form / dashboard / editorial),
the vibe words used, audience, existing brand tokens, quiet constraints
(accessibility-first, regulated — these override aesthetics).

State one line before generating: "Reading this as: <page kind> for
<audience>, <vibe> language." If genuinely ambiguous, ask exactly one
question. Otherwise proceed.

## 1. The three dials

Set after the read; every layout/motion/density decision is gated by them.

- `VARIANCE` 1=symmetric … 10=artsy chaos
- `MOTION` 1=static … 10=cinematic
- `DENSITY` 1=art gallery … 10=cockpit

Baseline 8/6/4 for marketing; 5/3/3 for calm product; 3/2/5 for
trust-first; match existing +1 motion for preserve-redesigns.

## 2. Sail mechanics (the only stack)

- Layout: `column`/`row`/`stack`/`grid`/`center`. Never raw containers.
- Spacing/typography/color: sx setters with token values only —
  `style.fontSize(display)`, `style.tracking(tighter)`, `style.fg(muted-foreground)`.
  No raw px/hex.
- Motion: presets (`enter`/`exit`/`loop`/`scroll`/`press`). Never
  hand-rolled CSS animation.
- Components: L1 first; reach for the element before composing.
- The compiler's rejection is the table of contents. `sail_check` after
  every edit.

## 3. Anti-default discipline

Banned as defaults: AI-purple gradients, centered hero over dark mesh,
three equal feature cards, glassmorphism on everything, infinite-loop
micro-animation everywhere, one safe sans + one safe gray. These are
recitation, not choice. The dials and the design read pick the direction.

## 4. Mechanical pre-flight (failing any = shipping broken work)

Typography
- Display type: `fontSize(display)`/`3xl`, `tracking(tighter)`,
  `leading(none)` — but see italic-descender rule below.
- Body: `leading(relaxed)`, measure capped (`maxW` token), muted fg.
- Emphasis inside a headline: italic or bold of the SAME family. Never
  inject a second family for one word.
- Italic display type with descenders (y g j p q): `leading(none)` clips
  them — use a looser leading or add bottom padding on the wrapper.

Color
- One accent token per page, locked. Audit every element before shipping:
  a warm-gray page does not get a blue CTA in section seven.
- Neutrals from the theme's surface scale; one temperature per page.
- Saturation restraint: accent does the work, backgrounds stay quiet.

Layout
- Hero fits the initial viewport: headline ≤ 2 lines, subtext ≤ 20 words,
  CTA visible without scroll, top padding capped — if it floats halfway
  down, that's a bug photograph.
- Hero stack ≤ 4 text elements: eyebrow OR brand strip (not both),
  headline, subtext, CTA row. Taglines/logo walls/pricing teasers move
  to the section below.
- Eyebrow restraint (the #1 AI tell): max 1 eyebrow per 3 sections.
  Count them mechanically before shipping.
- Layout-family repetition ban: 8 sections need ≥ 4 distinct layout
  families. Zigzag image/text splits: max 2 consecutive.
- Anti-center bias: VARIANCE > 4 forbids the centered hero; split,
  left-align, or asymmetrize.
- Nav: single line at desktop, height capped, condense before wrapping.
- Bento/grids: exactly as many cells as content; ≥ 2–3 cells carry real
  visual variation (surface tint, media, pattern) — never six white
  cards of text.
- Mobile collapse explicit per section via the responsive vocabulary —
  never "it'll work out".

Components & states
- Cards only when elevation communicates hierarchy; else `divide`/
  borders/negative space. One radius scale per page, documented rule,
  followed everywhere.
- Shadows tint to the surface hue; no black drop shadows on light.
- Full state cycle always: loading (skeleton shaped like the final
  layout, not a spinner), empty (composed, says how to populate),
  error (inline for forms, toast only for transient), active/press
  feedback (`press` preset).
- Buttons: label fits one line at desktop; contrast ≥ AA against the
  button surface; ONE label per intent across the whole page (nav, hero,
  footer agree).
- Forms: label above input, helper text present, error below input;
  never placeholder-as-label; every text/placeholder/focus state passes
  AA against its section background.

## 5. Self-critique loop

Before declaring done: render it (`sail render` / dev server), look at
the screenshot, and remove one thing. Spend boldness in one signature
place; keep everything around it quiet and disciplined.
