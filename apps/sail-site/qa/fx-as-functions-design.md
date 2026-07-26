# Design note: fx as functions on Style properties

**Date:** 2026-07-26  
**Branch:** `job/style-synthesis`  
**Status:** exploration only — no compiler work implied  
**Audience:** human pair for product / surface design

---

## 1. Problem — three mental models for one paint channel

Authors already animate the same properties three different ways:

| Lane | Author surface | What it means |
|------|----------------|---------------|
| **Style paint** | `sx={style.opacity(100).scale(100)}` | Static (or condition-keyed) values on the host. Closed vocabulary, theming integrity, autocomplete on setters. |
| **Interaction / lifecycle fx** | `fx:press={{ scale: 'press' }}`, `fx:enter={{ opacity: 'fade' }}` | Timed values for the *same* properties, driven by a trigger (hover, press, focus, enter, exit, reveal, scroll, loop). Preset catalog + optional inline timing. |
| **Timeline scrub** | `timeline(progress, { x: [[0, …], [1, …]] })` + bind/effect into CPs or style | Progress-driven choreography. Author owns the scrubber (signal, scroll, media). Not an element directive. |

The dual model that hurts most is **Style vs `fx:`**. Animation targets are paint/layout slots (`opacity`, `scale`, `translate*`, `rotate`) — not a parallel universe. Yet authors must:

1. Learn Style setters for rest state.
2. Re-express the *same* property under `fx:<trigger>={{ prop: 'preset' }}` with a different shape (map of channel → preset/inline, not a chain of setters).
3. Graduate to `timeline` + bind when they need scrub/orchestration — a third vocabulary again.

`fx:` is the right *runtime* attachment (one `__sui_motion_attach` config, reduced-motion policy, lifecycle hold for exit). The open question is whether the *author* should keep thinking in a separate directive grammar, or whether fx should read as **functions that fill Style slots with timed values**.

---

## 2. Proposal sketches

### A — fx returns timed values into Style slots

Treat motion as a *value kind* that Style setters can accept:

```sail
<!-- rest + hover scale from one chain -->
<button sx={style
  .scale(100, fx.hover('press'))
  .opacity(100, fx.enter('fade'))
}>
  Save
</button>
```

Sketch of the idea (not API-final):

- `style.scale(rest, …timed?)` — first arg is rest/base (token or number), later args (or a single options object) attach trigger-bound curves for that channel.
- `fx.hover('press')`, `fx.enter('fade')`, `fx.press({ from: 1, to: 0.96, … })` — factories that produce a **timed value** (trigger + preset or inline channel config), not CSS classes.
- Multiple triggers on one channel: `style.scale(100, fx.hover('press'), fx.press('press'))` or an explicit map if that overloads poorly.

**Why this is attractive:** one mental model — “I am authoring scale/opacity.” Autocomplete stays on Style. The compiler still lowers to the same motion IR the runtime already understands.

**Risks:** Style chains grow dual-purpose (static paint + attach side effects). Condition helpers (`when({ hover: … })`) already mean *CSS state*, not *WAAPI on hover* — naming and docs must keep those distinct.

### B — keep `fx:press={{…}}` as sugar over the same IR

Do not delete the directive. Make it pure sugar:

```text
fx:press={{ scale: 'press' }}
  ≡  (conceptually) style.scale(…rest…, fx.press('press'))
  ≡  MotionAttachConfig { trigger: 'press', motions: [{ property: 'scale', preset: 'press', … }] }
```

Rules of engagement:

- **One IR.** Directive form and function form lower to the same attach config the runtime already takes.
- **Sugar stays default for multi-channel bursts.** `fx:enter={{ opacity: 'fade', translate-y: 'rise' }}` is still the shortest way to attach several channels to one trigger.
- **Function form wins for co-location** with rest paint: “this button’s scale is 100 at rest and press-preset on press” lives in one `sx` expression when that is what the author is thinking about.

Public rename already landed (`motion:` → `fx:`). B keeps that surface stable while A experiments beside it.

### C — timeline + bind remains the advanced scrub lane

`timeline(progress, tracks)` is not an interaction preset and should not be forced into Style setters.

- **Progress-driven** (scroll, media, gesture scrub, demo player) stays `timeline` → signals → bind / CSS variables.
- **Event-driven** (press, hover, enter) stays fx (directive or Style-slot functions).
- **No false unification:** putting scrub curves inside `style.scale(fx.timeline(…))` would blur the “attach on trigger” model with “drive every frame from a number.” Keep the scrub lane advanced and explicit.

Reveal/scroll *triggers* on `fx:` (platform-driven progress, zero JS listeners) can stay on the fx IR; free-form multi-track choreography stays on `timeline`.

---

## 3. What stays out — ambient loops vs interaction fx

Infinite **platform** decoration (CSS `animate-spin`, marquee keyframes, skeleton shimmer owned by a keyframe catalog) is **not** interaction fx.

| Kind | Home | Why |
|------|------|-----|
| Spinner / continuous CSS keyframe utility | Keyframe / PE catalog (or closed ambient CSS), not Style-slot fx | No rest↔active story; pure decoration; often prefers-reduced-motion = off |
| `fx:loop` with a motion preset (`spin`, bounce, …) | Ambient branch of the **same** fx IR (iterations: infinity) | Still WAAPI-owned, catalog-preset, reduced-motion policy in the motion runtime |
| `fx:press` / `fx:enter` / Style-slot `fx.press` | Interaction / lifecycle fx | Finite rest↔active or enter/exit |

