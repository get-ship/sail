// Context-aware completions for .sail templates.
//
// Inspects the character immediately before the cursor (and a small
// look-back window) to decide which completion list to surface.
// Triggers are stable strings the static SUI_TEMPLATE_COMPLETIONS
// list also keys on — so the editor sees the same shape it does
// today, just with the *set* picked dynamically.

export interface SuiCompletionSource {
  trigger: string
  items: Array<{
    label: string
    detail?: string
    info?: string
    insert?: string
  }>
}

/** Hard-coded sources keyed by trigger. Source of truth for the
 *  modifier list: crates/sail/compiler/src/parse/template/attrs.rs */
const SOURCES: Record<string, SuiCompletionSource> = {
  'on:': {
    trigger: 'on:',
    items: [
      { label: 'click', detail: 'event' },
      { label: 'input', detail: 'event' },
      { label: 'change', detail: 'event' },
      { label: 'submit', detail: 'event' },
      { label: 'keydown', detail: 'event' },
      { label: 'keyup', detail: 'event' },
      { label: 'focus', detail: 'event' },
      { label: 'blur', detail: 'event' },
      { label: 'mouseenter', detail: 'event' },
      { label: 'mouseleave', detail: 'event' },
      { label: 'sail-change', detail: 'custom event (ship-code-editor, sail-input)' },
      { label: 'sail-tab-change', detail: 'custom event (sail-tabs)' },
    ],
  },
  '|': {
    trigger: '|',
    items: [
      { label: 'preventDefault', detail: 'modifier', info: 'Calls e.preventDefault() before handler' },
      { label: 'stopPropagation', detail: 'modifier', info: 'Calls e.stopPropagation() before handler' },
      { label: 'stopImmediatePropagation', detail: 'modifier' },
      { label: 'self', detail: 'modifier', info: 'Only fires when e.target === currentTarget' },
      { label: 'trusted', detail: 'modifier', info: 'Only fires when e.isTrusted' },
      { label: 'key:Enter', detail: 'key filter', insert: 'key:Enter' },
      { label: 'key:Escape', detail: 'key filter', insert: 'key:Escape' },
      { label: 'key:Tab', detail: 'key filter', insert: 'key:Tab' },
      { label: 'key:ArrowUp', detail: 'key filter', insert: 'key:ArrowUp' },
      { label: 'key:ArrowDown', detail: 'key filter', insert: 'key:ArrowDown' },
      { label: 'debounce:150', detail: 'modifier', insert: 'debounce:150', info: 'Trailing-edge debounce in ms' },
      { label: 'throttle:150', detail: 'modifier', insert: 'throttle:150', info: 'Leading-edge throttle in ms' },
    ],
  },
  'bind:': {
    trigger: 'bind:',
    items: [
      { label: 'value', detail: 'two-way bind to {signal}' },
      { label: 'checked', detail: 'two-way bind to {signal} (checkbox/radio)' },
      { label: 'this', detail: 'capture element ref into {signal}' },
      { label: 'group', detail: 'two-way bind for radio group / select multiple' },
    ],
  },
  'prop:': {
    trigger: 'prop:',
    items: [
      { label: 'value', detail: 'one-way set DOM property' },
      { label: 'checked', detail: 'one-way set DOM property' },
      { label: 'disabled', detail: 'one-way set DOM property' },
    ],
  },
  'class:': {
    trigger: 'class:',
    items: [
      { label: 'active', detail: 'class toggle: {signal} truthy → class on' },
    ],
  },
  '{#': {
    trigger: '{#',
    items: [
      { label: 'if', detail: 'block', insert: 'if condition}\n  \n{/if}' },
      { label: 'for', detail: 'block', insert: 'for item of items() key={item.id}}\n  \n{/for}' },
      { label: 'while-loading', detail: 'data block', insert: 'while-loading of data}\n  \n{/while-loading}' },
      { label: 'while-error', detail: 'data block', insert: 'while-error err of data}\n  \n{/while-error}' },
    ],
  },
  '{:': {
    trigger: '{:',
    items: [
      { label: 'else', detail: 'else branch', insert: 'else}' },
      { label: 'else if', detail: 'else-if branch', insert: 'else if condition}' },
    ],
  },
}

/** Pick the right completion source for the cursor position. Looks
 *  back at most 8 characters for a trigger string. Returns null if
 *  nothing recognizable precedes the cursor — caller can then fall
 *  back to the editor's built-in completion harness. */
export function completionsAt(source: string, offset: number): SuiCompletionSource | null {
  const safe = Math.max(0, Math.min(offset, source.length))
  const lookback = source.slice(Math.max(0, safe - 8), safe)
  // Longest triggers first so 'bind:' wins over ':' etc.
  for (const trigger of ['{#', '{:', 'bind:', 'prop:', 'class:', 'on:', '|']) {
    if (lookback.endsWith(trigger)) return SOURCES[trigger]!
  }
  return null
}

/** All sources — what the editor wants if it's going to wire static
 *  triggers and let CodeMirror handle the dispatch itself. */
export function allSources(): SuiCompletionSource[] {
  return Object.values(SOURCES)
}
