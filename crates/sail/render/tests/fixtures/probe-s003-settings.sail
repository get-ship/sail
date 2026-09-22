<!--
  Agent probe s003 (settings). From sail-probe runs/20260731-135831-s003.
  Sanitization: `variant="solid"` on the primary button so solid/primary CVA
  paints. Bare `tone="primary"` (agent raw) uses the default variant chrome —
  that matches browser CVA, not a paint miss. Keep this solid so goldens pin
  the anti-steering path (L1 skins, not raw sx fills).
-->
<column gap="6" style({ size({ max-w: overlay-max-w-md }).pad(6) })>
  <h1 style({ fg(foreground).type({ size: 2xl, weight: semibold }) })>Settings</h1>

  <card style({ size({ w: 100% }) })>
    <column gap="4" style({ size({ w: 100% }) })>
      <h2 style({ fg(foreground).type({ size: lg, weight: semibold }) })>Profile</h2>
      <field style({ size({ w: 100% }) })>
        <label slot="label">Display name</label>
        <input type="text" placeholder="Your display name" style({ size({ w: 100% }) }) />
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
