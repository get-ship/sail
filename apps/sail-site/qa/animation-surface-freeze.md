# Animation surface freeze

**Date:** 2026-07-26  
**Branch:** `job/style-synthesis`  
**Status:** product freeze — implement next; no dual dialects  
**Audience:** engineers cutting over author surface + runtime IR

This supersedes exploration in `fx-as-functions-design.md` as the taught product. That note’s dual author forms (`fx:` sugar + Style-slot timed values) are **not** the freeze. One author model. One attach IR under the hood.

---

## 1. Author model — two concerns only

### Look

Style chain paints rest and state patches:

```
style.…rest…
  .when({ hover, active, focus, focusWithin, disabled, … })
```

- `when` = browser/CSS state → style patch (pseudo / data-driven).
- Pair with Style `transition` / `duration` / `ease` for **micro ease** on those patches.
- Not WAAPI. Not lifecycle. Not progress scrub.

### Time — who owns the clock

| Clock owner | Author surface | Engine |
|-------------|----------------|--------|
| Browser state flip | `when({…})` + CSS transition on the patch | CSS transitions |
| Framework node life | **`.enter` / `.exit` on Style** | Motion attach IR → `__sui_motion_attach` |
| Author progress | `timeline` + drivers (`clock`, `scrollProgress`, `springValue`) + `bind` into Style / CPs | Timeline lane |

Three clocks. Three surfaces. Do not invent a fourth grammar.

---

## 2. Kill / demote

| Item | Decision |
|------|----------|
| Permanent **`fx:` as taught product** | **Migrate off.** Directive grammar is not the long-term author story. Runtime attach may stay private (see §3). |
| Enter inside `when()` | **Forbidden.** Lifecycle ≠ pseudo-state. |
| Free-form author `@keyframes` | **Forbidden.** Closed PE catalog only (platform). |
| `style.animate(spin)` | **Awkward, keep for now.** PE catalog: `spin`, `pulse`, `shimmer`, `progress-indeterminate`. Only **spin** is Style-facing today (`STYLE_ANIMATE_NAMES`). Follow-up (not blockers): rename candidates e.g. `loop(spin)` or hide PE behind spinner element only. |
| Mega-`.animate` for enter/exit | **No.** Enter/exit are `.enter` / `.exit`, not a named lifecycle brand and not a wasteful catch-all animate API. |
| Dual dialects (directive + Style-slot `fx.press` factories as taught pair) | **Do not ship as product.** One author path. |

---

## 3. Keep runtime (implementation floor)

Do not rip these while rewriting grammar:

- **exit-hold** — node stays mounted until exit motion finishes
- **`__sui_motion_attach`** (or private rename) — single attach seam; compiler emits `MotionAttachConfig`
- **reduced-motion** — runtime policy on attach path
- **stagger** — already on `MotionAttachConfig.stagger`; must remain expressible for list enter
- **action marquee / icon attach** — private attach consumers of the same IR / PE catalog
- **limited PE catalog** in sail-base / ambient CSS for product chrome (spinner, shimmer, progress, pulse)
- **timeline lane** for multi-step / scrub choreography (`timeline` + drivers + bind)

IR shape today (stable enough for cutover):

```ts
MotionAttachConfig {
  trigger: string   // enter | exit | loop | … internal
  motions: MotionConfig[]
  stagger?: number  // seconds
}
```

`.enter` / `.exit` **lower to the same IR** as today’s `fx:enter` / `fx:exit`. No second motion stack.

---

## 4. Frozen author shape: `.enter` / `.exit`

```
style.…rest…
  .enter(fromStyle, motion?)
  .exit(toStyle, motion?)
  .when({ … })
```

Rules:

- **`from` / `to`** are **Style snapshots** (same vocabulary as rest paint: opacity, scale, translate*, etc.).
- **`motion`** = preset name from `animation-presets.json` (e.g. `spring:snappy`, `fade`) **or** options object aligned with that catalog’s resolved fields (duration, easing, spring params — not free-form keyframes).
- Chain order is documentation preference, not a hard parser law: rest → enter/exit → when is the teachable shape.
- **Do not** put enter/exit keys inside `when({…})`.
- **Do not** reintroduce named lifecycle triggers as a parallel public directive set once Style methods ship.

### Stagger (list enter)

Must be expressible without a second product:

1. **Preferred freeze option:** stagger as a field on enter **options**  
   ` .enter(fromStyle, { preset: 'fade', stagger: 0.15 })`  
   → `MotionAttachConfig.stagger` on the enter attach.
2. **Parent helper** (e.g. list/for wrapper that stamps delay index) is allowed as sugar if options form is insufficient; same IR.

Either way, runtime already understands `stagger` on attach config (see thinking-dots / motion tests). Author API must not lose that.

### Lowering

```
.enter(from, motion)  →  MotionAttachConfig { trigger: 'enter', motions: […], stagger? }
.exit(to, motion)     →  MotionAttachConfig { trigger: 'exit',  motions: […], stagger? }
```

