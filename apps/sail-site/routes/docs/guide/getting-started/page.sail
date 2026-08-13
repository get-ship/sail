<!--
  /docs/guide/getting-started — install placeholder until public binary/npm story ships.
  Authoring content lives in language + Tutorial, not here.
-->
<scope>
  style page = { box: size({ max-w: breakpoint-tablet }).space({ mx: auto }) }
  style eyebrow = { box: space({ mb: 2 }), paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style card = { box: space({ pad: 5 }), paint: border({ color: border, width: 1 }).fill({ color: surface-1 }) }
  style muted = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style footNav = { box: space({ pad-t: 4, mt: 4 }), paint: border({ color: border, width: 1, edge: t }) }
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
