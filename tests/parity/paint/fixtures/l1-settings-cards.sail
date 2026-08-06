<!-- PARITY-1 paint: stacked cards + solid primary button (RENDER-1 s003 path). Static only. -->
<column gap="6" max-w="panel-max-w-md">
  style.pad(6)

  <h1>
    style.fontSize(2xl).fontWeight(semibold).fg(foreground)
Settings</h1>

  <card w="full">
    <column gap="4" w="full">
      <h2>
        style.fontSize(lg).fontWeight(semibold).fg(foreground)
Profile</h2>
      <field w="full">
        <label slot="label">Display name</label>
        <input type="text" placeholder="Your display name" w="full" />
      </field>
    </column>
  </card>

  <card w="full">
    <column gap="4" w="full">
      <h2>
        style.fontSize(lg).fontWeight(semibold).fg(foreground)
Notifications</h2>
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
