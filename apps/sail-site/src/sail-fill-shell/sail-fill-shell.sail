<!--
  <sail-fill-shell> — full-bleed fill shell for workbench / residual routes.
  Used by /playground and parked /sail/tutorial* — no docs rail.
-->
<scope>
  style shell = { size({ w: full, h: full, min-w: 0, min-h: 0 }).grow(1).overflow({ all: hidden }) }
  style mainWorkbench = { display(flex).dir(column).size({h: fill, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }) }
</scope>

<row nowrap align="stretch" gap="0" style(shell)>
  <main testid="fill-shell" style(mainWorkbench, { grow(1).size({ w: full, min-w: 0 }) })>
    <small>Structure intelligence only (parse, layout, vocabulary). TypeScript checking for controller and expressions runs in the sail CLI language server, not in this tab.</small>
    <slot name="route"></slot>
  </main>
</row>
