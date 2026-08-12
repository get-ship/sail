/**
 * Structural UI event shapes for Pure product code.
 *
 * Do not name DOM lib types (`Event`, `HTMLElement`, `HTMLInputElement`, …)
 * under Pure — the type gate rejects them. Templates still hand native events
 * at runtime; these shapes describe the fields we read without coupling to DOM.
 */

export type UiTarget = {
  value?: string
  checked?: boolean
  closest?: (selector: string) => UiTarget | null
  getAttribute?: (name: string) => string | null
  tagName?: string
  href?: string
}

/** Custom-event detail bag commonly mined by surfaces (tabs, pickers). */
export type UiEventDetail = {
  value?: string
  name?: string
  [k: string]: unknown
}

export type UiEvent = {
  preventDefault(): void
  stopPropagation(): void
  target: UiTarget | null
  currentTarget?: UiTarget | null
  detail?: UiEventDetail
  button?: number
}

export function inputValue(e: UiEvent): string {
  return e.target?.value ?? ''
}

export function inputChecked(e: UiEvent): boolean {
  return !!e.target?.checked
}

export function selectValue(e: UiEvent): string {
  return e.target?.value ?? ''
}

/** Walk from event target with `closest` and read an attribute. */
export function closestAttr(e: UiEvent, selector: string, attr: string): string {
  return e.target?.closest?.(selector)?.getAttribute?.(attr) ?? ''
}

export function isMiddleButton(e: UiEvent): boolean {
  return e.button === 1
}

/** Read a string field from custom-event detail (value / name / arbitrary key). */
export function detailField(e: UiEvent, key: 'value' | 'name' | string): string {
  const d = e.detail
  if (!d || typeof d !== 'object') return ''
  const v = d[key]
  return typeof v === 'string' ? v : ''
}
