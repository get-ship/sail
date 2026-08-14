<!--
  /docs — docs hub: how to navigate the section tree.
-->

<scope>
  style page = { size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style intro = { size({ max-w: breakpoint-tablet }) }
  style eyebrow = { fg(accent).type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { fg(muted-foreground).type({ size: lg, leading: relaxed }) }
  style section = { margin({ t: 2 }) }
  style cardGrid = { size({ w: full }) }
  style card = { pad(4).size({ h: full }).border({ color: border, width: 1 }).radius(lg).bg(surface-1) }
  style cardTitle = { type({ size: md, weight: semibold }) }
  style cardBody = { fg(muted-foreground).type({ size: sm, leading: relaxed }) }
  style ctaRow = { pad({ t: 4 }).margin({ t: 4 }).border({ color: border, width: 1, edge: t }) }
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
