<column>
  style({ box: pad(4).gap(3).w(full), paint: bg(surface-0) })

  <h1>
    style({ paint: fg(foreground), text: fontSize(2xl) })
Mid-size route</h1>
  <row>
    style({ box: gap(3).w(full) })

    <div grow>
      style({ box: pad(3), paint: bg(surface-1).radius(md) })

      <h2>
        style({ paint: fg(foreground), text: fontSize(lg) })
Card A</h2>
      <p>
        style({ paint: fg(muted-foreground) })
Static projection paint probe.</p>
    </div>
    <div grow>
      style({ box: pad(3), paint: bg(surface-2).radius(md) })

      <h2>
        style({ paint: fg(foreground), text: fontSize(lg) })
Card B</h2>
      <p>
        style({ paint: fg(muted-foreground) })
Theme tokens resolve without CSS scrape.</p>
    </div>
    <div grow>
      style({ box: pad(3), paint: bg(primary).radius(md) })

      <h2>
        style({ paint: fg(primary-foreground), text: fontSize(lg) })
Primary</h2>
      <p>
        style({ paint: fg(primary-foreground) })
Painted from registry.</p>
    </div>
  </row>
  <column>
    style({ box: pad(3).gap(2), paint: bg(surface-1).radius(sm) })

    <row>
      style({ box: gap(2) })

      <div>
        style({ box: pad(2), paint: bg(accent).radius(sm) })
A1</div>
      <div>
        style({ box: pad(2), paint: bg(muted).radius(sm) })
A2</div>
      <div>
        style({ box: pad(2), paint: bg(border).radius(sm) })
A3</div>
    </row>
    <row>
      style({ box: gap(2) })

      <div>
        style({ box: pad(2), paint: bg(surface-2).radius(sm) })
B1</div>
      <div>
        style({ box: pad(2), paint: bg(surface-3).radius(sm) })
B2</div>
      <div>
        style({ box: pad(2), paint: bg(card).radius(sm) })
B3</div>
    </row>
  </column>
  <p>
    style({ paint: fg(foreground) })
Footer line with no dynamics.</p>
</column>
