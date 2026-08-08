<!-- Uses shared <demo-chip> CE (chunk-dedup partner of gallery). -->

<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style title = { paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) }
  style muted = { paint: fg(muted-foreground), text: fontSize(sm) }
</scope>

<column gap="3" style(page)>
  <h1 style(title)>Widgets</h1>
  <p style(muted)>Using the shared demo-chip component:</p>
  <demo-chip>widget-alpha</demo-chip>
  <demo-chip>widget-beta</demo-chip>
</column>
