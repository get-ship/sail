<!--
  Agent probe s003 (settings). From sail-probe runs/20260731-135831-s003.
  Sanitization: `variant="solid"` on the primary button so solid/primary CVA
  paints. Bare `tone="primary"` (agent raw) uses the default variant chrome —
  that matches browser CVA, not a paint miss. Keep this solid so goldens pin
  the anti-steering path (L1 skins, not raw sx fills).
-->
<column gap="6">
  style({ box: maxW(panel-max-w-md).pad(6) })

  <h1>
    style({ paint: fg(foreground), text: fontSize(2xl).fontWeight(semibold) })
Settings</h1>

  <card>
    style({ box: w(full) })

    <column gap="4">
      style({ box: w(full) })

      <h2>
        style({ paint: fg(foreground), text: fontSize(lg).fontWeight(semibold) })
Profile</h2>
      <field>
        style({ box: w(full) })

        <label slot="label">Display name</label>
        <input type="text" placeholder="Your display name">
          style({ box: w(full) })
        </input>
      </field>
    </column>
  </card>

  <card>
    style({ box: w(full) })

    <column gap="4">
      style({ box: w(full) })

      <h2>
        style({ paint: fg(foreground), text: fontSize(lg).fontWeight(semibold) })
Notifications</h2>
      <row gap="3" align="center">
        <switch />
        <label>Email alerts</label>
      </row>
    </column>
  </card>

  <row justify="end">
    style({ box: w(full) })

    <button variant="solid" tone="primary">Save changes</button>
  </row>
</column>
