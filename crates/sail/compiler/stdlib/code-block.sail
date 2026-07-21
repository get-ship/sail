<!--
  <code-block> — syntax-highlighted source.

  Host paint: display block is sx. pre chrome is sx on <pre>.
  Contracts: pre line-height 1.6 + tab-size 2; .tok-* highlighter ink (anatomy).
-->

<scope>
  props: {
    code = ''
    lang = 'sail'
  }
</scope>

<div use:syntaxHighlight sx="block">
  <pre sx="m-0 font-mono text-sm bg-surface-1 text-foreground border border-border rounded-lg py-4 px-5 overflow-x-auto whitespace-pre"></pre>
</div>
