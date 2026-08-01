---
name: sail-motion-review
description: Reviews sail motion usage against a high craft bar. Default to flagging; approval is earned. Use when asked to review animation/motion in sail sources.
---

# Reviewing Sail Motion

One job: review motion in `.sail` sources against a craft bar. Not
features, not unrelated bugs, not general code review.

Sail motion is preset-driven: `enter`/`exit`/`loop`/`scroll`/`press`
on Style, plus the closed keyframe catalog. Reviewing sail motion means
judging **preset choice, parameters, and placement** — and catching
raw-CSS or controller-driven motion that escaped the vocabulary (an
immediate finding: motion outside presets bypasses every guarantee
below).

## The ten standards

1. **Justified.** Every motion answers "why does this animate" —
   spatial consistency, state indication, feedback, explanation.
   "Looks cool" on a frequently-seen element is a block.
2. **Frequency-appropriate.** Keyboard-initiated or 100×/day actions:
   no animation. Tens/day: reduced. Occasional: standard. Rare: delight.
3. **Responsive easing.** Entrances/exits ease out. Motion that delays
   the moment the user watches most is a block.
4. **Sub-300ms UI.** Anything slower on a UI element needs written
   justification. Ambient loops are exempt; interactions are not.
5. **Origin & physicality.** Popovers/dropdowns/tooltips scale from
   their trigger, not center. Nothing appears from nothing — enter
   presets must pair a small transform with opacity, not scale-from-zero.
6. **Interruptible.** Rapidly retriggered motion (toasts, toggles,
   hovers) must retarget from current state, never restart from zero.
7. **Composited properties only.** Transform and opacity. Motion driving
   layout properties (width/height/inset/padding) is a performance
   finding — in sail this almost always means raw escapes, since presets
   are composited by construction.
8. **Reduced motion honored.** Gentler, not zero: keep opacity/color
   transitions, drop movement. Hover motion gated to precise pointers.
9. **Asymmetric enter/exit.** Deliberate actions (press, hold,
   destructive confirm) animate slower; system responses snap. Symmetric
   timing on press-and-release is a finding.
10. **Cohesive.** Motion matches the component's personality and the
    product. When unsure whether a motion earns its place — delete it.

## Escalation triggers (flag on sight)

- Any motion written as raw CSS / keyframes outside the preset system.
- A `loop` preset on content the user reads or interacts with
  constantly (ambient belongs on decoration only).
- `scroll` motion that gates content visibility — content must exist
  in the static end state (REVEAL contract); scroll enhances, never
  reveals-from-empty.
- Entrance on everything at once where a stagger belongs.
- Press feedback missing on primary buttons (`press` preset exists —
  absence on interactive elements is a finding, not a nit).
- Controller-driven style mutation per frame (signal-updated sx in an
  interval/rAF) where a preset exists.

## Remedial hierarchy (prefer earlier)

1. Delete it. 2. Reduce it (duration, distance, properties).
3. Fix the easing/preset choice. 4. Fix origin/physicality.
5. Make it interruptible. 6. Move it into the preset system.
7. Asymmetric timing. 8. Polish (stagger, blur bridges).
9. Accessibility & cohesion.

## Output format

**Part 1 — findings table** (required): | Source location | Current | Recommended | Why | — one row per issue, `file:line` cited.

**Part 2 — verdict** (required): findings grouped by tier — feel-breaking
regressions / missed simplifications / performance / interruptibility &
timing / origin & cohesion / accessibility — then an explicit decision:

- **Block**: any feel-breaking regression, motion on a keyboard or
  high-frequency action, motion outside the preset system, content
  gated behind scroll motion.
- **Approve**: presets throughout, durations in bounds, interruptible
  where needed, reduced motion respected, nothing obviously deletable.
