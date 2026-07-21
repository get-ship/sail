<!--
  <code-block> — syntax-highlighted source.

    <code-block code={src} lang="sail"></code-block>

  Colours are the `--syntax-*` theme tokens. Host paint: display block is sx.
  pre chrome (mono/size/surface/border/radius/pad/overflow) is sx on <pre>.
  Residue: line-height 1.6, tab-size 2, tok-* token colors (highlighter anatomy).
-->

<scope>
  props: {
    code = ''
    lang = 'sail'
  }
</scope>

<div use:codeBlock sx="block">
  <pre sx="m-0 font-mono text-sm bg-surface-1 text-foreground border border-border rounded-lg py-4 px-5 overflow-x-auto whitespace-pre"></pre>
</div>
