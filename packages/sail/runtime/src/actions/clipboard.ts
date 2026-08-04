// clipboard — copy text to the system clipboard on host click.
// `use:clipboard` / `use:clipboard={textOrOpts}`
//
// Param shapes:
//   - string | (() => string) — text to copy (fallback: host textContent)
//   - { text?, onCopy? } — text plus optional success callback
//
// Transient success is the caller's concern (toast, "Copied!" label) via
// `onCopy`. The action only owns the write + click wiring.

import type { Action } from '../action'

export type ClipboardParam =
  | string
  | (() => string)
  | {
      text?: string | (() => string)
      /** Called after a successful clipboard write. */
      onCopy?: () => void
    }
  | undefined

function resolveText(node: Element, param: ClipboardParam): string {
  if (param == null || param === '') {
    return (node.textContent ?? '').trim()
  }
  if (typeof param === 'string') return param
  if (typeof param === 'function') return param()
  const t = param.text
  if (t == null) return (node.textContent ?? '').trim()
  return typeof t === 'function' ? t() : t
}

function resolveOnCopy(param: ClipboardParam): (() => void) | undefined {
  if (param != null && typeof param === 'object' && !Array.isArray(param)) {
    return param.onCopy
  }
  return undefined
}

/**
 * Copy param text (or host text content) to the clipboard on click.
 *
 * `use:clipboard` · `use:clipboard={url}` ·
 * `use:clipboard={{ text: brief, onCopy: markCopied }}`.
 */
export const clipboard: Action<ClipboardParam> = (node, param) => {
  let current = param

  const onClick = (e: Event): void => {
    e.preventDefault()
    const text = resolveText(node, current)
    const onCopy = resolveOnCopy(current)
    if (!text) return
    if (typeof navigator === 'undefined' || !navigator.clipboard?.writeText) {
      return
    }
    void navigator.clipboard.writeText(text).then(
      () => {
        onCopy?.()
      },
      () => {
        /* denied / insecure context */
      },
    )
  }

  node.addEventListener('click', onClick)
  return {
    update: (next) => {
      current = next
    },
    destroy: () => {
      node.removeEventListener('click', onClick)
    },
  }
}
