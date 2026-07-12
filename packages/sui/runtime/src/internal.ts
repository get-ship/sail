// THE COMPILER'S ABI. Not the language.
//
// Everything here is something the EMITTER writes into generated code. No human
// types `bindFor` or `attachRef`, and no agent should ever see them offered as
// API — but they sat in the same public export as `signal` and `effect`, so the
// runtime's front door advertised ~80 symbols when the language is about ten.
// That is the single largest thing standing between us and a teachable surface:
// an agent reading that index cannot tell the language from the plumbing.
//
// So the rule is now one line:
//
//     the COMPILER imports from `@ship/sui-runtime/internal`
//     HUMANS import from `@ship/sui-runtime`
//
// Splitting the entry is step one and it is purely additive — nothing breaks,
// because the root still re-exports these for now. Step two points the emitter
// here (a hardcoded '@ship/sui-runtime' literal in emit/js.rs, emit/dts.rs and
// emit/synthetic/render.rs). Step three strips the ABI from the root, which is
// the only breaking part and is coupled to the legacy migration — the 66 files
// still importing `ref` are all pre-cutover and don't compile anyway.

// ── template control flow (each / if / await / html) ─────────────────────────
export { bindFor, bindForSignal, bindIf, bindAwait, bindRawHtml } from './index'
export { bindWhileError, bindWhileLoading } from './index'

// ── element wiring the emitter generates ─────────────────────────────────────
// `attachRef` is what `bind:this` COMPILES TO. `ref` is the same concept exposed
// a second time as an author API — three names for one idea (bind:this / ref /
// attachRef). `ref` leaves the root in step three; `bind:this` is the language.
export { attachRef, ref } from './index'
export { runAction, on, matchesKey } from './index'
export { bindOpen, bindPopover, bindSlotEmpty, slotFilled } from './index'
export { wireFormMutation } from './index'

// ── mount / lifecycle plumbing ───────────────────────────────────────────────
export { createMountState, componentMountState, pickSignals } from './index'
export { SIGNALS_KEY, LIFECYCLE_KEY } from './index'
export { resolveController, resolveSlots } from './index'
export { setHostContext, hostContext } from './index'
export { adoptSuiSheets } from './index'

// ── list patching / morph ────────────────────────────────────────────────────
export { applyPatch, applyPatches, snapshotChildren, morph } from './index'

// ── animation ABI ────────────────────────────────────────────────────────────
// NOTE: `runEnter`/`runLeave`/`cancelTransitions` belong to the LEGACY animation
// system (transition:/in:/out: + registerTransition). They die when that folds
// into `motion:` as motion:enter / motion:exit — one system, one catalog.
export { runEnter, runLeave, cancelTransitions } from './index'
export { __sui_motion_attach } from './index'

// ── misc emitter helpers ─────────────────────────────────────────────────────
export { camelOf, tokenizePath } from './index'
