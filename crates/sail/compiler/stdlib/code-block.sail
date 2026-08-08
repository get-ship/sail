<!--
  <code-block> — syntax-highlighted source.
  pre Style (leading-code + tab-2); .tok-* via compiler descendant map → skins.

  Paint: bg/fg from component.code-block tokens (cssNames code-block,
  code-block.foreground).

  Element mode forbids use:action={param} (static splice). The action node is
  this structure root; props land on the stamped CE host — syntaxHighlight
  resolves host `.code` / `code` attr (and observes attribute changes).
-->
<scope>
  props: {
    code = ''
    lang = 'sail'
  }

  style host = { box: display(block) }
  style pre = { box: m(0).padY(4).padX(5).overflowX(auto), paint: bg(code-block).fg(code-block.foreground).borderWidth(1).border(border).radius(lg), text: fontFamily(mono).fontSize(sm).leading(code).tab(2).whitespace(pre) }
</scope>
<div use:syntaxHighlight style(host)>
  <pre style(pre)></pre>
</div>
