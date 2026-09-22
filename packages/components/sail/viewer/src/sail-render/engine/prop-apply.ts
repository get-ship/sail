// Prop application — the load-bearing wiring split (build-spec §Mount by
// catalog tag, step 3). Two paths by value shape:
//
//   - SCALAR (string / number / boolean) → reflected ATTRIBUTE with HTML
//     presence semantics: a truthy boolean writes a bare attribute
//     (`setAttribute(name,'')`), a false boolean removes it (never
//     `disabled="false"`); a null/empty string removes the attribute so the
//     element keeps its declared default. This half mirrors studio's
//     `applyProps`.
//   - STRUCTURED / IDENTITY (object, array, function, signal) → a JS PROPERTY
//     (`el[camelName] = value`), preserving identity. NEVER JSON-stringified
//     onto an attribute. This branch is NET-NEW — studio has no property path.
//
// Patching deltas in place (not recreating the node) is what lets a focused
// input keep focus and a component's internal reactive state survive a knob
// change (build-spec §Reactivity, the prop-change channel).

/** kebab/snake attribute name → camelCase JS property name. */
export function camelOf(name: string): string {
  return name.replace(/[-_]([a-z0-9])/g, (_m, c: string) => c.toUpperCase())
}

/** A value is "scalar" (attribute-reflected) when it is a string, number, or
 *  boolean. Everything else — objects, arrays, functions, signals — is
 *  structured and flows by JS-property identity. `null`/`undefined` are
 *  treated as scalar so they clear the attribute (declared-default restore). */
export function isScalar(value: unknown): boolean {
  if (value == null) return true
  const t = typeof value
  return t === 'string' || t === 'number' || t === 'boolean'
}

/** Apply a single scalar value as a reflected attribute with presence
 *  semantics. */
function applyScalarAttr(el: any, name: string, value: unknown): void {
  if (typeof value === 'boolean') {
    if (value) el.setAttribute(name, '')
    else el.removeAttribute(name)
    return
  }
  if (value == null || value === '') {
    el.removeAttribute(name)
    return
  }
  el.setAttribute(name, String(value))
}

/** Apply ONE prop to the createElement host, dispatching on value shape. */
export function applyProp(el: any, name: string, value: unknown): void {
  if (isScalar(value)) {
    applyScalarAttr(el, name, value)
    return
  }
  // Structured/identity prop — set as a JS property by camelCase name.
  // Identity is preserved (no stringify); the element reads it as a live
  // reference. A plain assignment is preferred, but some names shadow a
  // read-only DOM accessor (e.g. `dataset`) where `el.x = v` THROWS (getter
  // only) or silently no-ops; fall back to an own data property so the
  // structured value still lands.
  const key = camelOf(name)
  const target = el as unknown as Record<string, unknown>
  try {
    target[key] = value
  } catch {
    /* read-only accessor — fall through to defineProperty below */
  }
  if (target[key] !== value) {
    try {
      Object.defineProperty(el, key, { configurable: true, writable: true, value })
    } catch {
      /* non-configurable reserved accessor — the property path can't apply */
    }
  }
}

/**
 * Apply a full `propValues` record to the host. Used on the first mount and on
 * every prop-change delta (the engine holds the node; this patches in place,
 * no remount). Keys are applied in insertion order.
 */
export function applyProps(el: any, props: Record<string, unknown> | null | undefined): void {
  if (!props) return
  for (const [name, value] of Object.entries(props)) {
    applyProp(el, name, value)
  }
}
