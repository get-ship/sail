<!--
  /docs — docs hub: how to navigate the section tree.
-->

<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style intro = { box: maxW(breakpoint-tablet) }
  style eyebrow = { paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style section = { box: mt(2) }
  style cardGrid = { box: w(full) }
  style card = { box: pad(4).h(full), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) }
  style cardTitle = { text: fontSize(md).fontWeight(semibold) }
  style cardBody = { paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) }
  style ctaRow = { box: mt(4).padT(4), paint: borderTWidth(1).borderT(border) }
</scope>

<column gap="6" style(page)>
  <column gap="3" style(intro)>
    <p style(eyebrow)>Documentation</p>
    <h1>Sail docs</h1>
    <p style(lead)>
      How-tos, language concepts, app shell (Deck), AI authoring surfaces, and
      the API reference — one rail, organized by doc kind.
    </p>
  </column>

  <column gap="3" style(section)>
    <h2>Sections</h2>
    <grid cols="2" gap="3" style(cardGrid)>
      <column gap="2" style(card)>
        <a href="/docs/guide/introduction" style(cardTitle)>Guide</a>
        <p style(cardBody)>Installation, using components, forms, themes, and practical how-tos.</p>
      </column>
      <column gap="2" style(card)>
        <a href="/docs/concepts/sail-file" style(cardTitle)>Concepts</a>
        <p style(cardBody)>State, templates, styling, layout, animation, data, and the language model.</p>
      </column>
      <column gap="2" style(card)>
        <a href="/docs/app" style(cardTitle)>App</a>
        <p style(cardBody)>Routing, layouts, rendering, live data, HMR, and build — the Deck app shell.</p>
      </column>
      <column gap="2" style(card)>
        <a href="/docs/ai" style(cardTitle)>AI</a>
        <p style(cardBody)>Authoring loops, tooling, and eval surfaces for agent-written Sail.</p>
      </column>
      <column gap="2" style(card)>
        <a href="/docs/reference" style(cardTitle)>Reference</a>
        <p style(cardBody)>API surface: style, template, regions, runtime, config, and more.</p>
      </column>
      <column gap="2" style(card)>
        <a href="/components" style(cardTitle)>Components</a>
        <p style(cardBody)>Interactive catalog of interface patterns — full-bleed, outside the docs rail.</p>
      </column>
    </grid>
  </column>

  <row gap="2" style(ctaRow)>
    <a variant="solid" tone="primary" href="/docs/guide/getting-started">Installation →</a>
    <a variant="default" href="/playground">Playground</a>
    <a variant="default" href="/docs/guide/introduction">Introduction</a>
  </row>
</column>
