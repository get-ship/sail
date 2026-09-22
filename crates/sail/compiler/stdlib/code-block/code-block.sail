<!--
  <code-block> — syntax-highlighted source.
  pre: pad, mono, pre-wrap so long lines wrap (docs never force horizontal scroll).
  Copy chrome lives on docs wrappers (<doc-code>, sail-component-viewer).
  terminal: darker inverse chrome. line-numbers is a language gap — highlighter
  and platform/code-block.css do not wrap lines, so a gutter cannot be keyed
  without a new highlighter.

  Look: host/pre Style owns paint — no DOM.
  State: this file's <controller> (code + highlighted html).
  Listen: none — highlight is derived, not an action.
-->
<scope>
  props: {
    code = ''
    lang = 'sail'
    terminal = false
  }

  style host = {display(block).position(relative)}
  style pre = {pad({ y: 4, s: 5, e: 12 }).margin(0).overflow({ x: hidden, y: auto }).bg(surface).fg(foreground).border({ color: border, width: 1 }).radius(lg).type({ family: mono, size: sm, leading: relaxed, wrap: pre-wrap }).tab(2)}
  style terminalPre = {bg(foreground).fg(background)}
</scope>

<controller>
export default controller((init) => {
  function read(v) { return typeof v === 'function' ? v() : v }
  const html = derived(() => {
    const code = String(read(init.code) ?? '')
    if (!code) return ''
    // Hoisted highlight registry if app registered one; else escape + plain.
    const g = globalThis as unknown as Record<string, unknown>
    const fn = g['__sail_highlighter'] as ((s: string, lang: string) => string) | undefined
    if (typeof fn === 'function') {
      try { return fn(code, String(read(init.lang) ?? 'sail')) } catch {}
    }
    return String(code).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  })
  return { html }
})
</controller>

<div label="Code" style(host)>
  <pre style(pre, {match: { terminal: { true: terminalPre } }})><code>{html()}</code></pre>
</div>
