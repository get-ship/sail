// Minimal DOM morph — surgical patches that preserve focus, selection,
// and form state on inputs whose identity is unchanged.
//
// This is an intentionally small implementation, not a full port of
// morphdom. The semantics required by `patch-elements`:
//
//  - When the incoming element's tag matches the target's tag, update
//    attributes in place and recurse into children. The element node
//    identity is preserved — focus, scroll, IME state, value-on-input,
//    Selection ranges all survive.
//  - When tags differ, replace the target with the incoming element.
//  - Children are matched first by `id`, then by `data-key`, then by
//    position. Anything in the target without a peer in the new tree
//    is removed; anything new without a peer is inserted.
//
// Limitations vs morphdom (acceptable for v1):
//  - No `onBeforeNodeDiscarded` / `onElUpdated` hook surface.
//  - No special-case for `<input type=file>` value preservation.
//  - No special-case for SVG namespace edge cases beyond what
//    `outerHTML` round-tripping already handles.

const KEY_ATTR = 'data-key'

/**
 * Morph `target` toward the structure described by `html`.
 *
 * `html` MUST parse to a single root element. The root is matched
 * against `target` (tag + attrs + children). When the root tags
 * differ, `target` is replaced; otherwise the existing node identity
 * is preserved.
 */
export function morph(target: Element, html: string): void {
  const fresh = parseFragmentRoot(html, target.ownerDocument)
  if (!fresh) return
  if (sameKind(target, fresh)) {
    morphElement(target, fresh)
  } else {
    target.replaceWith(fresh)
  }
}

function parseFragmentRoot(html: string, doc: Document): Element | null {
  const tpl = doc.createElement('template')
  tpl.innerHTML = html.trim()
  const first = tpl.content.firstElementChild
  return first
}

function sameKind(a: Element, b: Element): boolean {
  return a.tagName === b.tagName
}

function morphElement(from: Element, to: Element): void {
  syncAttributes(from, to)
  morphChildren(from, to)
}

function syncAttributes(from: Element, to: Element): void {
  const toAttrs = to.attributes
  for (let i = 0; i < toAttrs.length; i++) {
    const a = toAttrs.item(i)!
    if (from.getAttribute(a.name) !== a.value) {
      from.setAttribute(a.name, a.value)
    }
  }
  for (let i = from.attributes.length - 1; i >= 0; i--) {
    const a = from.attributes.item(i)!
    if (!to.hasAttribute(a.name)) {
      from.removeAttribute(a.name)
    }
  }
}

function nodeKey(n: Node): string | null {
  if (n.nodeType !== 1) return null
  const el = n as Element
  const id = el.getAttribute('id')
  if (id) return `id:${id}`
  const k = el.getAttribute(KEY_ATTR)
  if (k) return `key:${k}`
  return null
}

function morphChildren(from: Element, to: Element): void {
  const fromKeyed = new Map<string, Element>()
  for (const child of Array.from(from.children)) {
    const k = nodeKey(child)
    if (k) fromKeyed.set(k, child)
  }

  const toChildren = Array.from(to.childNodes)
  let cursor: Node | null = from.firstChild

  for (const target of toChildren) {
    const targetKey = nodeKey(target)
    let match: Node | null = null

    if (targetKey && fromKeyed.has(targetKey)) {
      match = fromKeyed.get(targetKey)!
      fromKeyed.delete(targetKey)
    } else if (cursor && cursor.nodeType === target.nodeType && nodeKey(cursor) === null) {
      // Position-based match for unkeyed nodes of the same type.
      if (target.nodeType !== 1 || sameKind(cursor as Element, target as Element)) {
        match = cursor
      }
    }

    if (match) {
      if (match !== cursor) {
        from.insertBefore(match, cursor)
      } else {
        cursor = cursor!.nextSibling
      }
      mergeNode(match, target)
    } else {
      // No peer — clone the incoming node into place.
      const clone = target.cloneNode(true)
      from.insertBefore(clone, cursor)
    }
  }

  // Anything left after the last target child (and unmatched keyed
  // entries) is excess — drop it.
  while (cursor) {
    const next = cursor.nextSibling
    from.removeChild(cursor)
    cursor = next
  }
  for (const leftover of fromKeyed.values()) {
    if (leftover.parentNode === from) from.removeChild(leftover)
  }
}

function mergeNode(into: Node, from: Node): void {
  if (into.nodeType !== from.nodeType) {
    ;(into as ChildNode).replaceWith(from.cloneNode(true))
    return
  }
  if (into.nodeType === 1) {
    morphElement(into as Element, from as Element)
    return
  }
  if (into.nodeValue !== from.nodeValue) {
    into.nodeValue = from.nodeValue
  }
}
