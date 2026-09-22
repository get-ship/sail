/**
 * Bind the first preview specimen host under a surface root for draft paint.
 * Surfaces call this from mountAmbient / mount.
 *
 * Host discovery uses studio-shell Framework helpers (query + MutationObserver).
 * Pure product code never names document/querySelector/HTMLElement.
 */
import type { SailHost } from '@ship/sail-runtime'
import { hostQuerySelector, observeSubtree } from '@ship/sail-studio-shell'
import { bindPreviewHost, unbindPreviewHost } from './theme-draft-paint'

const SELECTOR =
  '[preview-specimen], [color-specimen], [shape-specimen], [overview-specimen], [assets-specimen], [components-specimen]'

export function watchPreviewHost(
  root: unknown,
  onBind?: (el: SailHost) => void,
): () => void {
  let paintHost: SailHost | null = null

  const tryBind = () => {
    const el = hostQuerySelector(root, SELECTOR)
    if (!el || el === paintHost) return
    if (paintHost) unbindPreviewHost(paintHost)
    paintHost = el
    bindPreviewHost(el)
    onBind?.(el)
  }

  tryBind()
  queueMicrotask(tryBind)
  const stopObserve = observeSubtree(root, tryBind)

  return () => {
    stopObserve()
    if (paintHost) unbindPreviewHost(paintHost)
    paintHost = null
  }
}
