// currentNav — stamp `current="page"` / `aria-current="page"` on descendant
// links whose href matches the live path. Layout CEs often do not receive
// `ctx.route`; this is the DOM-side mark so a rail can show the selected page.
//
// `use:currentNav` on the <nav> (or any ancestor of the links).

import type { Action } from '@ship/sail-runtime'

function here(): string {
  if (typeof location === 'undefined') return ''
  const p = location.pathname
  return p.length > 1 && p.endsWith('/') ? p.slice(0, -1) : p
}

function hrefPath(a: Element): string {
  const raw = a.getAttribute('href') ?? ''
  if (!raw || raw.startsWith('#') || raw.startsWith('http')) return ''
  try {
    const u = new URL(raw, typeof location !== 'undefined' ? location.origin : 'http://local')
    const p = u.pathname
    return p.length > 1 && p.endsWith('/') ? p.slice(0, -1) : p
  } catch {
    return raw.split('?')[0] ?? ''
  }
}

function sync(root: Element): void {
  const path = here()
  root.querySelectorAll('a[href]').forEach((a) => {
    const href = hrefPath(a)
    const on = href !== '' && href === path
    if (on) {
      a.setAttribute('current', 'page')
      a.setAttribute('aria-current', 'page')
    } else {
      a.removeAttribute('current')
      a.removeAttribute('aria-current')
    }
  })
}

/**
 * Mark the in-page link that matches `location.pathname`.
 * Re-syncs on click (SPA) and popstate.
 */
export const currentNav: Action<undefined> = (node) => {
  const el = node as HTMLElement
  const run = (): void => sync(el)
  // use: runs before children are appended — wait a frame, then observe.
  const raf = requestAnimationFrame(run)
  const mo =
    typeof MutationObserver !== 'undefined'
      ? new MutationObserver(run)
      : null
  mo?.observe(el, { childList: true, subtree: true })
  const onClick = (): void => {
    queueMicrotask(run)
    requestAnimationFrame(run)
  }
  el.addEventListener('click', onClick)
  if (typeof window !== 'undefined') {
    window.addEventListener('popstate', run)
  }
  return {
    destroy: () => {
      cancelAnimationFrame(raf)
      mo?.disconnect()
      el.removeEventListener('click', onClick)
      if (typeof window !== 'undefined') window.removeEventListener('popstate', run)
    },
  }
}
