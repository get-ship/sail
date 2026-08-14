<!--
  pricing — three tiers, default card measure, no per-card style().
-->
<scope>
  style shell = { size({ min-h: fill }).pad(6).bg(background) }
</scope>

<column gap="6" style(shell)>
  <column gap="2">
    <h1>Pricing</h1>
    <p>Default cards. If these look wrong, the card defaults are wrong.</p>
  </column>

  <row gap="4" align="start">
    <card>
      <small slot="header">Starter</small>
      <h2>$0</h2>
      <p>One project. Community support.</p>
      <button slot="footer" variant="outline">Start</button>
    </card>
    <card variant="elevated">
      <small slot="header">Studio</small>
      <h2>$24</h2>
      <p>Themes, blocks, and a team.</p>
      <button slot="footer">Choose Studio</button>
    </card>
    <card variant="outline">
      <small slot="header">Fleet</small>
      <h2>Talk</h2>
      <p>SSO, audit, dedicated support.</p>
      <button slot="footer" variant="ghost">Contact</button>
    </card>
  </row>
</column>
