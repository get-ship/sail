<!--
  Agent probe s003 (settings). From sail-probe runs/20260731-135831-s003.
  Sanitization: `variant="solid"` on the primary button so solid/primary CVA
  paints. Bare `tone="primary"` (agent raw) uses the default variant chrome —
  that matches browser CVA, not a paint miss. Keep this solid so goldens pin
  the anti-steering path (L1 skins, not raw sx fills).
-->
<column gap="6" style({ size({ max-w: panel-max-w-md }).pad(6) })>
  <h1 style({ fg(foreground), type({ size: 2xl, weight: semibold }) })>Settings</h1>

  <card style({ size({ w: full, min-h: 28 }).pad(4) })>
    <column gap="4" style({ size({ w: full }) })>
      <h2 style({ fg(foreground), type({ size: lg, weight: semibold }) })>Profile</h2>
      <field style({ size({ w: full }) })>
        <label slot="label">Display name</label>
        <input type="text" placeholder="Your display name" style({ size({ w: 40, h: 8 }) }) />
      </field>
    </column>
  </card>

  <card style({ size({ w: full, min-h: 24 }).pad(4) })>
    <column gap="4" style({ size({ w: full }) })>
      <h2 style({ fg(foreground), type({ size: lg, weight: semibold }) })>Notifications</h2>
      <row gap="3" align="center">
        <switch style({ size({ w: 10, h: 6 }) }) />
        <label>Email alerts</label>
      </row>
    </column>
  </card>

  <row justify="end" style({ size({ w: full }) })>
    <button variant="solid" tone="primary" style({ size({ h: 8 }) })>Save changes</button>
  </row>
</column>
