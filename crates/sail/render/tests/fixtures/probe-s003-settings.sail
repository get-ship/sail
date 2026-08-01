<!--
  Agent probe s003 (settings). From sail-probe runs/20260731-135831-s003.
  Sanitization: `variant="solid"` on the primary button so solid/primary CVA
  paints. Bare `tone="primary"` (agent raw) uses the default variant chrome —
  that matches browser CVA, not a paint miss. Keep this solid so goldens pin
  the anti-steering path (L1 skins, not raw sx fills).
-->
<column gap="6" sx={style.pad(6)} max-w="panel-max-w-md">
  <h1 sx={style.fontSize(2xl).fontWeight(semibold).fg(foreground)}>Settings</h1>

  <card w="full">
    <column gap="4" w="full">
      <h2 sx={style.fontSize(lg).fontWeight(semibold).fg(foreground)}>Profile</h2>
      <field w="full">
        <label slot="label">Display name</label>
        <input type="text" placeholder="Your display name" w="full" />
      </field>
    </column>
  </card>

  <card w="full">
    <column gap="4" w="full">
      <h2 sx={style.fontSize(lg).fontWeight(semibold).fg(foreground)}>Notifications</h2>
      <row gap="3" align="center">
        <switch />
        <label>Email alerts</label>
      </row>
    </column>
  </card>

  <row justify="end" w="full">
    <button variant="solid" tone="primary">Save changes</button>
  </row>
</column>
