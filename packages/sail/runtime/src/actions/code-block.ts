import type { Action } from '../action'
import { highlight } from './highlight'

/**
 * `use:codeBlock` — the `<code-block>` element's behavior.
 *
 * There were FOUR syntax highlighters: the docs site hand-rolled one with its own
 * token spans and its own CSS, <sail-markdown> had a second, <sail-diff> a third,
 * and <sail-code-editor> ships CodeMirror. Four token vocabularies drifting apart
 * in a project whose whole argument is that one team's library must not meet
 * another team's. This is the one.
 *
 * The source is read from the `code` attribute (or the element's own text), and
 * the highlighter escapes everything and only ever emits its own `tok-*` spans —
 * so `innerHTML` here cannot inject. Colours come from the `--syntax-*` theme
 * tokens, which means the code block re-themes with everything else.
 */
export const codeBlock: Action<void> = (node) => {
  const el = node as HTMLElement
  const pre = el.querySelector('pre') ?? el

  const render = (): void => {
    const src = el.getAttribute('code') ?? el.textContent ?? ''
    pre.innerHTML = highlight(src)
  }

  render()

  // The `code` attribute is often bound, so it changes after mount.
  const mo = new MutationObserver(render)
  mo.observe(el, { attributes: true, attributeFilter: ['code'] })

  return { destroy: () => mo.disconnect() }
}
