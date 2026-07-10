// Built-in `use:` actions — reusable element behaviors the L1 components lean
// on. Each is a plain `Action` (see ../action.ts): it receives the element,
// optionally a reactive param, and returns `{ update?, destroy? }`. Authored
// once here, attached anywhere with `use:<name>` — the primitive that removes
// hand-wired refs + effects for element behavior.

export { autosize } from './autosize'
export { clickOutside } from './click-outside'
export { anchor } from './anchor'
export type { AnchorPlacement } from './anchor'
export { dropzone } from './dropzone'
export { avatar } from './avatar'
