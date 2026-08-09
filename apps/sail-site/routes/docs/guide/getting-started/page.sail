<!--
  /docs/guide/getting-started — install placeholder until public binary/npm story ships.
  Authoring content lives in language + Tutorial, not here.
-->
<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style card = { box: pad(5), paint: borderWidth(1).border(border).bg(surface-1) }
  style muted = { paint: fg(muted-foreground), text: fontSize(sm) }
  style footNav = { box: mt(4).padT(4), paint: borderTWidth(1).borderT(border) }
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
