---
title: "Review like an owner"
description: "The capstone: five flaws that compile — and the checklist that catches what the compiler cannot."
order: 9
slug: "09-review-like-an-owner"
prev: "08-the-compiler-talks-back"
next: null
chapter: "agents"
---

The north star of this language is a loop: **agents author, humans
review**. Lesson 08 showed the half the compiler owns. This lesson is
the other half — the flaws that *compile*, run, and demo beautifully,
and are still wrong.

The starter is a complete support-inbox screen. It builds cleanly. It
uses live(), keyed lists, motion, tones — everything from lessons 1–8.
It also contains **five planted review flaws**. Read it like an agent's
pull request before you read the list below.

## The reviewer's checklist

Each of these is a question the compiler cannot ask, because the answer
is intent:

- **Does data flow, or is it frozen?** The component declares a
  `title` prop — and then hardcodes `Support inbox` in the markup. The
  demo looks identical; every future consumer (a rename, a translation
  pass, a reuse) is broken. *Check: does every declared prop actually
  reach the markup?*
- **Do all three states render?** The template handles `error` and
  `ok` — there is no `{#while-loading}`. On a fast dev machine, nobody
  notices; on a real network, first paint is a blank hole. *Check: for
  every live() handle, find all three states in the template.*
- **Is the key an identity?** `key={m.topic}` compiles — but topics
  repeat (`latency` twice, `billing` twice). Rows will collide and
  recycle each other's DOM on any reorder or archive. *Check: could two
  live items ever share this key? Then it is not a key.*
- **Does removal have an exit?** Rows arrive with a staggered
  `motion:enter` — and vanish with a snap, because `motion:exit` is
  missing. Arrival was designed; departure was forgotten. *Check: every
  list that animates in should say what leaving looks like.*
- **Is that div a component the system already has?** The empty case
  is a hand-rolled `<div sx="…">Nothing matches…</div>`. It works; it
  also skips the semantics, sizing, and tone contract of
  `<empty-state>` — the component that exists precisely so empty never
  gets improvised. *Check: before approving a styled div, ask what
  component it is imitating.*

## Why these five

They share a signature: each one optimizes for the *first render on the
happy path* — which is exactly what a demo (or an agent maximizing "it
works") sees. Ownership means reading for the second render, the slow
network, the reorder, the retranslation. The compiler guarantees the
vocabulary; the reviewer guarantees the intent.

## Try it

1. Fix all five in the starter, then diff against the solved version.
2. Break the solved version's key back to `m.topic`, archive the first
   `latency` row, and watch what happens to the other one. That is the
   bug the checklist item prevents.
3. Write your own sixth flaw — something that compiles and is wrong —
   and hand it to a colleague (or an agent) with this checklist. If the
   checklist catches it, it belongs in your team's review doc.
