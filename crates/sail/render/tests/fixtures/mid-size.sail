<column gap="3" style({ pad(4).size({ w: 100% }).bg(surface) })>
  <h1 style({ fg(foreground).type({ size: 2xl }) })>Mid-size route</h1>
  <row gap="3" style({ size({ w: 100% }) })>
    <div style({ grow(1).pad(3).bg(surface).radius(md) })>
      <h2 style({ fg(foreground).type({ size: lg }) })>Card A</h2>
      <p style({ fg(muted-foreground) })>Static projection paint probe.</p>
    </div>
    <div style({ grow(1).pad(3).bg(surface).radius(md) })>
      <h2 style({ fg(foreground).type({ size: lg }) })>Card B</h2>
      <p style({ fg(muted-foreground) })>Theme tokens resolve without CSS scrape.</p>
    </div>
    <div style({ grow(1).pad(3).bg(primary).radius(md) })>
      <h2 style({ fg(primary-foreground).type({ size: lg }) })>Primary</h2>
      <p style({ fg(primary-foreground) })>Painted from registry.</p>
    </div>
  </row>
  <column gap="2" style({ pad(3).bg(surface).radius(sm) })>
    <row gap="2">
      <div style({ pad(2).bg(accent).radius(sm) })>A1</div>
      <div style({ pad(2).bg(muted).radius(sm) })>A2</div>
      <div style({ pad(2).bg(border).radius(sm) })>A3</div>
    </row>
    <row gap="2">
      <div style({ pad(2).bg(surface).radius(sm) })>B1</div>
      <div style({ pad(2).bg(surface).radius(sm) })>B2</div>
      <div style({ pad(2).bg(surface).radius(sm) })>B3</div>
    </row>
  </column>
  <p style({ fg(foreground) })>Footer line with no dynamics.</p>
</column>
