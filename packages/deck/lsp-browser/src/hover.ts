// Browser-side hover harness for .sail templates.
//
// v2a: static dictionary lookups for directives / modifiers / block
// keywords. Zero-bundle hover. Covers ~80% of pedagogically useful
// hover queries an editor user will make (what does `on:click|self`
// do, what does `{#while-loading}` do).
//
// v2b (lazy): identifier resolution via oxc-parser-wasm — kicked off
// on first editor focus or tutorial nav so first-hover latency is
// near-zero. Not yet wired here; the worker's hover endpoint falls
// back to v2a until preloadIdentifiers() succeeds.

export interface SuiHoverInfo {
  /** Markdown — rendered in the editor's hover tooltip. */
  contents: string
  /** Optional 1-based range to highlight in the editor when the
   *  tooltip is open. */
  range?: { line: number; column: number; endColumn?: number }
}

interface Entry {
  /** Display label in the hover. */
  signature: string
  /** Plain prose, markdown allowed. */
  doc: string
  /** Where to learn more — rendered as a "see also" link. */
  see?: string
}

const DIRECTIVES: Record<string, Entry> = {
  'on:': {
    signature: '`on:<event>={handler}`',
    doc: 'Bind a DOM event listener. `<event>` is any DOM event name (`click`, `input`, `submit`, …) or a custom event (`sail-change`, `sail-tab-change`). Compose with `|<modifier>` pipes to wrap the handler.',
    see: 'docs/ship-sail/references/docs/template.md#events',
  },
  'bind:': {
    signature: '`bind:<target>={signal}`',
    doc: 'Two-way binding. `bind:value` mirrors a `<input>`/`<select>`/`<textarea>` value into the signal and back; `bind:checked` for checkboxes; `bind:this` captures the DOM element ref into the signal.',
    see: 'docs/ship-sail/references/tutorial/06-bind-value/',
  },
  'prop:': {
    signature: '`prop:<name>={expr}`',
    doc: 'One-way assignment to a DOM property (not attribute). Use when the property and attribute diverge (`<input prop:value={…}>` vs the attribute).',
  },
  'class:': {
    signature: '`class:<name>={truthy}`',
    doc: 'Conditional class toggle. Adds the class when the expression is truthy, removes it otherwise. Composes with `class=` (static classes survive).',
    see: 'docs/ship-sail/references/tutorial/14-class-style/',
  },
  'style:': {
    signature: '`style:<--var>={value}`',
    doc: 'Set a CSS custom property (or any CSS property) from an expression. Most useful for `style:--my-var={…}` so component CSS reading `var(--my-var)` picks up the value.',
    see: 'docs/ship-sail/references/tutorial/14-class-style/',
  },
}

const MODIFIERS: Record<string, Entry> = {
  preventDefault: {
    signature: '`|preventDefault`',
    doc: 'Calls `e.preventDefault()` before the handler runs. Common on `<form on:submit>` to take over submission from the browser.',
  },
  prevent: { signature: '`|prevent`', doc: 'Shorthand for `|preventDefault`.' },
  stopPropagation: {
    signature: '`|stopPropagation`',
    doc: 'Calls `e.stopPropagation()` before the handler runs.',
  },
  stop: { signature: '`|stop`', doc: 'Shorthand for `|stopPropagation`.' },
  stopImmediatePropagation: {
    signature: '`|stopImmediatePropagation`',
    doc: 'Calls `e.stopImmediatePropagation()` — also skips remaining listeners on this element.',
  },
  self: {
    signature: '`|self`',
    doc: 'Only fires when `e.target === e.currentTarget`. Useful for "click outside content" patterns.',
  },
  trusted: {
    signature: '`|trusted`',
    doc: 'Only fires when `e.isTrusted` — real user input, not programmatic dispatch.',
  },
  key: {
    signature: '`|key:<KeyName>`',
    doc: 'Filter `keydown`/`keyup`/`keypress` to a specific `e.key` value. Examples: `key:Enter`, `key:Escape`, `key:ArrowUp`.',
  },
  debounce: {
    signature: '`|debounce:<ms>`',
    doc: 'Trailing-edge debounce. Suppresses calls within the window; fires once after silence.',
  },
  throttle: {
    signature: '`|throttle:<ms>`',
    doc: 'Leading-edge throttle. Fires immediately, then ignores subsequent calls within the window.',
  },
}

