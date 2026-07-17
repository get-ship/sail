// server-dom.ts — a minimal, isomorphic SERVER DOM for ship-js SSR.
//
// The SAME synthesized render (`createElement`/`appendChild`/`bindText`/
// `bindFor`/…) and the SAME runtime DOM bindings (`bind.ts`, `for.ts`,
// `if.ts`, `morph.ts`) that run in the browser run UNCHANGED against this,
// then each node serializes itself to a first-paint HTML string. One render
// artifact for server + browser — not a second string-renderer.
//
// `installServerDom()` installs `globalThis.document` + `globalThis.Node`
// (and dummy `HTMLElement`/`HTMLInputElement` so morph's `instanceof` guards
// are inert), and returns a `root()` factory for the serialization container.
//
// Coverage:
//   * text / comment (control-flow anchors, `nodeType` 8) / element / fragment
//   * `class` / `style` / arbitrary attributes
//   * SVG namespace — `createElementNS` + namespaced serialization (empty
//     SVG elements self-close: `<circle/>`)
//   * `innerHTML` + the Node-navigation surface morph needs
//     (`firstChild`/`nextSibling`/`children`/`cloneNode`/`replaceChild`/
//     `attributes`/`hasAttribute`/`replaceWith`), so `bindHtml` first-paints
// Event listeners are server-side no-ops.

