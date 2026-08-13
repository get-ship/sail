<!--
  <sail-fill-shell> — full-bleed fill shell for workbench / residual routes.
  Used by /playground, parked /sail/tutorial*, and legacy redirect prefixes
  that must not carry a docs rail.
-->
<scope>
  style shell = { box: size({ w: full, h: full, min-w: 0, min-h: 0 }).flow({ grow: 1 }).overflow({ all: hidden }) }
  style mainWorkbench = { box: flow({ display: flex, dir: column }).basis(0).size({ h: fill, min-w: 0, min-h: 0 }).overflow({ all: hidden }) }
</scope>

<row nowrap align="stretch" gap="0" style(shell)>
  <main testid="fill-shell" style(mainWorkbench, { box: flow({ grow: 1 }).size({ w: full, min-w: 0 }) })>
    <slot name="route"></slot>
  </main>
</row>
