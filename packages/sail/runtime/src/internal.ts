// THE COMPILER'S ABI. Not the language.
//
// Everything here is something the EMITTER writes into generated code. No human
// types `bindFor` or `runAction`, and no agent should ever see them offered as
// API — but they sat in the same public export as `signal` and `effect`, so the
// runtime's front door advertised ~80 symbols when the language is about ten.
// That is the single largest thing standing between us and a teachable surface:
// an agent reading that index cannot tell the language from the plumbing.
//
// So the rule is now one line:
//
//     the COMPILER imports from `@ship/sail-runtime/internal`
//     HUMANS import from `@ship/sail-runtime`
//
// Splitting the entry is step one and it is purely additive — nothing breaks,
// because the root still re-exports these for now. Step two points the emitter
// here (a hardcoded '@ship/sail-runtime' literal in emit/js.rs, emit/dts.rs and
// emit/synthetic/render.rs). Step three strips the ABI from the root, which is
// the only breaking part. Once the emitter no longer writes the removed symbols
// the root can drop them too.

// A TOTAL re-export, deliberately. A curated list would break silently the first
// time the emitter reaches for a symbol nobody remembered to add here — and the
// emitter reaches for ~37 of them (control flow, element wiring, mount plumbing,
// list patching, the animation ABI, plus `component`/`controller` and every
// auto-imported action and transition). The pruning happens on the ROOT, where it
// is visible and testable, not here.
export * from './index'
