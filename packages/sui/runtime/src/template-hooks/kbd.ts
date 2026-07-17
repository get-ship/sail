// <use-kbd> — document-scoped keyboard shortcuts.
//
// Children are `<on key="..." action="..." />` records. Each binds a
// key combo (parsed by the runtime's `matchesKey` modifier) to a
// scope-resolved action — typically a controller `on.*` handler.
//
// Example:
//   <use-kbd>
//     <on key="/"      action="toggleSearch" />
//     <on key="cmd+n"  action="newAgent" />
//     <on key="escape" action="clearSelection" />
//   </use-kbd>
//
// Listener attaches to `document` at the hook's mount and tears down
// on unmount. Default actions for matched keys are NOT prevented; if
// an author needs preventDefault they can put a regular `on:keydown`
// on the element.

import type { TemplateHook, TemplateHookContext } from '../controller'
import { matchesKey } from '@ship/sail-runtime'

/** A single `<on key="..." action="..." />` record inside `<use-kbd>`. */
export interface KbdShortcut {
  key: string
  action: string
}

interface KbdAttrs extends Record<string, unknown> {
  /** Parsed `<on key="..." action="..." />` children. The hydrate
   * flow walks the `<use-kbd>` element's children and builds this
   * array; the hook factory consumes it. */
  shortcuts: KbdShortcut[]
  /** Action lookup — resolves `action="X"` to the actual handler
   * function. Provided by hydrate from the merged template scope. */
  resolve: (action: string) => ((event: KeyboardEvent) => void) | undefined
}

export const useKbdHook: TemplateHook<KbdAttrs, void> = {
  name: 'kbd',
  attrs: { shortcuts: 'literal', resolve: 'literal' },
  children: true,
  factory: (ctx: TemplateHookContext<KbdAttrs>) => {
    const { shortcuts, resolve } = ctx.attrs
    if (!shortcuts || shortcuts.length === 0) return

    const handler = (event: KeyboardEvent) => {
      for (const sc of shortcuts) {
        if (matchesKey(event, sc.key)) {
          const fn = resolve(sc.action)
          if (fn) fn(event)
          return
        }
      }
    }

    document.addEventListener('keydown', handler)
    ctx.cleanup(() => document.removeEventListener('keydown', handler))
  },
}
