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

<style>
  .sail-code-block pre {
    line-height: 1.6; /* residue — no exact 1.6 leading token (normal=1.5; relaxed=1.7) */
    tab-size: 2; /* residue literal — no tab-size token */
  }
  .sail-code-block .tok-tag { color: var(--syntax-tag); }
  .sail-code-block .tok-attr { color: var(--syntax-attr); }
  .sail-code-block .tok-string { color: var(--syntax-string); }
  .sail-code-block .tok-comment { color: var(--syntax-comment); font-style: italic; }
  .sail-code-block .tok-keyword { color: var(--syntax-keyword); }
  .sail-code-block .tok-fn { color: var(--syntax-function); }
  .sail-code-block .tok-number { color: var(--syntax-number); }
  .sail-code-block .tok-bind { color: var(--syntax-function); }
</style>

<div use:codeBlock sx="block">
  <pre sx="m-0 font-mono text-sm bg-surface-1 text-foreground border border-border rounded-lg py-4 px-5 overflow-x-auto whitespace-pre"></pre>
</div>
