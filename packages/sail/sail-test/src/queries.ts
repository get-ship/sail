// Semantic queries over real DOM accessibility as stamped by ARIA-1.
// App tests locate UI via role, accessible name, and state — never
// data-sail-host, data-test, or DOM structure selectors.

export type ByRoleOptions = {
  name?: string | RegExp
  hidden?: boolean
  pressed?: boolean | 'mixed'
  expanded?: boolean
  selected?: boolean
  checked?: boolean | 'mixed'
  current?: boolean | string
  level?: number
  /**
   * String name matching only (ignored for RegExp). Default `true`: full
   * equality after whitespace normalize. `false`: substring match.
   */
  exact?: boolean
}

function matchesText(actual: string, expected: string | RegExp, exact = true): boolean {
  const n = actual.replace(/\s+/g, ' ').trim()
  if (typeof expected === 'string') {
    const e = expected.replace(/\s+/g, ' ').trim()
    return exact ? n === e : n.includes(e)
  }
  return expected.test(n)
}

function ariaBool(el: Element, attr: string): boolean | null {
  const v = el.getAttribute(attr)
  if (v === null) return null
  if (v === 'true') return true
  if (v === 'false') return false
  return null
}

function isAriaHidden(el: Element): boolean {
  let cur: Element | null = el
  while (cur) {
    if (cur.getAttribute('aria-hidden') === 'true') return true
    if (cur instanceof HTMLElement && cur.hidden) return true
    const style = typeof getComputedStyle === 'function' ? getComputedStyle(cur) : null
    if (style && (style.display === 'none' || style.visibility === 'hidden')) return true
    cur = cur.parentElement
  }
  return false
}

/** Accessible name — label / labelledby / text / title (ARIA-1 stamps). */
export function accessibleName(el: Element): string {
  const labelledBy = el.getAttribute('aria-labelledby')
  if (labelledBy) {
    const parts = labelledBy
      .split(/\s+/)
      .map((id) => document.getElementById(id)?.textContent ?? '')
      .join(' ')
    if (parts.trim()) return parts.replace(/\s+/g, ' ').trim()
  }
  const label = el.getAttribute('aria-label')
  if (label) return label.trim()

  if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement || el instanceof HTMLSelectElement) {
    if (el.id && typeof document !== 'undefined') {
      const lab = document.querySelector(`label[for="${cssEscape(el.id)}"]`)
      if (lab?.textContent) return lab.textContent.replace(/\s+/g, ' ').trim()
    }
    const wrap = el.closest('label')
    if (wrap) {
      const clone = wrap.cloneNode(true) as HTMLElement
      clone.querySelectorAll('input,textarea,select').forEach((n) => n.remove())
      const t = clone.textContent?.replace(/\s+/g, ' ').trim()
      if (t) return t
    }
    if (el instanceof HTMLInputElement && el.placeholder) return el.placeholder
  }

  if (el instanceof HTMLElement) {
    const t = el.innerText || el.textContent || ''
    const n = t.replace(/\s+/g, ' ').trim()
    if (n) return n
  }

  const title = el.getAttribute('title')
  if (title) return title.trim()
  if (el instanceof HTMLImageElement && el.alt) return el.alt.trim()
  return ''
}

