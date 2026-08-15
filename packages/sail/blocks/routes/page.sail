<!--
  Blocks index — taste surface. Live recipes, not a marketplace.
-->
<scope>
  style shell = { size({ min-h: fill }).bg(background) }
  style page = { size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }).pad({ x: 6, y: 8 }) }
  style lead = { fg(muted-foreground).type({ size: lg, leading: relaxed }) }
  style blurb = { fg(muted-foreground).type({ size: sm, leading: relaxed }) }
</scope>

<column gap="0" style(shell)>
  <navbar bordered>
    <strong slot="brand">Blocks</strong>
    <row slot="center" gap="2">
      <a href="/dashboard">Dashboard</a>
      <a href="/pricing">Pricing</a>
      <a href="/settings">Settings</a>
    </row>
    <theme-picker slot="actions"></theme-picker>
  </navbar>

  <column gap="6" style(page)>
    <column gap="2">
      <badge>Taste</badge>
      <h1>Blocks</h1>
      <p style(lead)>Page-shaped compositions of L1 defaults. If a block needs paint on every tag, the atoms are wrong.</p>
    </column>

    <grid min="sm" gap="4">
      <a href="/dashboard">
        <card interactive>
          <small slot="header">Application</small>
          <h2>Dashboard</h2>
          <p style(blurb)>Rail, dialog, filters, table, toast. The new-order path is the form default.</p>
        </card>
      </a>
      <a href="/pricing">
        <card interactive>
          <small slot="header">Marketing</small>
          <h2>Pricing</h2>
          <p style(blurb)>Three tiers, comparison table, quotes, FAQ. The Tailwind Plus bar.</p>
        </card>
      </a>
      <a href="/settings">
        <card interactive>
          <small slot="header">Application</small>
          <h2>Settings</h2>
          <p style(blurb)>Tabbed form. Stretch stops at the field column.</p>
        </card>
      </a>
    </grid>
  </column>
</column>
