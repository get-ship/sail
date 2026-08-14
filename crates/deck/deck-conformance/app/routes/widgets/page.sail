<!-- Uses shared <demo-chip> CE (chunk-dedup partner of gallery). -->

<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style title = { fg(foreground).type({ size: 2xl, weight: bold }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
</scope>

<column gap="3" style(page)>
  <h1 style(title)>Widgets</h1>
  <p style(muted)>Using the shared demo-chip component:</p>
  <demo-chip>widget-alpha</demo-chip>
  <demo-chip>widget-beta</demo-chip>
</column>
