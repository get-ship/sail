<scope>
  style page = { pad(6).margin({ x: auto }).size({ max-w: breakpoint-tablet }) }
  style section = { pad(4).margin({ y: 4 }).border({ color: border, width: 1 }).radius(lg) }
  style sectionTitle = { margin({ b: 2 }).fg(muted-foreground).type({ size: sm, weight: semibold }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
</scope>

<column gap="4" style(page)>
  <column gap="1">
    <h1>Sail Demo</h1>
    <p style(muted)>Visual sanity-check for the Sail component bundle.</p>
  </column>

  <column gap="2" style(section)>
    <h2 style(sectionTitle)>button — solid</h2>
    <row gap="2" align="center">
      <button variant="solid">Default</button>
      <button variant="solid" tone="primary">Primary</button>
      <button variant="solid" tone="success">Success</button>
      <button variant="solid" tone="warning">Warning</button>
      <button variant="solid" tone="destructive">Destructive</button>
      <button variant="solid" tone="info">Info</button>
    </row>
  </column>

  <column gap="2" style(section)>
    <h2 style(sectionTitle)>button — soft</h2>
    <row gap="2" align="center">
      <button variant="soft">Default</button>
      <button variant="soft" tone="primary">Primary</button>
      <button variant="soft" tone="success">Success</button>
      <button variant="soft" tone="destructive">Destructive</button>
    </row>
  </column>

  <column gap="2" style(section)>
    <h2 style(sectionTitle)>button — outline</h2>
    <row gap="2" align="center">
      <button variant="outline" tone="primary">Primary</button>
      <button variant="outline" tone="success">Success</button>
      <button variant="outline" tone="destructive">Destructive</button>
    </row>
  </column>

  <column gap="2" style(section)>
    <h2 style(sectionTitle)>button — ghost</h2>
    <row gap="2" align="center">
      <button variant="ghost" tone="primary">Primary</button>
      <button variant="ghost" tone="destructive">Destructive</button>
    </row>
  </column>

  <column gap="2" style(section)>
    <h2 style(sectionTitle)>button — sizes</h2>
    <row gap="2" align="center">
      <button variant="solid" tone="primary" size="sm">Small</button>
      <button variant="solid" tone="primary" size="md">Medium</button>
      <button variant="solid" tone="primary" size="lg">Large</button>
    </row>
  </column>

  <column gap="2" style(section)>
    <h2 style(sectionTitle)>button — states</h2>
    <row gap="2" align="center">
      <button variant="solid" tone="primary" disabled>Disabled</button>
      <button variant="solid" tone="primary" disabled aria-busy="true">
        <spinner size="sm"></spinner>
        Loading
      </button>
    </row>
  </column>

  <column gap="2" style(section)>
    <h2 style(sectionTitle)>badge</h2>
    <row gap="2" align="center">
      <badge variant="soft">default</badge>
      <badge tone="primary" variant="solid">primary</badge>
      <badge tone="success" variant="solid">success</badge>
      <badge tone="warning" variant="solid">warning solid</badge>
      <badge tone="info" variant="outline">info outline</badge>
      <badge tone="destructive" variant="solid">destructive</badge>
    </row>
  </column>

  <column gap="2" style(section)>
    <h2 style(sectionTitle)>card</h2>
    <card>
      <h3 slot="header">Card title</h3>
      <p>Card body content lives in the default slot.</p>
      <row slot="footer" gap="2">
        <button variant="solid" tone="primary">Action</button>
      </row>
    </card>
  </column>
</column>
