<!--
  <sail-fill-shell> — full-bleed workbench (no docs rail).
-->
<scope>
  style shell = { size({ w: 100%, h: 100%, min-w: 0, min-h: 0 }).overflow({ all: hidden }) }
  style mainWorkbench = { display(flex).dir(column).size({h: fill, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }) }
</scope>

<row nowrap align="stretch" gap="0" style(shell)>
  <main testid="fill-shell" style(mainWorkbench, { size(grow).size({ w: 100%, min-w: 0 }) })>
    <slot name="route"></slot>
  </main>
</row>
