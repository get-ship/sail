<!--
  Owned L1 rest-state strip. Theme paints tags. L1 owns skins.
  Probe finds [data-sail-host]. Add a tag here to fan the matrix out.
  Overlays (dialog, popover, tooltip) stay off this page — they need a press.
-->
<scope>
  style shell = { size({ min-h: fill }).pad(6).bg(background) }
  style measure = { size({ w: full, max-w: breakpoint-desktop }) }
</scope>

<column gap="8" style(shell)>
  <column gap="2" style(measure)>
    <row gap="3" align="center" justify="between">
      <column gap="2">
        <small>Owned L1 matrix</small>
        <h1>Rest state</h1>
      </column>
      <theme-picker />
    </row>
    <p>Not the docs site. Official themes only. Probe drives THEME_META; the picker is for eyes.</p>
  </column>

  <column gap="3">
    <h2>Intrinsic</h2>
    <p>No stretching parent — default measure.</p>
    <row gap="2" align="end">
      <button>Button</button>
      <badge>Badge</badge>
      <chip>Chip</chip>
      <input placeholder="input" />
      <kbd>⌘K</kbd>
    </row>
  </column>

  <column gap="3" style(measure)>
    <h2>Actions</h2>
    <row gap="2">
      <button>Solid</button>
      <button variant="soft">Soft</button>
      <button variant="outline">Outline</button>
      <button variant="ghost">Ghost</button>
      <button tone="destructive">Destroy</button>
    </row>
    <row gap="2">
      <badge>rest</badge>
      <badge tone="primary">primary</badge>
      <chip>chip</chip>
    </row>
  </column>

  <column gap="3" style(measure)>
    <h2>Fields</h2>
    <field>
      <label slot="label">Name</label>
      <input placeholder="token, not hex" />
      <span slot="hint">Field + input.</span>
    </field>
    <file accept="image/*" />
    <textarea placeholder="notes"></textarea>
    <row gap="4" align="center">
      <checkbox />
      <switch />
    </row>
  </column>

  <card>
    <h2 slot="header">Card</h2>
    <p>Surface, radius, elevation on the element.</p>
    <row slot="footer" gap="2">
      <button variant="ghost" size="sm">Cancel</button>
      <button size="sm">Save</button>
    </row>
  </card>

  <alert tone="info">
    <span slot="title">Alert</span>
    Theme tokens, not a painted div.
  </alert>

  <row gap="3" align="center" style(measure)>
    <avatar />
    <spinner />
    <skeleton />
    <progress value="40" />
    <kbd>⌘K</kbd>
    <separator />
  </row>
</column>