const BLOCKS: Record<string, Entry> = {
  'if': {
    signature: '`{#if expr}…{:else if expr}…{:else}…{/if}`',
    doc: 'Conditional rendering. Branches mount/unmount as the expression flips. Effects + refs in the inactive branch are torn down.',
    see: 'docs/ship-sail/references/tutorial/05-bind-if/',
  },
  'for': {
    signature: '`{#for item of items() key={item.id}}…{/for}`',
    doc: 'Keyed list reconciliation. The `key` expression tracks identity across reorders. Per-field reactivity means each row only re-renders when *its* fields change.',
    see: 'docs/ship-sail/references/tutorial/03-bindfor/',
  },
  'while-loading': {
    signature: '`{#while-loading of <liveSignal>}…{/while-loading}`',
    doc: 'Mounts the body while the live signal is in `kind: "loading"` state. Boundary primitive — unmounts when the envelope transitions away from loading.',
    see: 'docs/ship-sail/references/tutorial/23-while-loading/',
  },
  'while-error': {
    signature: '`{#while-error err of <liveSignal>}…{/while-error}`',
    doc: 'Mounts while the live signal is in `kind: "error"` state. Inside the body, `err` is the unwrapped `{ message }` value.',
  },
  else: { signature: '`{:else}`', doc: 'Else branch of an enclosing `{#if}` block.' },
  'else if': { signature: '`{:else if expr}`', doc: 'Else-if branch of an enclosing `{#if}` block.' },
}

const TAG_ATTR_RE = /([\w:-]+)(?=\s*=|\s|$)/g

/**
 * Resolve a hover query at `offset` against the source text. Returns
 * static doc if the cursor is over a recognized directive, modifier,
 * or block keyword; null otherwise.
 */
export function hoverAt(source: string, offset: number): SuiHoverInfo | null {
  if (offset < 0 || offset > source.length) return null

  // Walk back over identifier chars only (and `:` so `on:click` and
  // `key:Enter` survive as one token). Sigils like `|` or `{#` stop
  // the walk so we can check them as the *preceding* context below.
  let start = offset
  while (start > 0 && /[\w:-]/.test(source[start - 1]!)) start--
  let end = offset
  while (end < source.length && /[\w:-]/.test(source[end]!)) end++

  const token = source.slice(start, end)
  if (!token) return null

  // 1. Directive prefix (on:, bind:, prop:, class:, style:). The
  //    token here will already include the trailing `:` (`on:`,
  //    `on:click`, etc.) so prefix-match is enough.
  for (const prefix of Object.keys(DIRECTIVES)) {
    if (token === prefix || token.startsWith(prefix)) {
      return renderEntry(DIRECTIVES[prefix]!)
    }
  }

  // 2. Modifier — preceded by `|`. e.g. `on:submit|preventDefault`,
  //    cursor in `preventDefault`, source[start-1] === '|'.
  if (start > 0 && source[start - 1] === '|') {
    if (MODIFIERS[token]) return renderEntry(MODIFIERS[token]!)
    // `key:Enter` → look up base `key`; same for debounce/throttle.
    const base = token.includes(':') ? token.slice(0, token.indexOf(':')) : token
    if (MODIFIERS[base]) return renderEntry(MODIFIERS[base]!)
  }

  // 3. Block keyword — preceded by `{#` or `{:`.
  //    `{#while-loading}` → cursor in `while-loading`,
  //    source[start-2..start] === '{#'.
  if (start >= 2) {
    const sigil = source.slice(start - 2, start)
    if (sigil === '{#' || sigil === '{:') {
      if (BLOCKS[token]) return renderEntry(BLOCKS[token]!)
    }
  }

  return null
}

function renderEntry(entry: Entry): SuiHoverInfo {
  let contents = `${entry.signature}\n\n${entry.doc}`
  if (entry.see) contents += `\n\n**See also**: \`${entry.see}\``
  return { contents }
}
