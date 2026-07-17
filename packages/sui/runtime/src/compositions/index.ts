// COMPOSITIONS — plain functions over the reactive primitives.
//
// These lived in a separate package (@ship/sail-std) that an author had to know
// about and install. That is a decision, and a decision is variance: you do not
// install `form()` any more than you install `signal()`. If it returns state it
// is a composition, and compositions are part of the language.
export { filteredList } from './filtered-list'
export type { ListSource } from './filtered-list'
export { selection } from './selection'
export { form } from './form'
export type { FormOptions } from './form'
