// A prop descriptor carries the prop's value type. Built-ins additionally
// carry (conceptually) default + reflect + their theme contract — so naming
// `tone` in a props() block declares the prop AND opts into the tone system.
export type Prop<T> = { readonly __prop: T }

// ── Built-in props (framework-owned: union + default + reflect + contract) ──
export const tone: Prop<'default'|'primary'|'info'|'success'|'warning'|'destructive'|'accent'|'muted'> = { __prop: 'default' }
export const size: Prop<'sm'|'md'|'lg'> = { __prop: 'md' }

// ── Type markers ──
export const bool: Prop<boolean> = { __prop: false }
export const string: Prop<string> = { __prop: '' }
export function enumOf<const T extends string>(...values: T[]): Prop<T> { return { __prop: values[0] } }

// The collector: a record of descriptors, or a bare string[] for an inline enum.
export function props<T extends Record<string, Prop<unknown> | readonly string[]>>(defs: T): T { return defs }