Parity with today’s `fx:enter` / `fx:exit` attach configs is the acceptance bar — byte-identical attach where possible.

---

## 5. Interaction press / hover / focus after freeze

| Today | After freeze |
|-------|----------------|
| `fx:press={{ scale: 'press' }}` | Prefer **Look**: rest + `when({ active / hover: style.scale(…) })` + CSS transition. Use motion attach only if spring physics / multi-channel burst cannot be said with when+transition. |
| `fx:hover` / `fx:focus` | Same: demote to `when` + transition for micro UI. |
| `fx:enter` / `fx:exit` | **`.enter` / `.exit` on Style.** |
| `fx:loop` | Split: product chrome loops → PE catalog / element skin; authored continuous WAAPI → private attach or timeline, not taught `fx:` grammar. |
| `fx:reveal` / `fx:scroll` | **Drivers + timeline + bind** (author owns progress). |

No permanent public `fx:` chapter once cutover completes.

---

## 6. Serious `.animate`?

**Non-goal for v1 cutover.**

- Not the brand for enter/exit.
- Not a replacement mega-API for all time-based motion.
- Optional future sugar for `springValue` + bind only if justified later; document and gate separately.

Ambient PE remains the closed keyframe catalog (`keyframes-catalog.ts`), not author `.animate({…})` bags.

---

## 7. FLIP

Want eventually. **Out of freeze.** Separate feature; do not block enter/exit or fx retirement on FLIP.

---

## 8. Cutover — this monorepo (~74–84 `fx:` call sites)

Rough inventory (`.sail`, monorepo, ~2026-07-26): ~80+ directive sites across site, tutorial, stdlib, chat-elements, demos. Treat **~74 product sites** as the migration batch; docs/tutorial count in the same phases.

### Phase A — press / hover / focus → Look

- Rewrite `fx:press` / `fx:hover` / `fx:focus` to Style rest + `when({…})` + transition where visually equivalent.
- Keep attach only for cases that fail CSS transition (true spring, multi-channel WAAPI).
- Update button/control chrome and site press demos first (highest teaching weight).

### Phase B — enter / exit → Style methods

- Ship `.enter` / `.exit` compiler + types; lower to existing `MotionAttachConfig`.
- Migrate trays, lists, flow-trace, tutorial 06, showcase, departures, etc.
- Preserve exit-hold and stagger behavior; golden + motion tests pin attach parity.

### Phase C — loop split (PE vs attach)

- Spinner / ambient chrome: PE catalog (`animate(spin)` or spinner-only PE) — not `fx:loop`.
- Thinking-dots / authored continuous motion: keep runtime attach; re-home author API off public `fx:loop` (private attach or catalog-backed helper).
- Marquee action: stays attach consumer; no public `fx:` required in templates.

### Phase D — reveal / scroll → drivers

- `fx:reveal` / `fx:scroll` → `scrollProgress` / `clock` / other drivers + `timeline` + bind into Style.
- Site scroll/reveal demos are the reference cut.

### Phase E — retire `fx` grammar

- Lint: `fx:` deprecated → error after dual window.
- Docs/tutorial rewrite (lesson 06 title/story changes from “motion is a directive”).
- Codemod optional for external shape; monorepo can hand-migrate at this scale.
- Leave `__sui_motion_attach` private; rename only if public string surface is a problem.

Order A→B→C→D→E is preferred; B can land tooling while A lands content.

---

## 9. Readiness for apps outside this monorepo

Do **not** mass-convert external Sail apps until the product floor is frozen and shipped:

| Prerequisite | Why |
|--------------|-----|
| **Style** as the only paint surface | No parallel class= / raw CSS author story |
| **Layout attrs** settled (`grow`/`shrink`/box participation, etc.) | Motion targets layout-stable hosts |
| **No author `class=`** for product chrome | Avoid dual styling during motion cutover |
| **Tokens closed** | Enter/exit Style snapshots must resolve the same token tables |
| **`.enter` / `.exit` API shipped** (compiler + runtime parity) | External rewrite target must exist |
| **`fx:` deprecated** with **codemod or dual-window** | External repos need a mechanical path or a dated dual compile window |
| **PE catalog + reduced-motion policy documented** | Chrome loops and a11y must not regress |

Suggested external window: dual compile (`fx:` + Style enter/exit) → codemod → lint error. Monorepo may skip dual window if migrated in one branch train.

---

## 10. Explicit non-goals (freeze)

- Dual taught dialects (directive + Style-slot fx factories).
- Enter-in-`when`.
- Author free-form `@keyframes`.
- FLIP in this cut.
- Public mega-`.animate` lifecycle API.
- Forcing every ambient loop through WAAPI attach.

---

## 11. One-line product rule

**Look is Style + `when` (+ CSS transition). Life is `.enter` / `.exit`. Progress is `timeline` + drivers + bind. Runtime attach stays private. `fx:` leaves the teaching surface.**
