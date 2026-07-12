// @ship/sui-std — SUI's COMPOSITIONS.
//
// A composition is a plain function that combines the reactive primitives and
// returns signals. That is the whole definition. There is no `Composition` type,
// no registration, no lifecycle contract, no rules — nothing to author *against*,
// which is exactly why anyone can write one.
//
// So this package is deliberately UNPRIVILEGED. It imports `signal` and `derived`
// from the same front door you do; the compiler does not know it exists; nothing
// here can do anything your own file cannot. If `selection()` looks like something
// you could have written in forty lines — good. Open it. You could have.
//
//   primitives  : compositions  ::  L1 elements : packs
//
// The one language type they touch is `Signal<T>`. That is the lingua franca: the
// language owns Signal, everything else composes over it. Cleanup is free — an
// `effect()` created while a component is setting up enrols on that component's
// mount scope and dies with it, so a composition never hands you a disposer to
// thread. (Owner scoping is what makes "just a function" true instead of aspirational.)
//
// WHAT LIVES HERE vs `use:` — one line: if it touches the DOM it is an action, not
// a composition. `useDialog`/`usePopover` used to live alongside these and broke
// that rule (DOM work in state's clothing); the platform then finished them off —
// `<dialog>` + invoker commands needs zero JS, and `<popover>` + `use:anchor` is
// the whole overlay family. They are gone rather than ported.

export { filteredList } from './filtered-list'
export type { ListSource } from './filtered-list'
export { selection } from './selection'
export { form } from './form'
export type { FormOptions } from './form'