function cssEscape(id: string): string {
  if (typeof CSS !== 'undefined' && typeof CSS.escape === 'function') return CSS.escape(id)
  return id.replace(/"/g, '\\"')
}

/** Implicit + explicit role for ARIA-1 / host anatomy stamps. */
export function getRole(el: Element): string | null {
  const explicit = el.getAttribute('role')
  if (explicit) return explicit

  const tag = el.tagName.toLowerCase()
  switch (tag) {
    case 'button':
      return 'button'
    case 'a':
      return el.hasAttribute('href') ? 'link' : null
    case 'nav':
      return 'navigation'
    case 'main':
      return 'main'
    case 'header':
      return 'banner'
    case 'footer':
      return 'contentinfo'
    case 'aside':
      return 'complementary'
    case 'form':
      return 'form'
    case 'img':
      return 'img'
    case 'table':
      return 'table'
    case 'ul':
    case 'ol':
      return 'list'
    case 'li':
      return 'listitem'
    case 'option':
      return 'option'
    case 'dialog':
      return 'dialog'
    case 'h1':
    case 'h2':
    case 'h3':
    case 'h4':
    case 'h5':
    case 'h6':
      return 'heading'
    case 'input': {
      const type = ((el as HTMLInputElement).type || 'text').toLowerCase()
      if (type === 'checkbox') return 'checkbox'
      if (type === 'radio') return 'radio'
      if (type === 'submit' || type === 'button' || type === 'reset' || type === 'image')
        return 'button'
      if (type === 'range') return 'slider'
      if (type === 'number') return 'spinbutton'
      if (type === 'search') return 'searchbox'
      return 'textbox'
    }
    case 'textarea':
      return 'textbox'
    case 'select':
      return 'combobox'
    case 'progress':
      return 'progressbar'
    default:
      return null
  }
}

function headingLevel(el: Element): number | null {
  const tag = el.tagName.toLowerCase()
  if (/^h[1-6]$/.test(tag)) return Number(tag[1])
  const al = el.getAttribute('aria-level')
  if (al && /^\d+$/.test(al)) return Number(al)
  return null
}

function matchesRole(el: Element, role: string, opts: ByRoleOptions): boolean {
  if (getRole(el) !== role) return false
  if (!opts.hidden && isAriaHidden(el)) return false

  if (opts.name !== undefined) {
    // exact defaults true for string names; RegExp ignores the flag.
    if (!matchesText(accessibleName(el), opts.name, opts.exact !== false)) return false
  }

  if (opts.pressed !== undefined) {
    const v = el.getAttribute('aria-pressed')
    if (opts.pressed === 'mixed') {
      if (v !== 'mixed') return false
    } else if (v !== String(opts.pressed)) return false
  }

  if (opts.expanded !== undefined) {
    if (ariaBool(el, 'aria-expanded') !== opts.expanded) return false
  }

  if (opts.selected !== undefined) {
    const aria = ariaBool(el, 'aria-selected')
    const sel =
      el instanceof HTMLOptionElement
        ? el.selected
        : el.hasAttribute('selected')
          ? true
          : null
    const effective = aria ?? sel
    if (effective !== opts.selected) return false
  }

  if (opts.checked !== undefined) {
    if (opts.checked === 'mixed') {
      if (el.getAttribute('aria-checked') !== 'mixed') return false
    } else {
      const aria = ariaBool(el, 'aria-checked')
      const native =
        el instanceof HTMLInputElement && (el.type === 'checkbox' || el.type === 'radio')
          ? el.checked
          : null
      if ((aria ?? native) !== opts.checked) return false
    }
  }

  if (opts.current !== undefined) {
    const cur = el.getAttribute('aria-current')
    if (opts.current === true) {
      if (!cur || cur === 'false') return false
    } else if (opts.current === false) {
      if (cur && cur !== 'false') return false
    } else if (cur !== opts.current) return false
  }

  if (opts.level !== undefined) {
    if (headingLevel(el) !== opts.level) return false
  }

  return true
}

function walk(root: ParentNode): Element[] {
  const out: Element[] = []
  const tree = root.querySelectorAll('*')
  if (root instanceof Element) out.push(root)
  for (const el of tree) out.push(el)
  return out
}

function findByRole(container: ParentNode, role: string, options: ByRoleOptions = {}): Element[] {
  return walk(container).filter((el) => matchesRole(el, role, options))
}

function fail(msg: string): never {
  throw new Error(msg)
}

/** Exactly one match by role (+ optional name/state). Throws if 0 or many. */
export function getByRole(
  container: ParentNode,
  role: string,
  options: ByRoleOptions = {},
): HTMLElement {
  const found = findByRole(container, role, options)
  if (found.length === 0) {
    fail(
      `getByRole: no element with role="${role}"` +
        (options.name !== undefined ? ` name=${String(options.name)}` : ''),
    )
  }
  if (found.length > 1) {
    fail(
      `getByRole: ${found.length} elements with role="${role}"` +
        (options.name !== undefined ? ` name=${String(options.name)}` : '') +
        ' — narrow with name or state',
    )
  }
  return found[0] as HTMLElement
}

/** First match or null. */
export function queryByRole(
  container: ParentNode,
  role: string,
  options: ByRoleOptions = {},
): HTMLElement | null {
  const found = findByRole(container, role, options)
  return (found[0] as HTMLElement) ?? null
}

/** All matches (may be empty). */
export function getAllByRole(
  container: ParentNode,
  role: string,
  options: ByRoleOptions = {},
): HTMLElement[] {
  return findByRole(container, role, options) as HTMLElement[]
}

function textMatches(el: Element, text: string | RegExp): boolean {
  if (!(el instanceof HTMLElement)) return false
  if (el.children.length > 0) {
    // Prefer leaf-ish text hosts so nested wrappers don't all match.
    const own = Array.from(el.childNodes)
      .filter((n) => n.nodeType === Node.TEXT_NODE)
      .map((n) => n.textContent ?? '')
      .join('')
      .replace(/\s+/g, ' ')
      .trim()
    if (own && matchesText(own, text)) return true
  }
  const full = (el.textContent ?? '').replace(/\s+/g, ' ').trim()
  return matchesText(full, text)
}

function findByText(container: ParentNode, text: string | RegExp): Element[] {
  return walk(container).filter((el) => {
    if (isAriaHidden(el)) return false
    return textMatches(el, text)
  })
}

/** Exactly one visible-text match. Prefer role queries when possible. */
export function getByText(container: ParentNode, text: string | RegExp): HTMLElement {
  const found = findByText(container, text)
  // Prefer the deepest matching element (most specific host).
  const ranked = [...found].sort((a, b) => depth(b) - depth(a))
  const best = ranked.filter((el) => !ranked.some((o) => o !== el && el.contains(o)))
  if (best.length === 0) fail(`getByText: no element with text ${String(text)}`)
  if (best.length > 1) {
    fail(`getByText: ${best.length} elements match text ${String(text)}`)
  }
  return best[0] as HTMLElement
}

export function queryByText(container: ParentNode, text: string | RegExp): HTMLElement | null {
  try {
    return getByText(container, text)
  } catch {
    return null
  }
}

function depth(el: Element): number {
  let d = 0
  let cur: Element | null = el
  while (cur) {
    d++
    cur = cur.parentElement
  }
  return d
}

/** Dispatch a bubbling event on an element found by role. */
export function click(el: Element, init?: MouseEventInit): boolean {
  return el.dispatchEvent(
    new MouseEvent('click', { bubbles: true, cancelable: true, ...init }),
  )
}
