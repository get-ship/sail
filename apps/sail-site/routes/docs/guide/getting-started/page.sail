<!--
  /docs/guide/getting-started — install placeholder until public binary/npm story ships.
  Authoring content lives in language + Tutorial, not here.
-->
<scope>
  style page = { size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style eyebrow = { margin({ b: 2 }).fg(accent).type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style card = { pad(5).border({ color: border, width: 1 }).bg(surface-1) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
  style footNav = { pad({ t: 4 }).margin({ t: 4 }).border({ color: border, width: 1, edge: t }) }
</scope>

<column gap="4" style(page)>
  <p style(eyebrow)>Start here</p>
  <h1>Installation</h1>

  <card style(card)>
    <column gap="2">
      <row gap="2" align="center">
        <badge tone="warning">Under construction</badge>
      </row>
      <p>
        This page ships with the public release. Sail installs as a single
        downloadable binary, with an npm wrapper for package-manager
        workflows — the packages and this page land together.
      </p>
    </column>
  </card>

  <p style(muted)>
    In the meantime, the <a href="/playground">Playground</a> runs entirely in
    your browser — no install required.
  </p>

  <row gap="2" style(footNav)>
    <a variant="solid" tone="primary" href="/playground">Playground →</a>
    <a variant="default" href="/docs/guide/introduction">Introduction</a>
    <a variant="default" href="/docs/guide/why">Why Sail</a>
  </row>
</column>
