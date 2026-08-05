// Viewport forcing for structural `<when {token}>` — uses matchMedia, the
// same seam bindWhen observes. No private poke of when.ts caches beyond the
// documented test reset when present.

/** Theme breakpoint min-widths (aligned with Style conditions / WHEN-1). */
export const BREAKPOINT_MIN_WIDTH: Record<'mobile' | 'tablet' | 'desktop', number> = {
  mobile: 390,
  tablet: 768,
  desktop: 1280,
}

export type BreakpointToken = keyof typeof BREAKPOINT_MIN_WIDTH

type MediaStub = {
  matches: boolean
  media: string
  onchange: null
  addListener: () => void
  removeListener: () => void
  addEventListener: (type: string, cb: EventListenerOrEventListenerObject) => void
  removeEventListener: (type: string, cb: EventListenerOrEventListenerObject) => void
  dispatchEvent: () => boolean
  _listeners: Set<() => void>
  _setMatches: (v: boolean) => void
}

function parseMinWidth(query: string): number | null {
  const m = query.match(/min-width:\s*(\d+(?:\.\d+)?)(px)?/i)
  if (!m) return null
  return Number(m[1])
}

/**
 * Install a matchMedia that reports matches for queries whose min-width is
 * ≤ `widthPx`. Returns a restore function. Call before mounting when/bindWhen
 * consumers so shared listeners see the stub.
 */
export function installViewport(widthPx: number): () => void {
  if (typeof window === 'undefined') {
    return () => {}
  }

  const previous = window.matchMedia.bind(window)
  const stubs = new Map<string, MediaStub>()

  const make = (query: string): MediaStub => {
    const existing = stubs.get(query)
    if (existing) return existing
    const min = parseMinWidth(query)
    const listeners = new Set<() => void>()
    const stub: MediaStub = {
      get matches() {
        if (min === null) return false
        return widthPx >= min
      },
      media: query,
      onchange: null,
      addListener() {},
      removeListener() {},
      addEventListener(_type, cb) {
        listeners.add(typeof cb === 'function' ? cb : () => cb.handleEvent(new Event('change')))
      },
      removeEventListener(_type, cb) {
        listeners.delete(typeof cb === 'function' ? cb : () => cb.handleEvent(new Event('change')))
      },
      dispatchEvent() {
        return false
      },
      _listeners: listeners,
      _setMatches() {
        for (const l of listeners) l()
      },
    }
    stubs.set(query, stub)
    return stub
  }

  window.matchMedia = ((query: string) => make(query)) as typeof window.matchMedia

  return () => {
    window.matchMedia = previous
    stubs.clear()
  }
}

/** Force a named breakpoint token (mobile / tablet / desktop). */
export function forceBreakpoint(token: BreakpointToken): () => void {
  return installViewport(BREAKPOINT_MIN_WIDTH[token])
}
