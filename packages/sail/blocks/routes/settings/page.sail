<!--
  settings — form defaults must hold.
  Host props only. style() on L1 is a finding.
-->
<scope>
  style shell = { size({ min-h: fill }).bg(background) }
  style page = { pad(6) }
  style stack = { size({ max-w: panel-max-w-md }) }
</scope>

<column gap="6" style(shell)>
  <navbar bordered>
    <span slot="brand">Northwind</span>
    <button slot="actions" size="sm">Save</button>
  </navbar>

  <column gap="6" style(page)>
    <column gap="2">
      <h1>Settings</h1>
      <p>Fieldset + field + switch. Stretch comes from the form column, not the input.</p>
    </column>

    <column gap="5" style(stack)>
      <fieldset>
        <legend>Profile</legend>
        <column gap="3">
          <field>
            <label slot="label">Display name</label>
            <input placeholder="Ada Lovelace" />
            <span slot="hint">Shown on invoices.</span>
          </field>
          <field>
            <label slot="label">Email</label>
            <input type="email" placeholder="ada@ship.dev" />
          </field>
        </column>
      </fieldset>

      <fieldset>
        <legend>Preferences</legend>
        <column gap="3">
          <row gap="3" align="center" justify="between">
            <label>Compact density</label>
            <switch />
          </row>
          <row gap="3" align="center" justify="between">
            <label>Email digest</label>
            <switch />
          </row>
        </column>
      </fieldset>

      <row gap="2">
        <button variant="ghost">Cancel</button>
        <button>Save changes</button>
      </row>
    </column>
  </column>
</column>
