import type { Action } from '../action'
import { highlight } from './highlight'

/**
 * `use:syntaxHighlight` — the `<code-block>` element's behavior.
 *
 * Element-mode stdlib cannot pass `use:syntaxHighlight={code}` (static splice).
 * At use sites the compiler emits (order matters):
 *   runAction(el, syntaxHighlight)
 *   bindAttr(el, 'code', () => …)
 * so the first action paint often sees no `code` yet. We re-read after the
 * current effect flush (microtask + rAF) and observe attribute mutations.
 *
 * Source resolution (first non-empty wins):
 *  1. Action param when present (free `use:syntaxHighlight={…}` outside stdlib)
 *  2. Host CE / stamped root `.code` property / `code` attribute
 *  3. Action node's `code` attribute
 *
 * The highlighter escapes everything and only ever emits its own `tok-*` spans —
 * so `innerHTML` here cannot inject. Colours come from the `--syntax-*` theme
 * tokens, which means the code block re-themes with everything else.
 */
export type SyntaxHighlightParam = string | null | undefined

function findHost(el: HTMLElement): HTMLElement | null {
  if (
    el.getAttribute('data-sail-host') === 'code-block' ||
    el.classList.contains('sail-code-block')
  ) {
    return el
  }
  return (
    (el.closest('[data-sail-host="code-block"]') as HTMLElement | null) ??
    (el.closest('.sail-code-block') as HTMLElement | null) ??
    (el.closest('code-block') as HTMLElement | null)
  )
}

function readCode(node: HTMLElement | null): string {
  if (!node) return ''
  const prop = (node as HTMLElement & { code?: unknown }).code
  if (typeof prop === 'string' && prop.length > 0) return prop
  if (typeof prop === 'function') {
    try {
      const v = (prop as () => unknown)()
      if (typeof v === 'string' && v.length > 0) return v
    } catch {
      /* ignore */
    }
  }
  const attr = node.getAttribute('code')
  if (attr != null && attr.length > 0) return attr
  return ''
}

function resolveSource(el: HTMLElement, param: SyntaxHighlightParam): string {
  if (typeof param === 'string' && param.length > 0) return param
  const host = findHost(el)
  const fromHost = readCode(host)
  if (fromHost.length > 0) return fromHost
  return readCode(el)
}

export const syntaxHighlight: Action<SyntaxHighlightParam> = (node, param) => {
  const el = node as HTMLElement
  const pre = el.querySelector('pre') ?? el
  let current = param
  const host = findHost(el) ?? el
  let last = ''

  const render = (): void => {
    const src = resolveSource(el, current)
    if (src === last && pre.childNodes.length > 0) return
    last = src
    pre.innerHTML = highlight(src)
  }

  render()

  // bindAttr runs as a sibling effect after this action mounts — re-read after
  // the current flush so the first paint is not permanently empty.
  queueMicrotask(render)
  const raf =
    typeof requestAnimationFrame === 'function'
      ? requestAnimationFrame(() => {
          render()
        })
      : 0

  const mo = new MutationObserver(render)
  mo.observe(el, { attributes: true, attributeFilter: ['code'] })
  if (host !== el) {
    mo.observe(host, { attributes: true, attributeFilter: ['code'] })
  }

  return {
    update: (next) => {
      current = next
      render()
    },
    destroy: () => {
      mo.disconnect()
      if (raf && typeof cancelAnimationFrame === 'function') {
        cancelAnimationFrame(raf)
      }
    },
  }
}
