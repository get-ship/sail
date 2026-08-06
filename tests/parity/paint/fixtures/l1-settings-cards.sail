<!-- PARITY-1 paint: stacked cards + solid primary button (RENDER-1 s003 path). Static only. -->
<column gap="6" max-w="panel-max-w-md">
  style({ box: pad(6) })

  <h1>
    style({ paint: fg(foreground), text: fontSize(2xl).fontWeight(semibold) })
Settings</h1>

  <card w="full">
    <column gap="4" w="full">
      <h2>
        style({ paint: fg(foreground), text: fontSize(lg).fontWeight(semibold) })
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
        style({ paint: fg(foreground), text: fontSize(lg).fontWeight(semibold) })
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
