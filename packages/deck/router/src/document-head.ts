// Apply a page's static document head. The SPA shell uses sail.config
// `app` for first paint. Navigation applies the matched route's `head`
// (from the route table), or the compiled module if the table has none.

export interface SailHeadMeta {
  name?: string
  property?: string
  content: string
}

export interface SailHeadLink {
  rel: string
  href: string
}

export interface SailHead {
  title?: string
  lang?: string
  metas?: SailHeadMeta[]
  links?: SailHeadLink[]
}

const ATTR = 'data-sail-head'

function moduleHead(mod: unknown): SailHead | undefined {
  if (!mod || typeof mod !== 'object') return undefined
  const rec = mod as Record<string, unknown>
  const head = rec.__sailHead
  if (!head || typeof head !== 'object') return undefined
  return head as SailHead
}

function clearManaged(): void {
  if (typeof document === 'undefined') return
  document
    .querySelectorAll(`meta[${ATTR}], link[${ATTR}]`)
    .forEach((el) => el.remove())
}

export function applySailHead(head?: SailHead | null, fallbackTitle?: string): void {
  if (typeof document === 'undefined') return
  document.title = head?.title ?? fallbackTitle ?? document.title
  if (head?.lang) document.documentElement.lang = head.lang
  clearManaged()
  if (!head) return
  for (const meta of head.metas ?? []) {
    const el = document.createElement('meta')
    if (meta.name) el.setAttribute('name', meta.name)
    if (meta.property) el.setAttribute('property', meta.property)
    el.setAttribute('content', meta.content)
    el.setAttribute(ATTR, '')
    document.head.appendChild(el)
  }
  for (const link of head.links ?? []) {
    const el = document.createElement('link')
    el.setAttribute('rel', link.rel)
    el.setAttribute('href', link.href)
    el.setAttribute(ATTR, '')
    document.head.appendChild(el)
  }
}

export function applyDocumentHead(mod: unknown, fallbackTitle?: string): void {
  applySailHead(moduleHead(mod), fallbackTitle)
}
