// Text-interpolation hydrator.
//
// Direct-emit shape: the compiler walks to a `<!--sail:text:bN-->` open
// comment and passes it in. We collapse the SSR'd nodes between this
// open comment and its sibling `<!--/sail-->` close into a single Text
// node we own, then run an effect that writes the stringified getter
// value to it on every signal change.
//
// `getter` is a closure the compiler emitted from the IR expression
// against the captured scope — no string-eval at runtime.

import { effect } from '@ship/sail-runtime'

const TEXT_OPEN_PREFIX = 'sail:text:'
const TEXT_CLOSE = '/sail'

export function hydrateText(open: Comment, getter: () => unknown): () => void {
  const parent = open.parentNode
  if (!parent) return () => {}
  let close: Comment | null = null
  for (let s: Node | null = open.nextSibling; s; s = s.nextSibling) {
    if (s.nodeType === 8 && (s as Comment).data === TEXT_CLOSE) {
      close = s as Comment
      break
    }
  }
  if (!close) return () => {}
  let s: Node | null = open.nextSibling
  while (s && s !== close) {
    const next: Node | null = s.nextSibling
    parent.removeChild(s)
    s = next
  }
  const text = (parent.ownerDocument ?? document).createTextNode('')
  parent.insertBefore(text, close)
  return effect(() => {
    const v = getter()
    text.data = v == null ? '' : String(v)
  })
}

// Single-pass scan for every `<!--sail:text:bN-->` opener under `root`.
// The compiler-emitted `hydrate(root, ctx)` calls this once and then
// dispatches each text binding by key — one tree walk for the whole
// route rather than one per binding.
export function collectTextOpens(root: Node): Record<string, Comment> {
  const owner = (root as Element).ownerDocument ?? document
  const walker = owner.createTreeWalker(root, 128 /* SHOW_COMMENT */)
  const out: Record<string, Comment> = Object.create(null)
  let n: Node | null = walker.currentNode
  while ((n = walker.nextNode())) {
    if (n.nodeType !== 8) continue
    const data = (n as Comment).data
    if (!data.startsWith(TEXT_OPEN_PREFIX)) continue
    out[data.slice(TEXT_OPEN_PREFIX.length)] = n as Comment
  }
  return out
}
