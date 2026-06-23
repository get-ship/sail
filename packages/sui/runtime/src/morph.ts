// DOM morph. Reconcile the contents of `target` to match `source`
// in-place, preserving DOM identity where possible (focus, scroll,
// selection, custom-element state, animation).
//
// This is a focused implementation — not a full idiomorph port. It
// handles the cases Sail's HMR + SSR-hydration use:
// - Same-tag elements: morph attributes + recurse children
// - Different-tag: replace
// - Text nodes: update nodeValue
// - Children align by id first (best signal), then tag+position
//
// Limitations:
// - Doesn't preserve `<select>` value across morphs (browser-specific)
// - Doesn't morph into shadow DOM (use a fresh re-render for those)
// - No content-similarity heuristics — only id and tag+position

/** Options that control which subtrees `morph` will descend into. */
export interface MorphOptions {
  /** Return `true` to leave a subtree untouched (e.g. third-party widgets). */
  skip?: (el: Element) => boolean
}

/**
 * Reconcile the contents of `target` to match `source` in-place,
 * preserving DOM identity where possible (focus, scroll, selection,
 * custom-element state, animations).
 *
 * `source` may be an `Element` whose shape `target` converges to, or
 * an HTML string (parsed via `<template>` and treated as the contents
 * of `target`).
 *
 * Children align by id first (best signal). Same-id nodes are reused
 * — moved into place if needed — even across reorders. Children
 * without ids align by tag + position. Different-tag elements are
 * replaced. Text and Comment nodes have their `nodeValue` patched.
 *
 * @param target DOM element to mutate in place.
 * @param source Element whose shape `target` should converge to, or
 *   HTML string to parse as `target`'s new contents.
 * @param opts Optional skip predicate.
 *
 * @example
 * ```ts
 * morph(rootEl, '<div>new content</div>')
 * morph(rootEl, otherElement, { skip: (el) => el.hasAttribute('data-no-morph') })
 * ```
 *
 * @see bindFor for keyed list reconciliation.
 */
export function morph(
  target: Element,
  source: string | Element,
  opts: MorphOptions = {},
): void {
  if (typeof source === 'string') {
    const tag = target.tagName.toLowerCase()
    const tmp = target.namespaceURI === 'http://www.w3.org/2000/svg'
      ? target.ownerDocument.createElementNS('http://www.w3.org/2000/svg', tag)
      : target.ownerDocument.createElement(tag)
    tmp.innerHTML = source
    morphChildren(target, tmp, opts)
    return
  }
  morphElement(target, source, opts)
}

function morphElement(target: Element, source: Element, opts: MorphOptions): void {
  if (opts.skip?.(target)) return
  if (target.tagName !== source.tagName) {
    target.replaceWith(source.cloneNode(true))
    return
  }
  morphAttributes(target, source)
  morphChildren(target, source, opts)
}

function morphAttributes(target: Element, source: Element): void {
  const tAttrs = Array.from(target.attributes)
  for (const a of tAttrs) {
    if (!source.hasAttribute(a.name)) target.removeAttribute(a.name)
  }
  const sAttrs = Array.from(source.attributes)
  for (const a of sAttrs) {
    if (target.getAttribute(a.name) !== a.value) {
      target.setAttribute(a.name, a.value)
    }
  }
  // Inputs need explicit value/checked sync; attribute alone is the
  // *default*, not the *current* value.
  if (target instanceof HTMLInputElement && source instanceof HTMLInputElement) {
    if (source.hasAttribute('value') && target.value !== source.value) {
      target.value = source.value
    }
    if (target.checked !== source.checked) {
      target.checked = source.checked
    }
  }
}

function morphChildren(target: Element, source: Element, opts: MorphOptions): void {
  // Id index of target's element children — lets us reuse nodes
  // across reorders, not just at matching indices.
  const targetById = new Map<string, Element>()
  for (const c of Array.from(target.children)) {
    if (c.id) targetById.set(c.id, c)
  }
  // Ids in source — used to recognise that a cursor target node is
  // wanted later (and shouldn't be claimed by tag+position now).
  const sourceIds = new Set<string>()
  for (const c of Array.from(source.children)) {
    if (c.id) sourceIds.add(c.id)
  }

  const sChildren = Array.from(source.childNodes)
  let cursor: Node | null = target.firstChild

  for (const s of sChildren) {
    // Skip any cursor target child whose id belongs to source but to
    // a *later* source position — leave it in place for its rightful
    // claimant.
    while (
      cursor &&
      cursor.nodeType === Node.ELEMENT_NODE &&
      (cursor as Element).id &&
      sourceIds.has((cursor as Element).id) &&
      !(s.nodeType === Node.ELEMENT_NODE && (s as Element).id === (cursor as Element).id)
    ) {
      cursor = cursor.nextSibling
    }

    let claim: Node | null = null
    if (s.nodeType === Node.ELEMENT_NODE) {
      const sId = (s as Element).id
      if (sId && targetById.has(sId)) {
        claim = targetById.get(sId)!
        targetById.delete(sId)
      }
    }
    if (!claim && cursor && nodesAlign(cursor, s)) {
      claim = cursor
    }

    if (claim) {
      if (claim === cursor) {
        cursor = cursor.nextSibling
      } else {
        target.insertBefore(claim, cursor)
      }
      morphNode(claim, s, target, opts)
    } else {
      target.insertBefore(s.cloneNode(true), cursor)
    }
  }

  while (cursor) {
    const next = cursor.nextSibling
    target.removeChild(cursor)
    cursor = next
  }
}

function nodesAlign(a: Node, b: Node): boolean {
  if (a.nodeType !== b.nodeType) return false
  if (a.nodeType === Node.ELEMENT_NODE) {
    return (a as Element).tagName === (b as Element).tagName
  }
  return true
}

function morphNode(t: Node, s: Node, parent: Element, opts: MorphOptions): void {
  if (t.nodeType !== s.nodeType) {
    parent.replaceChild(s.cloneNode(true), t)
    return
  }
  if (t.nodeType === Node.TEXT_NODE || t.nodeType === Node.COMMENT_NODE) {
    if (t.nodeValue !== s.nodeValue) t.nodeValue = s.nodeValue
    return
  }
  if (t.nodeType === Node.ELEMENT_NODE) {
    morphElement(t as Element, s as Element, opts)
  }
}
