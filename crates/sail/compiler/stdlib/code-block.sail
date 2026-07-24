<!--
  <code-block> — syntax-highlighted source.
  pre sx (leading-code + tab-2); .tok-* via compiler descendant map → skins.

  Paint: bg/fg from component.code-block tokens (cssNames code-block,
  code-block-foreground).
-->
<scope>
  props: {
    code = ''
    lang = 'sail'
  }
</scope>
<div use:syntaxHighlight sx="block">
  <pre sx="m: 0 font-family: mono font-size: sm leading: code tab: 2 bg: code-block color: code-block-foreground border border: border rounded: lg py: 4 px: 5 overflow-x: auto whitespace: pre"></pre>
</div>
