<column sx={style.bg(surface-0).pad(4).gap(3).w(full)}>
  <h1 sx={style.fg(foreground).fontSize(2xl)}>Mid-size route</h1>
  <row sx={style.gap(3).w(full)}>
    <div grow sx={style.bg(surface-1).pad(3).radius(md)}>
      <h2 sx={style.fg(foreground).fontSize(lg)}>Card A</h2>
      <p sx={style.fg(muted-foreground)}>Static projection paint probe.</p>
    </div>
    <div grow sx={style.bg(surface-2).pad(3).radius(md)}>
      <h2 sx={style.fg(foreground).fontSize(lg)}>Card B</h2>
      <p sx={style.fg(muted-foreground)}>Theme tokens resolve without CSS scrape.</p>
    </div>
    <div grow sx={style.bg(primary).pad(3).radius(md)}>
      <h2 sx={style.fg(primary-foreground).fontSize(lg)}>Primary</h2>
      <p sx={style.fg(primary-foreground)}>Painted from registry.</p>
    </div>
  </row>
  <column sx={style.bg(surface-1).pad(3).radius(sm).gap(2)}>
    <row sx={style.gap(2)}>
      <div sx={style.bg(accent).pad(2).radius(sm)}>A1</div>
      <div sx={style.bg(muted).pad(2).radius(sm)}>A2</div>
      <div sx={style.bg(border).pad(2).radius(sm)}>A3</div>
    </row>
    <row sx={style.gap(2)}>
      <div sx={style.bg(surface-2).pad(2).radius(sm)}>B1</div>
      <div sx={style.bg(surface-3).pad(2).radius(sm)}>B2</div>
      <div sx={style.bg(card).pad(2).radius(sm)}>B3</div>
    </row>
  </column>
  <p sx={style.fg(foreground)}>Footer line with no dynamics.</p>
</column>
