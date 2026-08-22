<scope>
  style shell = {size({ min-h: fill }).pad(6).bg(background)}
  style measure = {size({ max-w: 60ch, w: 100% })}
</scope>

<column gap="6" align="start" style(shell)>
  <column gap="2" style(measure)>
    <small>Owned corpus</small>
    <h1>Style lab</h1>
    <p>
      Theme paints tags. L1 owns skins. This page does not restyle
      <code>p</code> or fake a card on a <code>div</code>.
    </p>
  </column>

  <column gap="3" style(measure)>
    <h2>Controls</h2>
    <row gap="3">
      <button>Solid</button>
      <button variant="soft">Soft</button>
      <button variant="outline">Outline</button>
      <button variant="ghost">Ghost</button>
      <button tone="danger">Destroy</button>
    </row>
    <row gap="2">
      <badge>rest</badge>
      <badge tone="primary">primary</badge>
      <badge tone="success">success</badge>
      <badge tone="warning">warning</badge>
      <badge tone="danger">danger</badge>
    </row>
  </column>

  <card>
    <h2 slot="header">Card</h2>
    <p>Theme surface, radius, and elevation live on the element.</p>
    <row slot="footer" gap="2">
      <button variant="ghost" size="sm">Cancel</button>
      <button size="sm">Save</button>
    </row>
  </card>

  <alert tone="info">
    <span slot="title">Adapter proof</span>
    Concern programs are locked in the Rust corpus. Raw
    <code>16px</code> and hex stay compile errors.
  </alert>

  <field>
    <label slot="label">Name</label>
    <input placeholder="token, not hex" />
    <span slot="hint">Field + input from stdlib.</span>
  </field>
</column>
