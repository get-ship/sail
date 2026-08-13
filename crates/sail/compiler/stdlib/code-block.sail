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

  style host = {box: flow({ display: block }).place({ position: relative })}
  style pre = {box: space({ m: 0, pad-y: 4, pad-s: 5, pad-e: 12 }).overflow({ x: hidden, y: auto }), paint: fill({ color: code-block }).ink({ color: code-block.foreground }).border({ color: border, width: 1 }).radius(lg), text: type({ family: mono, size: sm, leading: code, wrap: words }).tab(2)}
</scope>
<div use:syntaxHighlight style(host)>
  <pre style(pre)></pre>
</div>
