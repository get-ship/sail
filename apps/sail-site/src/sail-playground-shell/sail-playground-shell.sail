<!--
  /playground workbench only. Not the shared fill shell — /ai /deck /sail
  stay structure-silent.
-->
<scope>
  style shell = { size({ w: 100%, h: 100%, min-w: 0, min-h: 0 }).overflow({ all: hidden }) }
  style mainWorkbench = { display(flex).dir(column).size({h: fill, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }) }
  style tsNote = { pad({ x: 2, y: 1 }).fg(muted-foreground).type(body-sm) }
</scope>

<row nowrap align="stretch" gap="0" style(shell)>
  <main testid="playground-shell" style(mainWorkbench, { size(grow).size({ w: 100%, min-w: 0 }) })>
    <small style(tsNote) testid="playground-ts-note">Structure intelligence only (parse, layout, vocabulary). TypeScript checking for controller and expressions runs in the sail CLI language server, not in this tab.</small>
    <slot name="route"></slot>
  </main>
</row>
