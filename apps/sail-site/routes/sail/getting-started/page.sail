<!--
  /getting-started — UNDER CONSTRUCTION placeholder, deliberately.
  RULING (owner, 2026-07-18): the site publishes WHEN the packages publish —
  there is no audience for a source-build walkthrough, and writing interim
  content poisons future context. This page stays a marked placeholder until
  the install story (#117: binary + npm wrapper) is real, then it becomes the
  actual one-command install. Authoring content lives in the language section
  and the Tutorial — never inline here.
-->
<column gap="4">
  style.maxW(breakpoint-tablet).mx(auto)

  <p>
    style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)
Start here</p>
  <h1>Installation</h1>

  <card>
    style.pad(5).borderWidth(1).border(border).bg(surface-1)

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

  <p>
    style.fontSize(sm).fg(muted-foreground)

    In the meantime, the <a href="/sail/tutorial">Tutorial</a> runs entirely in
    your browser — no install required.
  </p>

  <row gap="2">
    style.mt(4).padT(4).borderTWidth(1).borderT(border)

    <a variant="solid" tone="primary" href="/sail/tutorial">Tutorial →</a>
    <a variant="default" href="/sail/overview">Introduction</a>
    <a variant="default" href="/sail/why">Why Sail</a>
  </row>
</column>
