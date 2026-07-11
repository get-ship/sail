import type { Action } from '../action'

/**
 * `use:themePicker` — the L1 `<theme-picker>` behavior. A radiogroup of the
 * framework's themes; selecting one sets `data-theme` on `<html>` (the theming
 * root), reflects `value`, wires role=radio + aria-checked + roving tabindex,
 * and emits `sui-theme-change`.
 */
export const themePicker: Action<void> = (node) => {
  const root = node as HTMLElement
  const target = (): HTMLElement => document.documentElement
  const optionsOf = (): HTMLElement[] =>
    Array.from(root.querySelectorAll<HTMLElement>('.sui-theme-option'))
  const valueOf = (el: Element): string => el.getAttribute('value') ?? ''
  const current = (): string => root.getAttribute('value') || target().getAttribute('data-theme') || ''

  if (!root.hasAttribute('role')) root.setAttribute('role', 'radiogroup')

  const sync = (): void => {
    const cur = current()
    const opts = optionsOf()
    const focusable = opts.find((o) => valueOf(o) === cur) ?? opts[0]
    for (const o of opts) {
      o.setAttribute('role', 'radio')
      o.setAttribute('aria-checked', valueOf(o) === cur ? 'true' : 'false')
      o.setAttribute('tabindex', o === focusable ? '0' : '-1')
    }
  }

  const setTheme = (v: string): void => {
    if (!v) return
    root.setAttribute('value', v)
    target().setAttribute('data-theme', v)
    sync()
    root.dispatchEvent(
      new CustomEvent('sui-theme-change', { detail: { theme: v }, bubbles: true, composed: true }),
    )
  }

  const optFrom = (e: Event): HTMLElement | null => {
    const o = (e.target as Element | null)?.closest<HTMLElement>('.sui-theme-option') ?? null
    return o && root.contains(o) ? o : null
  }

  const onClick = (e: MouseEvent): void => {
    const o = optFrom(e)
    if (o) setTheme(valueOf(o))
  }
  const onKeydown = (e: KeyboardEvent): void => {
    const o = optFrom(e)
    if (!o) return
    const opts = optionsOf()
    const i = opts.indexOf(o)
    let j = -1
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') j = (i + 1) % opts.length
    else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') j = (i - 1 + opts.length) % opts.length
    else if (e.key === 'Home') j = 0
    else if (e.key === 'End') j = opts.length - 1
    if (j >= 0) {
      e.preventDefault()
      setTheme(valueOf(opts[j]!))
      opts[j]!.focus()
    }
  }

  root.addEventListener('click', onClick)
  root.addEventListener('keydown', onKeydown)
  const mo = new MutationObserver(() => sync())
  mo.observe(root, { childList: true, subtree: true })
  sync()

  return {
    destroy: () => {
      root.removeEventListener('click', onClick)
      root.removeEventListener('keydown', onKeydown)
      mo.disconnect()
    },
  }
}