Design rule: **do not teach “put spin on `style.rotate(fx.loop('spin'))`” as the primary spinner story** if the product already has (or wants) a cheap CSS keyframe for indeterminate progress. Ambient `fx:loop` remains for *authored* continuous motion that should share the motion runtime and preset catalog — not as a replacement for every CSS loop utility.

---

## 4. Autocomplete and SSOT implications

Today roughly:

| Concern | SSOT |
|---------|------|
| Style setters (`opacity`, `scale`, …) | Style setter table (`SetterSpec` / lint style setters) |
| Motion properties allow-list | Motion property vocabulary (lint + compiler; subset of paint channels) |
| Triggers (`press`, `enter`, …) | Single trigger list (lint, parser, skill docs) |
| Presets (`press`, `fade`, …) | `animation-presets.json` (compiler + lint include) |
| Timeline track API | Runtime `timeline()` |

If fx becomes callable into Style slots:

1. **Style setters that accept timed values** must be the *same* channels the motion allow-list already accepts — generate one from the other, or share a single “animatable Style channel” table. Drift = autocomplete offers `style.blur(fx.hover(…))` that the compiler rejects.
2. **`fx.<trigger>(…)` factories** should autocomplete triggers from the existing trigger list and presets from `animation-presets.json` — not a hand-copied second list.
3. **Directive sugar and function form** must resolve presets and inline timing through one path (today’s fx lowerer), so diagnostics stay identical.
4. **LSP / skill docs** teach one story: “timed values for Style slots; `fx:` is the multi-channel sugar.” Avoid a third tutorial track.
5. **`when({ hover })` vs `fx.hover`** — document and, if needed, lint: CSS pseudo state vs WAAPI attach. Same word “hover,” different engines.

No new preset file. No parallel trigger enum. Growth surface is: which Style setters accept a timed-value argument (likely the existing motion property set only).

---

## 5. Migration path without thrash

Goal: authors never rewrite working `fx:` markup for fashion.

1. **Phase 0 (now):** `fx:press` / `fx:enter` / … are the public surface. Document that they lower to a stable attach IR.
2. **Phase 1 (optional types / runtime only):** introduce `fx.hover` / `fx.press` / … as **value factories** usable only where the compiler already understands motion configs (or as pure documentation of the IR). No Style integration yet.
3. **Phase 2:** allow timed values on a **small** Style channel set (`scale`, `opacity`, maybe `translateY` / `rotate`) inside `sx={…}`. Lower to the same IR as `fx:`. Emit identical runtime attach calls.
4. **Phase 3:** document dual authoring; prefer Style-slot form in new examples where rest + motion co-locate; keep directive form for multi-channel and for exit/enter-only attachments with no rest paint.
5. **Never required:** mass-migrate call sites. Codemod optional and mechanical (directive ↔ function) only after IR parity is proven in tests.
6. **Deprecation:** none until Style-slot form is preferred in stdlib *and* tutorials for a full cycle. Sugar is not a migration tax.

Compatibility check for every phase: existing `fx:press={{ scale: 'press' }}` snippets (site, stdlib, conformance) compile and behave byte-identical at the attach config layer.

---

## 6. Open questions for human pair

- **Arity and chaining:** is `style.scale(100, fx.hover('press'))` clearer than `style.scale({ rest: 100, hover: fx.press('press') })` or a separate `.fx(…)` chain method? Rest + N triggers needs a readable shape.
- **Rest ownership:** when both `sx={style.scale(100)}` and `fx:press={{ scale: 'press' }}` exist today, rest is Style and active is fx. If timed values live *inside* Style, who wins when both directive and Style-slot form attach the same trigger+channel?
- **`when({ hover })` collision:** do we rename, lint, or accept dual “hover” meanings forever? CSS hover paint vs press/hover motion are both legitimate.
- **Exit hold:** lifecycle `exit` needs the framework to keep the node mounted. Does that attach only via `fx:exit=…` (element-level), or can a Style-slot timed value participate in exit without a directive?
- **Ambient `loop`:** in or out of Style-slot functions? Allowing `style.rotate(fx.loop('spin'))` is consistent but may fight the “CSS keyframe for spinners” product line.
- **Inline physics in Style:** how much of `duration` / `easing` / `from` / `to` belongs in the Style expression before it becomes unreadable compared to `fx:enter={{ opacity: { from: 0, to: 1, duration: 200 } }}`?

---

## 7. Recommended first vertical slice

**Thin slice:** one channel, one trigger, sugar parity — no timeline, no exit, no ambient loop.

| Item | Choice |
|------|--------|
| Channel | `scale` (already dominant on `fx:press`) |
| Trigger | `press` |
| Author forms | (1) existing `fx:press={{ scale: 'press' }}` (2) experimental `sx={style.scale(100, fx.press('press'))}` or the arity we settle in open Q1 |
| Lowering | Both → same `MotionAttachConfig` / attach call |
| Proof | Compiler fixture: both forms emit identical attach config; one Playwright/motion test already covering press scale still passes for both |
| Docs | One short example next to existing press snippet; no stdlib rewrite |

Stop after IR parity on that pair. Only then consider `opacity` + `enter`, then multi-channel sugar equivalence tests. Do **not** start with full Style×trigger matrix or LSP completeness — prove the dual surface is one IR first.

---

## Summary recommendation

Treat **fx as timed values for Style channels**, keep **`fx:` directives as multi-channel sugar**, leave **`timeline`+bind as the scrub lane**, and keep **CSS/platform infinite decoration out of interaction fx**. Ship nothing until a single vertical slice (`scale` × `press`) proves both author forms lower to one attach IR; migrate nothing by force.
