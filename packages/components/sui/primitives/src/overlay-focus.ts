// Shared focus + scroll-lock utilities for overlay components (dialog, drawer).
// No side effects at import time; all state is per-module-instance.

const FOCUSABLE = [
  'a[href]', 'area[href]',
  'button:not([disabled])',
  'input:not([disabled]):not([type="hidden"])',
  'select:not([disabled])', 'textarea:not([disabled])',
  'iframe', 'object', 'embed', 'summary',
  '[contenteditable]:not([contenteditable="false"])',
  '[tabindex]:not([tabindex^="-"])',
].join(',')

export function focusableWithin(root: ParentNode): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(el => {
    if (el.hasAttribute('disabled')) return false
    if (el.getAttribute('aria-hidden') === 'true') return false
    return true
  })
}

export class FocusReturn {
  private saved: HTMLElement | null = null
  save(): void { this.saved = document.activeElement as HTMLElement ?? null }
  restore(): void {
    const t = this.saved; this.saved = null
    if (t && typeof t.focus === 'function') t.focus()
  }
}

let _scrollLockCount = 0
let _savedOverflow: string | null = null
let _savedPadding: string | null = null

export function acquireScrollLock(): void {
  if (typeof document === 'undefined') return
  _scrollLockCount += 1
  if (_scrollLockCount > 1) return
  const body = document.body
  _savedOverflow = body.style.overflow
  _savedPadding = body.style.paddingRight
  const scrollW =
    typeof window !== 'undefined'
      ? window.innerWidth - document.documentElement.clientWidth
      : 0
  if (scrollW > 0) {
    const cur = parseFloat(getComputedStyle(body).paddingRight || '0') || 0
    body.style.paddingRight = `${cur + scrollW}px`
  }
  body.style.overflow = 'hidden'
}

export function releaseScrollLock(): void {
  if (typeof document === 'undefined') return
  if (_scrollLockCount === 0) return
  _scrollLockCount -= 1
  if (_scrollLockCount > 0) return
  const body = document.body
  body.style.overflow = _savedOverflow ?? ''
  body.style.paddingRight = _savedPadding ?? ''
  _savedOverflow = null
  _savedPadding = null
}
