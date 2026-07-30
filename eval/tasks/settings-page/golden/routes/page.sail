<column gap="6" sx={style.pad(8)}>
  <h1>Settings</h1>
  <card>
    <column gap="4" sx={style.pad(6)}>
      <h2>Profile</h2>
      <label for="display-name">Display name</label>
      <input id="display-name" name="display-name" placeholder="Display name" />
    </column>
  </card>
  <card>
    <column gap="4" sx={style.pad(6)}>
      <h2>Notifications</h2>
      <row gap="2" align="center">
        <input id="email-alerts" type="checkbox" name="email-alerts" />
        <label for="email-alerts">Email alerts</label>
      </row>
    </column>
  </card>
  <button type="button" tone="primary">Save changes</button>
</column>
