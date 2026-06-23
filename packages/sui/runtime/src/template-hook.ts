// template-hook.ts — authoring contract for `<use-*>` template hooks.
//
// Split out of `controller.ts` to keep both files under the 300-line
// engineering cap. Template hooks are unrelated to controller hooks
// at the type level — they describe `<use-*>` directive elements in
// the template, not composable section bundles in the controller.

/**
 * Per-attribute binding shape for a template hook. Three kinds:
 *
 * - `'literal'` — author writes `attr="value"`; the hook receives the
 *   string verbatim. No expressions allowed in attribute values.
 * - `'identifier'` — author writes `as="X"` or `attr="ident"`; the
 *   compiler resolves `ident` to a scope binding. Used for hooks that
 *   contribute a name to scope (e.g. `as="shell"`).
 * - `'binding'` — author writes `bind:attr={ident}`; the runtime passes
 *   the resolved Signal/value reference. The compiler statically
 *   requires that the value be a bare identifier, never an expression.
 */
export type TemplateHookAttrKind = 'literal' | 'identifier' | 'binding'

/**
 * Mount-time context passed to a template hook's factory. The factory
 * may register cleanups (run on unmount) and may return a value that
 * the runtime exposes under the hook's `as=` name.
 */
export interface TemplateHookContext<Attrs> {
  /**
   * Resolved attribute values. Literal kinds come through as strings;
   * identifier and binding kinds come through as the resolved scope
   * binding (typically a `Signal<T>` or value reference).
   */
  attrs: Attrs
  /**
   * The owning element this hook is attached to. For non-rendered
   * hooks (`<use-kbd>`, `<use-title>`) this is the document body;
   * for hooks attached to a specific element via `bind:target=` the
   * element ref is passed through `attrs`.
   */
  el: HTMLElement
  /** Register a cleanup to run on unmount. */
  cleanup: (fn: () => void) => void
}

/**
 * Authoring contract for a template hook (`<use-*>` element).
 *
 * Hooks are registered with the runtime's hook registry by name; the
 * compiler recognizes `<use-<name>>` tags and emits the matching
 * factory invocation in the rendered DOM.
 *
 * @typeParam Attrs Resolved attribute shape (after the runtime
 *   discriminates by `attrs[k]` kind).
 * @typeParam Contributes The value (if any) the hook contributes back
 *   to scope under the `as=` name.
 */
export interface TemplateHook<Attrs = object, Contributes = unknown> {
  /** Tag suffix — `<use-<name>>`. Lowercase ASCII + hyphens. */
  name: string
  /**
   * Per-attribute kind schema. Keys are attribute names; values are
   * the kind the runtime should resolve them as. Omit attributes the
   * hook doesn't accept; the compiler rejects unknown attrs.
   */
  attrs?: { [K in keyof Attrs]: TemplateHookAttrKind }
  /**
   * Whether this hook accepts child template fragments. When `true`,
   * children may contain expressions, control flow, slots, and
   * nested elements (per the standard template grammar). When `false`,
   * the compiler rejects any child content.
   */
  children?: boolean
  /**
   * Mount-time factory. Invoked with parsed attrs/children + scope;
   * may register cleanups and may return a value to expose under the
   * hook's `as=` name.
   */
  factory: (ctx: TemplateHookContext<Attrs>) => Contributes | void
}
