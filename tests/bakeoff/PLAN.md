# Track D — compile-away emitter prototype

Goal: prove the compiler can tree-shake the runtime per app. Svelte-shaped:
static template parts -> __tpl clones; only features the file uses get emitted
as imports; the rest never ships.

Approach against pinned head (1991ce2b4):
1. Instrument emit/synthetic + template_emit to record feature usage per
   compiled unit (which bind* / on / morph / query paths are referenced).
2. Emit an import manifest instead of blanket `@ship/sail-runtime` import.
3. Bundle with rolldown treeshake; measure % of runtime shipped for:
   a. store-counter (minimal: signal, bind, on, custom-element)
   b. sail-site landing (full: motion, query, virtual)
4. Gate: dead-feature elimination proven by absence in bundle (grep gate),
   byte deltas recorded per fixture.

This track is compiler-side and language-neutral — A/B/C all benefit.

## FINDING (from pinned head, 1991ce2b4)
The synthetic emitter ALREADY emits per-feature named imports:
  - abi_imports   -> @ship/sail-runtime/internal
  - action_imports -> @ship/sail-runtime/actions
  - motion_imports -> @ship/sail-runtime/motion
(render.rs:182-205). So the "compile-away" architecture is ~70% in place:
rolldown tree-shaking over these named imports already eliminates unused
runtime features per app. Track D narrows to:
1. MEASURE current elimination rate (bundle sail-site dist vs full runtime)
2. Close gaps: check bind.ts/on.ts/morph.ts are side-effect-free modules
   (tree-shakeable) — package.json sideEffects field lists only
   boundary-elements.js, which is promising
3. The __tpl hoists + hydrate walkers are per-file already.

Revised scope: measurement + gap-closing, NOT new emit machinery.
