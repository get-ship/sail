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

SUI's errors are written to be *acted on*: each one names the rule, the
offending value, and the complete legal vocabulary — the rejection is
the documentation. An author (human or agent) who works this loop never
needs to memorize the token tables, and never needs to guess.

```
sail-compiler build --mode component tut-08-release-notes-starter.sail
```

## The loop, played out

Run 1. The compiler reports one costed rejection at a time — first, a
call shape the render path forbids:

```
Error: tut-08-release-notes-starter.sail: line 33 col 1: [expr `formatDate(publishedAt())`]
call `formatDate(…)` with arguments is not allowed in renderable expressions;
only zero-arg signal reads are permitted
```

The fix is lesson 04's move: do the work in the controller, expose a
zero-arg read. `formatDate(publishedAt())` becomes a `published`
derived, and the template reads `{published()}`.

Run 2. Now it reaches the sx lint:

```
Error: tut-08-release-notes-starter.sail: line 12 col 21: [sail-sx] sx `p:12px` —
`12px` is not a token (raw values are not allowed; spend a token). allowed: 0,
0_75, 1, 1_25, 1_5, 2, 2_5, 3, 3_5, 4, 4_5, 5, 6, 7, 8, 9, half, size-px; did
you mean `p:1`?
  suggest: sx is surface-only: spacing 0–8 (p/m…), colors (bg/fg), radius, text.
  Layout is the primitives' job (<row>/<column>/<grid>). Raw px/hex are not expressible.
```

The message *is* the spacing scale. `p:12px` was aiming at comfortable
padding; `p:4` is the token that means it.

Run 3. The motion system holds its vocabulary the same way:

```
Error: tut-08-release-notes-starter.sail: line 15 col 1: [sail-animate-token-only]
motion preset `whoosh` is not in the closed vocabulary. Known presets:
spring:snappy, spring:gentle, spring:bouncy, spring:deliberate, press, release,
pulse, fade, fade-out
  suggest: Use a registered preset (spring:snappy, fade, …) or extend animation-presets.json.
```

There is no `whoosh`. There is `fade`, and the full legal set is right
there in the error.

Run 4. Last, the floor itself:

```
Error: tut-08-release-notes-starter.sail: line 30 col 1: [sail-no-style-block]
raw <style> is not the floor styling surface — the floor is one way to do CSS:
sx token utilities (sx="p:4 bg:surface-2"). Motion/variants/responsive are
becoming language elements, not raw CSS. This is the opinionated floor;
primitives use it too.
  suggest: Text elements (<h1>-<h6>, <p>) are styled from the type scale
  automatically — write them plain. Everything else: sx= for surface tokens…
```

Delete the block. Whatever it was painting, the answer is sx or a
component — and if the vocabulary genuinely cannot express the design,
that is a **language gap to report**, never a reason to smuggle CSS.

Run 5: `compiled 1 file(s)`.

## Why this is the agent lesson

An agent authoring SUI does exactly what you just did — nothing more.
It cannot ship an off-system value, an invented preset, or a stylesheet,
because the build stops it; and the error text is precise enough to fix
mechanically. The compiler is the review that never gets tired. What it
*cannot* review — intent — is the next lesson.

## Try it

1. Work the loop until the starter compiles. Resist fixing ahead of the
   errors — the order the compiler chooses is part of the exercise.
2. Compare your result with the solved version. If you picked `p:3` or
   a spring instead of `fade`, fine: you stayed inside the vocabulary,
   which is the only thing the compiler was defending.
3. Plant a defect of your own — an unknown tone, a `style=` attribute,
   a keyless `{#for}` — and read what comes back.
