<!--
  settings — rail + form. Fields live in a readable column, not the viewport.
-->
<scope>
  style shell = { size({ min-h: fill }).bg(background) }
  style main = { size({ min-w: 0 }).pad(6) }
  style form = { size({ max-w: panel-max-w-md }) }
</scope>

<row gap="0" align="stretch" style(shell)>
  <navbar placement="rail" bordered sticky>
    <strong slot="brand">Northwind</strong>
    <column slot="center" gap="1">
      <button variant="ghost" size="sm">Profile</button>
      <button variant="ghost" size="sm">Team</button>
      <button variant="ghost" size="sm">Billing</button>
    </column>
  </navbar>

  <column gap="6" style(main)>
    <column gap="1">
      <h1>Settings</h1>
      <p>Account and preferences. Stretch is the form column, not the page.</p>
    </column>

    <column gap="5" style(form)>
      <fieldset>
        <legend>Profile</legend>
        <column gap="3">
          <field>
            <label slot="label">Display name</label>
            <input placeholder="Ada Lovelace" />
            <span slot="hint">Shown on invoices and the dashboard.</span>
          </field>
          <field>
            <label slot="label">Email</label>
            <input type="email" placeholder="ada@ship.dev" />
          </field>
          <field>
            <label slot="label">Role</label>
            <select>
              <option>Owner</option>
              <option>Admin</option>
              <option>Member</option>
            </select>
          </field>
          <field>
            <label slot="label">Bio</label>
            <textarea placeholder="What you ship"></textarea>
          </field>
        </column>
      </fieldset>

      <fieldset>
        <legend>Notifications</legend>
        <column gap="3">
          <row gap="3" align="center" justify="between">
            <label>Email digest</label>
            <switch />
          </row>
          <row gap="3" align="center" justify="between">
            <label>Order slips</label>
            <switch />
          </row>
          <radio-group name="digest">
            <radio value="daily">Daily</radio>
            <radio value="weekly">Weekly</radio>
            <radio value="off">Off</radio>
          </radio-group>
        </column>
      </fieldset>

      <row gap="2">
        <button variant="ghost">Cancel</button>
        <button>Save changes</button>
      </row>
    </column>
  </column>
</row>
