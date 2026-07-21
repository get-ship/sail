// liveRegion — tone-driven role / aria-live for status banners.
//
// Capability mix-in: warning|destructive → role=alert + aria-live=assertive;
// else role=status + aria-live=polite. Re-syncs when `tone` changes. Static
// PE defaults (status/polite) already announce correctly before hydrate; this
// only sharpens urgent tones. Stacked with `use:dismissControl` on L1 <alert>.

import type { Action } from '../action'

/**
 * Sync host `role` / `aria-live` from the `tone` attribute. Urgent tones
 * (`warning`, `destructive`) become an assertive alert; everything else
 * stays status/polite. Observes `tone` so flips re-evaluate immediately.
 */
export const liveRegion: Action<void> = (node) => {
  const el = node as HTMLElement

  const syncLive = (): void => {
    const urgent =
      el.getAttribute('tone') === 'warning' || el.getAttribute('tone') === 'destructive'
    el.setAttribute('role', urgent ? 'alert' : 'status')
    el.setAttribute('aria-live', urgent ? 'assertive' : 'polite')
  }
  syncLive()
  const obs = new MutationObserver(syncLive)
  obs.observe(el, { attributes: true, attributeFilter: ['tone'] })

  return {
    destroy: () => {
      obs.disconnect()
    },
  }
}
