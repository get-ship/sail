<!--
  settings — rail + tabbed form. Stretch is the leftover, not a 38ch island.
-->
<row nowrap gap="0" align="stretch">
  <app-rail current="settings"></app-rail>

  <app-main>
    <column gap="2">
      <h1>Settings</h1>
      <p>Account and preferences. Stretch is the leftover, not a postcard form.</p>
    </column>

    <tabs active="profile">
      <tab-list variant="line">
        <tab title="Profile" name="profile"></tab>
        <tab title="Team" name="team"></tab>
        <tab title="Billing" name="billing"></tab>
      </tab-list>

      <tab-panel name="profile">
        <form label="Profile" layout="fill">
        <column gap="5">
          <fieldset>
            <legend>Profile</legend>
            <column gap="4">
              <row gap="4" align="center">
                <avatar initials="AL" size="lg" ring></avatar>
                <field>
                  <label slot="label">Photo</label>
                  <file accept="image/*" />
                  <small slot="hint">Square, at least 256px.</small>
                </field>
              </row>
              <field>
                <label slot="label">Display name</label>
                <input surface="filled" placeholder="Ada Lovelace" />
                <small slot="hint">Shown on invoices and the dashboard.</small>
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
        </form>
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
                      Ada Lovelace
                    </row>
                  </td>
                  <td><chip size="sm">Owner</chip></td>
                  <td><badge tone="success" dot>Active</badge></td>
                </tr>
                <tr>
                  <td>
                    <row gap="2" align="center">
                      <avatar initials="HC" size="sm"></avatar>
                      Harbor Co
                    </row>
                  </td>
                  <td><chip size="sm" variant="outline">Admin</chip></td>
                  <td><badge tone="success" dot>Active</badge></td>
                </tr>
                <tr>
                  <td>
                    <row gap="2" align="center">
                      <avatar initials="NP" size="sm"></avatar>
                      North Pier
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
        <form label="Billing" layout="fill">
        <column gap="5">
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
            <p>Deleting the workspace removes orders, themes, and seats.</p>
            <button variant="outline" tone="destructive">Delete workspace</button>
          </column>
        </column>
        </form>
      </tab-panel>
    </tabs>
  </app-main>
</row>
