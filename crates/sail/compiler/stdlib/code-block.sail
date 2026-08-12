<!--
  <code-block> — syntax-highlighted source.
  pre: pad, mono, pre-wrap so long lines wrap (docs never force horizontal scroll).
  Copy chrome lives on docs wrappers (<doc-code>, sail-component-viewer).
-->
<scope>
  props: {
    code = ''
    lang = 'sail'
  }

  style host = { box: display(block).position(relative) }
  style pre = {
    box: m(0).padY(4).padS(5).padE(12).overflowX(hidden).overflowY(auto),
    paint: bg(code-block).fg(code-block.foreground).borderWidth(1).border(border).radius(lg),
    text: fontFamily(mono).fontSize(sm).leading(code).tab(2).whitespace(pre-wrap).break(words),
  }
</scope>
<div use:syntaxHighlight style(host)>
  <pre style(pre)></pre>
</div>
