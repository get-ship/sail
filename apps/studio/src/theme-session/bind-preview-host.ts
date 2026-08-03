/**
 * Bind the first preview specimen host under a surface root for draft paint.
 * Surfaces call this from mountAmbient / mount.
 */
import { bindPreviewHost, unbindPreviewHost } from './theme-draft-paint'

const SELECTOR =
  '[data-preview-specimen], [data-color-specimen], [data-shape-specimen], [data-overview-specimen]'

export function watchPreviewHost(
  root: ParentNode,
  onBind?: (el: HTMLElement) => void,
): () => void {
  let paintHost: HTMLElement | null = null

  const tryBind = () => {
    const el = root.querySelector(SELECTOR) as HTMLElement | null
    if (!el || el === paintHost) return
    if (paintHost) unbindPreviewHost(paintHost)
    paintHost = el
    bindPreviewHost(el)
    onBind?.(el)
  }

  tryBind()
  queueMicrotask(tryBind)
  const mo =
    typeof MutationObserver !== 'undefined'
      ? new MutationObserver(() => tryBind())
      : null
  if (mo && root instanceof Node) {
    mo.observe(root, { childList: true, subtree: true })
  }

  return () => {
    mo?.disconnect()
    if (paintHost) unbindPreviewHost(paintHost)
    paintHost = null
  }
}
