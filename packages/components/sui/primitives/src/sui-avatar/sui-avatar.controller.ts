import { controller, effect, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-avatar.sui'

// Radix-parity: when an image is provided but fails to load, fall back
// to initials instead of leaving a broken-image rect in the avatar.
// Mirrors Radix Avatar.Image's `onLoadingStatusChange('error')` →
// Avatar.Fallback compose pattern.

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const errored = signal(false)
    return { errored }
  },

  mount(state, host) {
    // Reset `errored` whenever the source changes so a recovered URL
    // gets another shot at loading.
    let lastSrc = ''
    const stopSrcReset = effect(() => {
      const src = (state.iconSlug || state.src) as string
      if (src !== lastSrc) {
        lastSrc = src
        state.errored = false
      }
    })

    const onImgError = (e: Event) => {
      const tag = (e.target as HTMLElement | null)?.tagName?.toLowerCase()
      if (tag !== 'img') return
      state.errored = true
    }
    // `error` doesn't bubble, but does compose via { capture: true }.
    host.shadowRoot?.addEventListener('error', onImgError, true)

    return () => {
      stopSrcReset()
      host.shadowRoot?.removeEventListener('error', onImgError, true)
    }
  },
})
