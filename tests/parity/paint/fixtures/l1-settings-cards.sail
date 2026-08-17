<!-- Stacked settings cards + solid primary button. Static only. -->
<column gap="6" style({ size({ max-w: panel-max-w-md }).pad(6) })>
  <h1 style({ fg(foreground).type({ size: 2xl, weight: semibold }) })>Settings</h1>

  <card style({ size({ w: 100% }) })>
    <column gap="4" style({ size({ w: 100% }) })>
      <h2 style({ fg(foreground).type({ size: lg, weight: semibold }) })>Profile</h2>
      <field style({ size({ w: 100% }) })>
        <label slot="label">Display name</label>
        <input type="text" placeholder="Your display name" style({ size({ w: 100% }) })>
      </field>
    </column>
  </card>

  <card style({ size({ w: 100% }) })>
    <column gap="4" style({ size({ w: 100% }) })>
      <h2 style({ fg(foreground).type({ size: lg, weight: semibold }) })>Notifications</h2>
      <row gap="3" align="center">
        <switch />
        <label>Email alerts</label>
      </row>
    </column>
  </card>

  <row justify="end" style({ size({ w: 100% }) })>
    <button variant="solid" tone="primary">Save changes</button>
  </row>
</column>
