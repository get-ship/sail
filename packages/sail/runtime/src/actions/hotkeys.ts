// hotkeys — declarative keyboard map on the host's scoped root.
// `use:hotkeys={map}` where map is `{ 'mod+k': handler, … }`.
//
// Key syntax matches `on.ts` (`matchesKey`) — no second parser.
// Default: ignore when focus is in INPUT / TEXTAREA / [contenteditable].
// Per-key opt-in: value may be `{ run, allowInInputs?: true }`.

import type { Action } from '../action'
import { matchesKey } from '@ship/sail-runtime/internal'

export type HotkeyHandler = (e: KeyboardEvent) => void

export type HotkeyEntry =
  | HotkeyHandler
  | {
      run: HotkeyHandler
      /** When true, fire even if focus is in an editable field. */
      allowInInputs?: boolean
    }

/** Map of key-spec → handler (or handler + opts). */
export type HotkeysMap = Record<string, HotkeyEntry>

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof Element)) return false
  const el = target as HTMLElement
  const tag = el.tagName
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true
  if (el.isContentEditable) return true
  return el.closest('[contenteditable=""], [contenteditable="true"]') != null
}

function entryOf(v: HotkeyEntry): { run: HotkeyHandler; allowInInputs: boolean } {
  if (typeof v === 'function') return { run: v, allowInInputs: false }
  return { run: v.run, allowInInputs: !!v.allowInInputs }
}

/**
 * Listen for key combos on the host (bubble). Page-level maps go on the
 * page root; widget maps go on that widget. Not document-global.
 *
 * `use:hotkeys={{ 'mod+s': save, Escape: close }}`.
 */
export const hotkeys: Action<HotkeysMap | undefined> = (node, map) => {
  let current: HotkeysMap = map ?? {}

  const onKey = (e: KeyboardEvent): void => {
    const editable = isEditableTarget(e.target)
    for (const [spec, raw] of Object.entries(current)) {
      if (!matchesKey(e, spec)) continue
      const { run, allowInInputs } = entryOf(raw)
      if (editable && !allowInInputs) continue
      e.preventDefault()
      run(e)
      return
    }
  }

  node.addEventListener('keydown', onKey)

  return {
    update: (next) => {
      current = next ?? {}
    },
    destroy: () => {
      node.removeEventListener('keydown', onKey)
    },
  }
}
