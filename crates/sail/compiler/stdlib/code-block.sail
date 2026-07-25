<!--
  <code-block> — syntax-highlighted source.
  pre Style (leading-code + tab-2); .tok-* via compiler descendant map → skins.

  Paint: bg/fg from component.code-block tokens (cssNames code-block,
  code-block-foreground).
-->
<scope>
  props: {
    code = ''
    lang = 'sail'
  }

  const host = style.display(block)
  const pre = style.m(0).fontFamily(mono).fontSize(sm).leading(code).tab(2).bg(code-block).fg(code-block-foreground).borderWidth(1).border(border).radius(lg).padY(4).padX(5).overflowX(auto).whitespace(pre)
</scope>
<div use:syntaxHighlight sx={host}>
  <pre sx={pre}></pre>
</div>
