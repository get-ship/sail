<!--
  settings — rail + tabbed form. Fields live in a readable column, not the viewport.
-->
<scope>
  style shell = { size({ min-h: fill }).bg(background) }
  style main = { size({ min-w: 0 }).pad(6) }
  style form = { size({ max-w: panel-max-w-md }) }
  style kicker = { fg(muted-foreground).type({ size: sm }) }
  style danger = { fg(destructive).type({ size: sm }) }
</scope>

<row gap="0" align="stretch" style(shell)>
  <navbar placement="rail" bordered sticky>
    <strong slot="brand">Northwind</strong>
    <column slot="center" gap="1">
      <a href="/dashboard">Overview</a>
      <button variant="ghost" size="sm">Profile</button>
      <button variant="ghost" size="sm">Team</button>
      <button variant="ghost" size="sm">Billing</button>
    </column>
    <column slot="actions" gap="3" align="center">
      <theme-picker></theme-picker>
      <avatar initials="AL" presence="online"></avatar>
    </column>
  </navbar>

  <column gap="6" style(main)>
    <column gap="2">
      <breadcrumb>
        <a href="/dashboard">Northwind</a>
        <small>Settings</small>
      </breadcrumb>
      <h1>Settings</h1>
      <p style(kicker)>Account and preferences. Stretch is the form column, not the page.</p>
    </column>

    <tabs active="profile">
      <tab-list variant="line">
        <tab name="profile">Profile</tab>
        <tab name="team">Team</tab>
        <tab name="billing">Billing</tab>
      </tab-list>

      <tab-panel name="profile">
        <column gap="5" style(form)>
          <fieldset>
            <legend>Profile</legend>
            <column gap="4">
              <row gap="4" align="center">
                <avatar initials="AL" size="lg" ring></avatar>
                <field>
                  <label slot="label">Photo</label>
                  <file accept="image/*" />
                  <span slot="hint">Square, at least 256px.</span>
                </field>
              </row>
              <field>
                <label slot="label">Display name</label>
                <input surface="filled" placeholder="Ada Lovelace" />
                <span slot="hint">Shown on invoices and the dashboard.</span>
              </field>
              <field>
                <label slot="label">Email</label>
                <input type="email" surface="filled" placeholder="ada@ship.dev" />
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
              <label>
                <checkbox />
                Send a copy to the ops inbox
              </label>
            </column>
          </fieldset>

          <row gap="2">
            <button variant="ghost">Cancel</button>
            <button>Save changes</button>
          </row>
        </column>
      </tab-panel>

      <tab-panel name="team">
        <column gap="4">
          <row justify="between" align="center">
            <h2>Team</h2>
            <button size="sm">Invite</button>
          </row>
          <card padding="none">
            <table hoverable>
              <thead>
                <tr>
                  <th>Member</th>
                  <th>Role</th>
                  <th>Status</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>
                    <row gap="2" align="center">
                      <avatar initials="AL" size="sm"></avatar>
                      <span>Ada Lovelace</span>
                    </row>
                  </td>
                  <td><chip size="sm">Owner</chip></td>
                  <td><badge tone="success" dot>Active</badge></td>
                </tr>
                <tr>
                  <td>
                    <row gap="2" align="center">
                      <avatar initials="HC" size="sm"></avatar>
                      <span>Harbor Co</span>
                    </row>
                  </td>
                  <td><chip size="sm" variant="outline">Admin</chip></td>
                  <td><badge tone="success" dot>Active</badge></td>
                </tr>
                <tr>
                  <td>
                    <row gap="2" align="center">
                      <avatar initials="NP" size="sm"></avatar>
                      <span>North Pier</span>
                    </row>
                  </td>
                  <td><chip size="sm" variant="outline">Member</chip></td>
                  <td><badge>Invited</badge></td>
                </tr>
              </tbody>
            </table>
          </card>
        </column>
      </tab-panel>

      <tab-panel name="billing">
        <column gap="5" style(form)>
          <fieldset>
            <legend>Plan</legend>
            <radio-group name="plan" value="studio">
              <radio value="hobby">Hobby — $0</radio>
              <radio value="studio">Studio — $24 / month</radio>
              <radio value="enterprise">Enterprise — talk to sales</radio>
            </radio-group>
          </fieldset>

          <separator />

          <column gap="3">
            <h2>Theme</h2>
            <theme-picker></theme-picker>
          </column>

          <separator />

          <column gap="3">
            <h2>Danger zone</h2>
            <p style(danger)>Deleting the workspace removes orders, themes, and seats.</p>
            <button variant="outline" tone="destructive">Delete workspace</button>
          </column>
        </column>
      </tab-panel>
    </tabs>
  </column>
</row>
