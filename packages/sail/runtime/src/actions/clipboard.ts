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
  if (param != null && param !== '') {
    if (typeof param === 'string') return param
    if (typeof param === 'function') return param()
    const t = param.text
    if (t != null) return typeof t === 'function' ? t() : t
  }
  // Param-less paths (stdlib element mode forbids reactive action params):
  // 1) explicit data-clipboard-text
  // 2) nearest <code-block> host `code` attr / prop (live specimen)
  // 3) highlighted <pre> text inside that host
  // 4) host textContent fallback
  const explicit = node.getAttribute('data-clipboard-text')
  if (explicit != null && explicit !== '') return explicit

  const codeHost =
    node.closest('[data-sail-host="code-block"], .sail-code-block') ??
    node.closest('[code]')
  if (codeHost) {
    const attr = codeHost.getAttribute('code')
    if (attr != null && attr !== '') return attr
    const prop = (codeHost as HTMLElement & { code?: string }).code
    if (typeof prop === 'string' && prop !== '') return prop
    const pre = codeHost.querySelector('pre')
    const fromPre = (pre?.textContent ?? '').trim()
    if (fromPre) return fromPre
  }

  return (node.textContent ?? '').trim()
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
