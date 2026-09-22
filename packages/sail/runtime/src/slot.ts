// Light-DOM slot resolution.
//
// Sail components use light DOM. Native <slot> distribution is shadow-only,
// so the runtime walks the host's children and places them at slot anchors
// at hydrate time.
//

/**
 * Declaration of one slot: name, anchor placeholder, and optional
 * fallback. The compiler emits an array of these for each component
 * that exposes slots; the runtime consumes the array via `resolveSlots`.
 */
export interface SlotDecl {
  /** Slot name; default slot is `'default'`. */
  name: string
  /** Anchor comment where slot content goes. */
  anchor: Comment
  /** Render fallback if no fill provided. Returns nodes to insert. */
  fallback?: () => Node[]
}

/**
 * Distribute a component's pre-existing children into its slot anchors,
 * matching native `<slot>` distribution semantics. Elements with
 * `slot="X"` route to the named anchor; everything else (un-named
 * elements, Text, Comment) routes to the default slot. The `slot`
 * attribute is stripped during routing.
 *
 * Should be called AFTER the component's template has rendered (so
 * anchors exist) but BEFORE lifecycle hooks fire (so authors see a
 * fully-distributed tree).
 *
 * @param host Custom-element host whose children are being distributed.
 * @param slots Slot declarations for this component.
 * @param originalChildren Snapshot of `host.childNodes` taken before
 *   the template overwrote the host's content (see `snapshotChildren`).
 *
 * @example
 * ```ts
 * const original = snapshotChildren(host)
 * host.replaceChildren(template)
 * resolveSlots(host, [
 *   { name: 'default', anchor: defaultAnchor },
 *   { name: 'icon',    anchor: iconAnchor, fallback: () => [defaultIcon()] },
 * ], original)
 * ```
 *
 * @see snapshotChildren, slotFilled.
 */
export function resolveSlots(host: Element, slots: SlotDecl[], originalChildren: Node[]): void {
  const fills = new Map<string, Node[]>()
  for (const child of originalChildren) {
    let name = 'default'
    if (child instanceof Element) {
      name = child.getAttribute('slot') ?? 'default'
      if (child.hasAttribute('slot')) child.removeAttribute('slot')
    }
    const list = fills.get(name) ?? []
    list.push(child)
    fills.set(name, list)
  }

  // Track which slots received caller-provided content (vs. fell back
  // to defaults). Read by `slotFilled(host, name)` and the compiler-
  // injected local of the same name. Non-empty meaningful nodes only —
  // pure-whitespace text fills count as "not filled" (they would
  // otherwise mark every slot with leading whitespace as filled).
  const filledNames = new Set<string>()
  for (const [name, nodes] of fills) {
    const hasMeaning = nodes.some((n) => {
      if (n.nodeType === 1) return true
      if (n.nodeType === 3) return (n.textContent || '').trim().length > 0
      return false
    })
    if (hasMeaning) filledNames.add(name)
  }
  ;(host as unknown as { __shipSetSlotFilled: Set<string> }).__shipSetSlotFilled = filledNames

  for (const slot of slots) {
    // Slot anchors inside conditional branches (e.g. `<if {x}><slot
    // name="y"/></if>`) aren't in the DOM at mount time. Tolerate
    // undefined anchors here so the rest of the slot wiring still
    // resolves; the missing slot becomes effectively hidden until
    // its branch renders.
    if (!slot || !slot.anchor) continue
    const filled = fills.get(slot.name) ?? []
    const parent = slot.anchor.parentNode
    if (!parent) continue
    if (filled.length > 0) {
      for (const n of filled) parent.insertBefore(n, slot.anchor)
    } else if (slot.fallback) {
      const nodes = slot.fallback()
      for (const n of nodes) parent.insertBefore(n, slot.anchor)
    }
  }
  for (const slot of slots) {
    if (slot && slot.anchor) slot.anchor.remove()
  }
}

/**
 * Was the named slot filled with author-provided content at mount, or
 * did it fall back? Lets templates gate styling / aux content on slot
 * presence without manual DOM probing. Pure-whitespace text fills
 * count as "not filled" so leading whitespace doesn't pollute the answer.
 *
 * Static — fills are evaluated once at mount; subsequent reparenting
 * does not update the result.
 *
 * The compiler injects a `slotFilled` local into the component setup,
 * so script-block code can call `slotFilled('icon')` without passing
 * the host explicitly.
 *
 * @param host Component host element.
 * @param name Slot name (use `'default'` for the unnamed slot).
 * @returns True if the slot received meaningful caller-provided content.
 *
 * @example
 * ```ts
 * if (!slotFilled(host, 'icon')) host.classList.add('iconless')
 * ```
 *
 * @see resolveSlots.
 */
export function slotFilled(host: Element, name: string): boolean {
  const set = (host as unknown as { __shipSetSlotFilled?: Set<string> }).__shipSetSlotFilled
  return set?.has(name) ?? false
}

/**
 * Snapshot the host's current child nodes (Elements, Text, and Comment)
 * before its innerHTML is replaced by the component template. Text and
 * comment children survive the round-trip so native-slot semantics are
 * preserved — `<my-component>Hi</my-component>` keeps the `Hi` text.
 *
 * @param host Component host element.
 * @returns A new array containing each child node in document order.
 *
 * @example
 * ```ts
 * const original = snapshotChildren(host)
 * host.replaceChildren(template)
 * resolveSlots(host, slots, original)
 * ```
 *
 * @see resolveSlots.
 */
export function snapshotChildren(host: Element): Node[] {
  return Array.from(host.childNodes)
}
