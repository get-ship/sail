import type { Action } from '../action'

/**
 * `use:mediaFallback` — the L1 `<avatar>` element's behavior, written once in the
 * toolchain (no per-instance controller). It reads `src` / `icon-slug` /
 * `name` / `initials` off the element's OWN attributes (which merge onto the
 * compiles-away root) and renders either an `<img>` — falling back to initials
 * when the image is absent or fails to load — or the initials directly. The
 * frame (circle/square, tone, size) is the skin; this is the behavior layer.
 *
 * Re-renders when any of those attributes change, so a src swap retries and a
 * name change re-derives the initials.
 */
export const mediaFallback: Action<void> = (node) => {
  const el = node as HTMLElement
  const doc = el.ownerDocument

  const initialsOf = (): string => {
    const explicit = el.getAttribute('initials')?.trim()
    if (explicit) return explicit.slice(0, 2).toUpperCase()
    const name = el.getAttribute('name')?.trim() ?? ''
    return name.slice(0, 1).toUpperCase() || '?'
  }

  const showInitials = (): void => {
    el.textContent = ''
    const span = doc.createElement('span')
    span.setAttribute('part', 'initials')
    span.textContent = initialsOf()
    el.appendChild(span)
  }

  const render = (): void => {
    const slug = el.getAttribute('icon-slug')?.trim()
    const src = slug
      ? `https://cdn.simpleicons.org/${slug}`
      : (el.getAttribute('src')?.trim() ?? '')
    if (!src) {
      showInitials()
      return
    }
    // Image path: render the <img>; on a load error, fall back to initials
    // (Radix-parity — a broken image never leaves a broken-image rect).
    el.textContent = ''
    const img = doc.createElement('img')
    img.setAttribute('part', 'img')
    img.alt = el.getAttribute('name') ?? ''
    img.addEventListener('error', showInitials, { once: true })
    img.src = src
    el.appendChild(img)
  }

  render()
  const obs = new MutationObserver(render)
  obs.observe(el, {
    attributes: true,
    attributeFilter: ['src', 'icon-slug', 'name', 'initials'],
  })
  return { destroy: () => obs.disconnect() }
}
