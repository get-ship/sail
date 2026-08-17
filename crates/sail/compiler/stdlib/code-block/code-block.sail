<!--
  <code-block> — syntax-highlighted source.
  pre: pad, mono, pre-wrap so long lines wrap (docs never force horizontal scroll).
  Copy chrome lives on docs wrappers (<doc-code>, sail-component-viewer).
  terminal: darker inverse chrome. line-numbers is a language gap — highlighter
  and platform/code-block.css do not wrap lines, so a gutter cannot be keyed
  without a new highlighter.
-->
<scope>
  props: {
    code = ''
    lang = 'sail'
    terminal = false
  }

  style host = {display(block).position(relative)}
  style pre = {pad({ y: 4, s: 5, e: 12 }).margin(0).overflow({ x: hidden, y: auto }).bg(surface).fg(foreground).border({ color: border, width: 1 }).radius(lg).type({ family: mono, size: sm, leading: code, wrap: pre-wrap }).tab(2)}
  style terminalPre = {bg(foreground).fg(background)}
</scope>
<div label="Code" use:syntaxHighlight style(host)>
  <pre style(pre, {match: { terminal: { true: terminalPre } }})></pre>
</div>
