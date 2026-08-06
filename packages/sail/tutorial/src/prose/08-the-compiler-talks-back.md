---
title: "The compiler talks back"
description: "The agent loop: run the build, read the rejection, fix exactly what it names, run again."
order: 8
slug: "08-the-compiler-talks-back"
prev: "07-themes-are-one-attribute"
next: "09-review-like-an-owner"
chapter: "agents"
---

This lesson's starter **does not compile — on purpose.** Four defects
are planted in it, and your job is not to find them by eye. It is to
practice the loop that makes this language work for agents and humans
alike:

1. run the compiler,
2. read the rejection,
3. fix exactly what it names,
4. run again.

SAIL's errors are written to be *acted on*: each one names the rule, the
offending value, and the complete legal vocabulary — the rejection is
the documentation. An author (human or agent) who works this loop never
needs to memorize the token tables, and never needs to guess.

```
sail-compiler build --mode component tut-08-release-notes-starter.sail
```

## The loop, played out

Run 1. The compiler reports one costed rejection at a time — first, a
Style paint value that is not on the closed scale:

```
Error: Style error in `sx=`: style `pad(12px)` — value is not a token on this setter's scale
```

The message *is* the spacing scale rule. `12px` is not a token —
comfortable padding on this ramp is `pad(4)`. Style failures are now
hard compile errors (not silently empty paint), so they surface before
later expression and motion checks.

Run 2. After the Style fix, a call shape the render path forbids:

```
Error: tut-08-release-notes-starter.sail: line 62 col 1: [expr `formatDate(publishedAt())`]
call `formatDate(…)` with arguments is not allowed in renderable expressions;
only zero-arg signal reads are permitted
```

Go by the expression in brackets, not the line number: this diagnostic
currently points at the end of the file rather than at the offending
line. The bracketed source is exact, and it is the part you search for.

The fix is lesson 04's move: do the work in the controller, expose a
zero-arg read. `formatDate(publishedAt())` becomes a `published`
derived, and the template reads `{published()}`.

Run 3. The motion system holds its vocabulary the same way:

```
Error: tut-08-release-notes-starter.sail: line 48 col 1: [sail-animate-token-only]
motion preset `whoosh` is not in the closed vocabulary. Known presets:
spring:snappy, spring:gentle, spring:bouncy, spring:deliberate, press, release,
pulse, fade, fade-out, bounce, marquee, marquee-reverse, spin
  suggest: Use a registered preset, write the values inline (`{ from: 0, to: 1 }`),
  or extend animation-presets.json.
```

There is no `whoosh`. There is `fade`, the full legal set is right there
in the error — and, as lesson 06 showed, the inline form if none of
them is what you meant.

Run 4. Now the `<style>` block, which answers twice. First for what is
*in* it:

```
Error: tut-08-release-notes-starter.sail: line 64 col 19: [sail-css-literal]
raw color `#ff0000` in `color` — use a design-system token (`var(--…)`)
```

Fix the colour and it objects to the block itself:

```
Error: tut-08-release-notes-starter.sail: line 64 col 1: [sail-no-style-block]
raw <style> is not the floor styling surface — the floor is one way to do CSS:
Style as a block statement (`style({ box: pad(4), paint: bg(surface-2) })`).
Motion/variants/responsive are becoming language elements, not raw CSS.
This is the opinionated floor; primitives use it too.
  suggest: Text elements (<h1>-<h6>, <p>) are styled from the type scale
  automatically — write them plain. Everything else: Style block attach
  (`style({ paint: bg(…) })` / when / match), <column>/<row>/<grid> for structure. A gap
  here is a missing language element to add, not a reason to hand-author CSS.
```

That order is the point. Tokenising the colour was a real improvement
and still not enough — the block was never a legal surface. Delete it.
Whatever it was painting, the answer is Style as a block attach or a
component; and if the vocabulary genuinely cannot express the design,
that is a **language gap to report**, never a reason to smuggle CSS.
Product form:
```
<div>
  style({ box: pad(4), paint: bg(surface-2) })
</div>
```

Run 5: `compiled 1 file(s)`.

## Why this is the agent lesson

An agent authoring SAIL does exactly what you just did — nothing more.
It cannot ship an off-system value, an invented preset, or a stylesheet,
because the build stops it; and the error text is precise enough to fix
mechanically. The compiler is the review that never gets tired. What it
*cannot* review — intent — is the next lesson.

## Try it

1. Work the loop until the starter compiles. Resist fixing ahead of the
   errors — the order the compiler chooses is part of the exercise.
2. Compare your result with the solved version. If you picked `pad(3)` or
   a spring instead of `fade`, fine: you stayed inside the vocabulary,
   which is the only thing the compiler was defending.
3. Plant a defect of your own — an unknown tone, a `style=` attribute,
   a keyless `<for>` — and read what comes back.
