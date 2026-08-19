// Spotlight paint is Style / platform CSS, not a injected sheet or mailbox.
// This stamp only records pointer presence (`data-on`) for a consumer sheet.

import type { Action } from '../action'

export const spotlight: Action<void> = (node) => {
  const host = node as HTMLElement
  const onMove = (): void => {
    host.setAttribute('data-on', 'true')
  }
  const onLeave = (): void => {
    host.removeAttribute('data-on')
  }
  host.addEventListener('pointermove', onMove)
  host.addEventListener('pointerleave', onLeave)
  return {
    destroy: () => {
      host.removeEventListener('pointermove', onMove)
      host.removeEventListener('pointerleave', onLeave)
      host.removeAttribute('data-on')
    },
  }
}
