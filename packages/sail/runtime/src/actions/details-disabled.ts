// detailsDisabled — suppress <details> activation while [disabled].
//
// Native <details> has no disabled state. This mix-in preventDefaults click
// and Enter/Space (capture) while the host carries [disabled]. Stacked with
// `use:detailsToggle` on the L1 <accordion> root.

import type { Action } from '../action'

/**
 * While the host has `[disabled]`, suppress activation paths native
 * <details>/<summary> would otherwise honor: click + Enter/Space, both in
 * capture so the default toggle never runs.
 */
export const detailsDisabled: Action<void> = (node) => {
  const el = node as HTMLElement

  const onClickCapture = (e: MouseEvent): void => {
    if (el.hasAttribute('disabled')) e.preventDefault()
  }
  const onKeydownCapture = (e: KeyboardEvent): void => {
    if (el.hasAttribute('disabled') && (e.key === 'Enter' || e.key === ' ')) e.preventDefault()
  }

  el.addEventListener('click', onClickCapture, true)
  el.addEventListener('keydown', onKeydownCapture, true)

  return {
    destroy: () => {
      el.removeEventListener('click', onClickCapture, true)
      el.removeEventListener('keydown', onKeydownCapture, true)
    },
  }
}
