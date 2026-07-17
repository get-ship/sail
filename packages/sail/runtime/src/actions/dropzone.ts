// dropzone — turn a container (a `<div dropzone>`) wrapping a native
// `<input type=file>` into a drag-and-drop upload surface. `use:dropzone` on
// the container. The native input stays the source of truth (form
// participation + the file picker); the action adds what the platform doesn't:
// click-to-browse, drag events, an active-state hook for the skin, and a
// reflected file summary.
//
// A `<div>` rather than a `<label>` on purpose: the compiler emits a form-label
// skin for every `<label>` (display/weight), which would fight the drop-zone
// skin. The div carries no such skin, so the surface stays clean.

import type { Action } from '../action'

/**
 * Wire click-to-browse + drag-and-drop onto a container holding an
 * `<input type="file">`.
 *
 * Clicking the surface opens the picker; dragging a file over it sets
 * `[data-drag]` (the skin highlights on that); dropping assigns the files to
 * the input and fires `change` so bindings/validation react exactly as for a
 * click-selected file; the selection is reflected into a `[data-dz-files]`
 * element if one is present.
 */
export const dropzone: Action<undefined> = (node) => {
  const el = node as HTMLElement
  // `use:` actions run before the element's children are appended, so resolve
  // the input/summary lazily (at event time they exist), not at init.
  const getInput = () => el.querySelector<HTMLInputElement>('input[type="file"]')
  const getSummary = () => el.querySelector<HTMLElement>('[data-dz-files]')
  let summaryDefault: string | null = null

  const reflect = () => {
    const input = getInput()
    const summary = getSummary()
    if (!input || !summary) return
    if (summaryDefault === null) summaryDefault = summary.textContent ?? ''
    const n = input.files?.length ?? 0
    summary.textContent =
      n === 0 ? summaryDefault : n === 1 ? (input.files?.[0]?.name ?? '1 file') : `${n} files selected`
  }

  const onClick = (e: MouseEvent) => {
    // The synthetic click from input.click() re-bubbles here — ignore it.
    const input = getInput()
    if (input && e.target !== input && !input.contains(e.target as Node | null)) {
      input.click()
    }
  }
  const onOver = (e: DragEvent) => {
    e.preventDefault()
    el.setAttribute('data-drag', '')
  }
  const onLeave = (e: DragEvent) => {
    // Ignore leaves into descendants — only clear when the pointer truly exits.
    if (!el.contains(e.relatedTarget as Node | null)) el.removeAttribute('data-drag')
  }
  const onDrop = (e: DragEvent) => {
    e.preventDefault()
    el.removeAttribute('data-drag')
    const input = getInput()
    if (input && e.dataTransfer?.files?.length) {
      input.files = e.dataTransfer.files
      input.dispatchEvent(new Event('change', { bubbles: true }))
    }
  }

  el.addEventListener('click', onClick)
  el.addEventListener('dragover', onOver)
  el.addEventListener('dragleave', onLeave)
  el.addEventListener('drop', onDrop)
  // `change` bubbles, so catch the input's change on the container — no need to
  // hold an input ref that doesn't exist yet at init.
  el.addEventListener('change', reflect)

  return {
    destroy: () => {
      el.removeEventListener('click', onClick)
      el.removeEventListener('dragover', onOver)
      el.removeEventListener('dragleave', onLeave)
      el.removeEventListener('drop', onDrop)
      el.removeEventListener('change', reflect)
    },
  }
}