const ESC: Record<string, string> = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }
const esc = (s: string): string => s.replace(/[&<>"]/g, (c) => ESC[c]!)

/** HTML void elements — never emit a closing tag. */
const VOID = new Set([
  'area', 'base', 'br', 'col', 'embed', 'hr', 'img', 'input', 'link', 'meta',
  'param', 'source', 'track', 'wbr',
])

/** Raw-text elements — their content is never entity-escaped, in the HTML
 *  spec or in a browser's `innerHTML` serialization. A `<style>` node's
 *  CSS text (which routinely contains `>` from descendant-combinator
 *  selectors) must round-trip byte-identical to what the client's
 *  `el.textContent = css` produces, or isomorphic-render parity breaks. */
const RAW_TEXT = new Set(['style', 'script'])

const SVG_NS = 'http://www.w3.org/2000/svg'
const ELEMENT_NODE = 1
const TEXT_NODE = 3
const COMMENT_NODE = 8
/** Sentinel for the raw-HTML node (innerHTML round-trip). Opaque to morph. */
const RAW_NODE = 99

type SNode = TextNode | CommentNode | El | RawHtml
type Parent = El | Frag

/** Index of `node` within its parent + 1, or null. Backs `nextSibling`. */
function siblingOf(node: SNode): SNode | null {
  const p = node.parentNode
  if (!p) return null
  const i = p.childNodes.indexOf(node)
  return i >= 0 ? p.childNodes[i + 1] ?? null : null
}

function detach(node: SNode): void {
  const p = node.parentNode
  if (p) {
    const i = p.childNodes.indexOf(node)
    if (i >= 0) p.childNodes.splice(i, 1)
    node.parentNode = null
  }
}

/** Append `child` to `parent`, expanding fragments (DOM move semantics). */
function adopt(parent: Parent, child: SNode | Frag): void {
  if (child instanceof Frag) {
    for (const c of [...child.childNodes]) {
      detach(c)
      parent.childNodes.push(c)
      c.parentNode = parent
    }
    child.childNodes = []
    return
  }
  detach(child)
  parent.childNodes.push(child)
  child.parentNode = parent
}

class TextNode {
  nodeType = TEXT_NODE
  parentNode: Parent | null = null
  _t = ''
  /** Slice 2 — hydration key the synthetic bind* calls record so the SSR
   *  serialization wraps the node in `<!--sail:text:bN-->...<!--/sail-->`. The
   *  client `hydrate()` reads these markers back via `collectTextOpens`.
   *  `null` on the browser DOM (only meaningful in the server-DOM). */
  __hydrateKey: string | null = null
  get textContent(): string {
    return this._t
  }
  set textContent(v: unknown) {
    this._t = v == null ? '' : String(v)
  }
  get nodeValue(): string {
    return this._t
  }
  set nodeValue(v: unknown) {
    this._t = v == null ? '' : String(v)
  }
  get nextSibling(): SNode | null {
    return siblingOf(this)
  }
  cloneNode(_deep?: boolean): TextNode {
    const t = new TextNode()
    t._t = this._t
    t.__hydrateKey = this.__hydrateKey
    return t
  }
  toHTML(): string {
    // Hydration marker — wrap so `collectTextOpens(root)` finds the key.
    // The element-mark contract emits the key as an HTML comment open +
    // close pair around the text contents. Inner text is escaped as usual.
    if (this.__hydrateKey) {
      return `<!--sail:text:${this.__hydrateKey}-->${esc(this._t)}<!--/sail-->`
    }
    return esc(this._t)
  }
}

class CommentNode {
  nodeType = COMMENT_NODE
  parentNode: Parent | null = null
  data: string
  constructor(data: string) {
    this.data = data
  }
  get nodeValue(): string {
    return this.data
  }
  set nodeValue(v: unknown) {
    this.data = v == null ? '' : String(v)
  }
  get nextSibling(): SNode | null {
    return siblingOf(this)
  }
  cloneNode(_deep?: boolean): CommentNode {
    return new CommentNode(this.data)
  }
  toHTML(): string {
    return `<!--${this.data}-->`
  }
}

/** Holds an `innerHTML` string verbatim — first-paint round-trips
 *  `bindHtml`/`morph` output without a server-side HTML parser. */
class RawHtml {
  nodeType = RAW_NODE
  parentNode: Parent | null = null
  html: string
  constructor(html: string) {
    this.html = html
  }
  get nextSibling(): SNode | null {
    return siblingOf(this)
  }
  cloneNode(_deep?: boolean): RawHtml {
    return new RawHtml(this.html)
  }
  toHTML(): string {
    return this.html
  }
}

/** One simple compound selector: `tag.class1.class2#id[attr][attr2="v"]`. */
interface Compound {
  tag: string | null
  classes: string[]
  id: string | null
  attrs: Array<{ name: string; value: string | null }>
}

const COMPOUND_TOKEN = /^[a-zA-Z][\w-]*|\.[\w-]+|#[\w-]+|\[[^\]]+\]/g

function parseCompound(text: string): Compound {
  const c: Compound = { tag: null, classes: [], id: null, attrs: [] }
  for (const tok of text.match(COMPOUND_TOKEN) ?? []) {
    if (tok[0] === '.') c.classes.push(tok.slice(1))
    else if (tok[0] === '#') c.id = tok.slice(1)
    else if (tok[0] === '[') {
      const inner = tok.slice(1, -1)
      const eq = inner.indexOf('=')
      if (eq < 0) c.attrs.push({ name: inner.trim(), value: null })
      else {
        const name = inner.slice(0, eq).trim()
        const value = inner.slice(eq + 1).trim().replace(/^['"]|['"]$/g, '')
        c.attrs.push({ name, value })
      }
    } else c.tag = tok
  }
  return c
}

/** `sel, sel` → alternatives; each alternative → a descendant-combinator chain. */
function parseSelector(selector: string): Compound[][] {
  return selector.split(',').map((group) =>
    group
      .trim()
      .split(/\s+/)
      .filter(Boolean)
      .map(parseCompound),
  )
}

function compoundMatches(el: El, c: Compound): boolean {
  if (c.tag && c.tag.toLowerCase() !== el.tag.toLowerCase()) return false
  if (c.id && el.getAttribute('id') !== c.id) return false
  for (const cls of c.classes) if (!el._classes.has(cls)) return false
  for (const a of c.attrs) {
    if (a.value === null) {
      if (!el.hasAttribute(a.name)) return false
    } else if (el.getAttribute(a.name) !== a.value) return false
  }
  return true
}

/** Does `el` satisfy `chain` (a descendant-combinator compound list), anchored at `el`? */
function chainMatchesAt(el: El, chain: Compound[]): boolean {
  if (!compoundMatches(el, chain[chain.length - 1]!)) return false
  let idx = chain.length - 2
  let anc = el.parentNode instanceof El ? el.parentNode : null
  while (anc && idx >= 0) {
    if (compoundMatches(anc, chain[idx]!)) idx--
    anc = anc.parentNode instanceof El ? anc.parentNode : null
  }
  return idx < 0
}

function matchesSelector(el: El, selector: string): boolean {
  return parseSelector(selector).some((chain) => chainMatchesAt(el, chain))
}

function descendants(node: Parent): El[] {
  const out: El[] = []
  for (const c of node.childNodes) {
    if (c.nodeType === ELEMENT_NODE) {
      out.push(c as El)
      out.push(...descendants(c as El))
    }
  }
  return out
}

class ClassList {
  constructor(private el: El) {}
  add(...names: string[]): void {
    for (const n of names) this.el._classes.add(n)
  }
  remove(...names: string[]): void {
    for (const n of names) this.el._classes.delete(n)
  }
  contains(n: string): boolean {
    return this.el._classes.has(n)
  }
  toggle(n: string, force?: boolean): boolean {
    const on = force ?? !this.el._classes.has(n)
    if (on) this.el._classes.add(n)
    else this.el._classes.delete(n)
    return on
  }
}

class Style {
  props = new Map<string, string>()
  setProperty(k: string, v: string): void {
    this.props.set(k, v)
  }
  removeProperty(k: string): void {
    this.props.delete(k)
  }
  serialize(): string {
    return [...this.props].map(([k, v]) => `${k}: ${v}`).join('; ')
  }
}

class El {
  nodeType = ELEMENT_NODE
  tag: string
  parentNode: Parent | null = null
  attrs = new Map<string, string>()
  /** Slice 2 — hydration marker attributes (`on:event`, `class:name`, etc.)
   *  the synthetic bind* calls record via `markHydrateAttr`. The El's
   *  `toHTML()` folds them into the serialized attribute set so the
   *  client `collectElementMarkers` finds them. `null` on the browser
   *  DOM (only meaningful in the server-DOM). */
  __hydrateAttr: Record<string, string> | null = null
  /** Slice 2 — boundary key for control-flow wrappers (`<sail-if key="bN">`).
   *  Set via `markHydrateBoundary`. Browser DOM uses the standard
   *  `setAttribute('key', ...)` path; the server-DOM shortcuts by
   *  carrying the key in this field for serialization. */
  __hydrateBoundary: string | null = null
  childNodes: SNode[] = []
  _classes = new Set<string>()
  classList: ClassList
  style = new Style()
  ownerDocument: ServerDocument
  namespaceURI: string | null = null
  constructor(tag: string, doc: ServerDocument) {
    this.tag = tag
    this.ownerDocument = doc
    this.classList = new ClassList(this)
  }
  get tagName(): string {
    return this.namespaceURI === SVG_NS ? this.tag : this.tag.toUpperCase()
  }
  get id(): string {
    return this.attrs.get('id') ?? ''
  }
  set id(v: string) {
    this.attrs.set('id', v)
  }
  setAttribute(name: string, value: unknown): void {
    if (name === 'class') {
      this._classes = new Set(String(value).split(/\s+/).filter(Boolean))
      return
    }
    this.attrs.set(name, value == null ? '' : String(value))
  }
  removeAttribute(name: string): void {
    if (name === 'class') this._classes.clear()
    else this.attrs.delete(name)
  }
  getAttribute(name: string): string | null {
    if (name === 'class') return this._classes.size ? [...this._classes].join(' ') : null
    return this.attrs.get(name) ?? null
  }
  hasAttribute(name: string): boolean {
    return this.getAttribute(name) != null
  }
  get attributes(): Array<{ name: string; value: string }> {
    const out: Array<{ name: string; value: string }> = []
    if (this._classes.size) out.push({ name: 'class', value: [...this._classes].join(' ') })
    if (this.style.props.size) out.push({ name: 'style', value: this.style.serialize() })
    for (const [k, v] of this.attrs) out.push({ name: k, value: v })
    return out
  }
  appendChild<T extends SNode | Frag>(child: T): T {
    adopt(this, child)
    return child
  }
  insertBefore<T extends SNode | Frag>(node: T, ref: SNode | null): T {
    if (ref == null) return this.appendChild(node)
    const i = this.childNodes.indexOf(ref)
    if (node instanceof Frag) {
      const kids = [...node.childNodes]
      node.childNodes = []
      let at = i < 0 ? this.childNodes.length : i
      for (const c of kids) {
        detach(c)
        this.childNodes.splice(at++, 0, c)
        c.parentNode = this
      }
      return node
    }
    detach(node)
    if (i < 0) this.childNodes.push(node)
    else this.childNodes.splice(i, 0, node)
    node.parentNode = this
    return node
  }
  removeChild<T extends SNode>(node: T): T {
    const i = this.childNodes.indexOf(node)
    if (i >= 0) this.childNodes.splice(i, 1)
    node.parentNode = null
    return node
  }
  replaceChild<T extends SNode>(next: SNode, old: T): T {
    const i = this.childNodes.indexOf(old)
    if (i >= 0) {
      detach(next)
      this.childNodes[i] = next
      next.parentNode = this
      old.parentNode = null
    }
    return old
  }
  replaceWith(node: SNode): void {
    if (this.parentNode instanceof El) this.parentNode.replaceChild(node, this)
  }
  get firstChild(): SNode | null {
    return this.childNodes[0] ?? null
  }
  get nextSibling(): SNode | null {
    return siblingOf(this)
  }
  get children(): El[] {
    return this.childNodes.filter((n): n is El => n.nodeType === ELEMENT_NODE)
  }
  addEventListener(): void {}
  removeEventListener(): void {}
  matches(selector: string): boolean {
    return matchesSelector(this, selector)
  }
  closest(selector: string): El | null {
    let node: El | null = this
    while (node) {
      if (matchesSelector(node, selector)) return node
      node = node.parentNode instanceof El ? node.parentNode : null
    }
    return null
  }
  querySelector(selector: string): El | null {
    return descendants(this).find((el) => matchesSelector(el, selector)) ?? null
  }
  querySelectorAll(selector: string): El[] {
    return descendants(this).filter((el) => matchesSelector(el, selector))
  }
  get textContent(): string {
    return this.childNodes.map((c) => ('textContent' in c ? c.textContent : '')).join('')
  }
  set textContent(v: unknown) {
    const t = new TextNode()
    t.textContent = v
    this.childNodes = []
    this.appendChild(t)
  }
  get innerHTML(): string {
    return this.childNodes.map((c) => c.toHTML()).join('')
  }
  set innerHTML(html: unknown) {
    this.childNodes = []
    this.appendChild(new RawHtml(html == null ? '' : String(html)))
  }
  cloneNode(deep?: boolean): El {
    const el = new El(this.tag, this.ownerDocument)
    el.namespaceURI = this.namespaceURI
    el.attrs = new Map(this.attrs)
    el._classes = new Set(this._classes)
    el.__hydrateAttr = this.__hydrateAttr ? { ...this.__hydrateAttr } : null
    el.__hydrateBoundary = this.__hydrateBoundary
    for (const [k, v] of this.style.props) el.style.props.set(k, v)
    if (deep) for (const c of this.childNodes) el.appendChild(c.cloneNode(true))
    return el
  }
  toHTML(): string {
    const cls = this._classes.size ? ` class="${esc([...this._classes].join(' '))}"` : ''
    const styleStr = this.style.props.size ? ` style="${esc(this.style.serialize())}"` : ''
    // Slice 2 — fold the hydration marker attrs (`on:click="bN"`,
    // `class:active="bN"`, etc.) into the serialized set so the
    // client `collectElementMarkers` picks them up. Boundary keys
    // (for `<sail-if key="bN">` etc.) go on as `key="bN"` (the
    // `collectElementMarkers` walker reads `getAttribute('key')`).
    const markerAttrs = this.__hydrateAttr
      ? Object.entries(this.__hydrateAttr).map(([k, v]) => ` ${k}="${esc(v)}"`)
      : []
    const boundaryAttr = this.__hydrateBoundary
      ? ` key="${esc(this.__hydrateBoundary)}"`
      : ''
    const attrs = [...this.attrs].map(([k, v]) => ` ${k}="${esc(v)}"`).join('')
    const open = `<${this.tag}${cls}${styleStr}${attrs}${markerAttrs.join('')}${boundaryAttr}`
    const inner = RAW_TEXT.has(this.tag)
      ? this.childNodes.map((c) => (c.nodeType === TEXT_NODE ? (c as TextNode)._t : c.toHTML())).join('')
      : this.childNodes.map((c) => c.toHTML()).join('')
    // HTML void elements never close; empty SVG elements self-close.
    if (VOID.has(this.tag)) return `${open}>`
    if (this.namespaceURI === SVG_NS && inner === '') return `${open}/>`
    return `${open}>${inner}</${this.tag}>`
  }
}

class Frag {
  childNodes: SNode[] = []
  appendChild<T extends SNode>(child: T): T {
    adopt(this, child)
    return child
  }
}

/** The shape the synthesized render + bindings call as `globalThis.document`.
 *  Internal — it references the non-exported `El`; the public surface is
 *  `installServerDom` + `ServerRoot`. */
interface ServerDocument {
  createElement(tag: string): El
  createElementNS(ns: string, tag: string): El
  createTextNode(text: string): TextNode
  createComment(data: string): CommentNode
  createDocumentFragment(): Frag
  /** Server-side no-op — emitted renders install `delegateEvents` at module
   *  scope; events never fire on the server. */
  addEventListener(): void
}

/** The SSR mount/serialize surface — hides the server-DOM element type. */
export interface ServerRoot {
  /** A fresh mount container, cast to `HTMLElement` for the render's `mount()`. */
  container(): HTMLElement
  /** Serialize a container's children to a first-paint HTML string. */
  html(container: HTMLElement): string
}

/**
 * Install the server DOM as `globalThis.document` / `globalThis.Node` (plus
 * inert `HTMLElement`/`HTMLInputElement` so morph's `instanceof` guards don't
 * throw), and return the mount/serialize surface. Idempotent.
 */
export function installServerDom(): ServerRoot {
  const doc: ServerDocument = {
    createElement: (tag) => new El(tag, doc),
    createElementNS: (ns, tag) => {
      const el = new El(tag, doc)
      el.namespaceURI = ns
      return el
    },
    createTextNode: (text) => {
      const t = new TextNode()
      t.textContent = text
      return t
    },
    createComment: (data) => new CommentNode(data),
    createDocumentFragment: () => new Frag(),
    addEventListener: () => {},
  }
  const g = globalThis as Record<string, unknown>
  g.document = doc
  // `Node` must be a constructor (morph reads `Node.ELEMENT_NODE`; other code
  // may do `x instanceof Node`). A real `Node` (jsdom) is left untouched. In a
  // bare V8 isolate / node there is none, so install a constructor carrying the
  // nodeType constants — server-DOM nodes are never instances of it, which is
  // exactly right (`instanceof Node` → false, constant access → works).
  if (!g.Node) {
    g.Node = class {
      static readonly ELEMENT_NODE = ELEMENT_NODE
      static readonly TEXT_NODE = TEXT_NODE
      static readonly COMMENT_NODE = COMMENT_NODE
    }
  }
  if (!g.HTMLElement) g.HTMLElement = class {}
  if (!g.HTMLInputElement) g.HTMLInputElement = class {}
  return {
    container: () => new El('__root__', doc) as unknown as HTMLElement,
    html: (container) => {
      const el = container as unknown as El
      return el.childNodes.map((c) => c.toHTML()).join('')
    },
  }
}
