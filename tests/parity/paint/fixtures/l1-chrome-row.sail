<!-- Load-bearing L1 chrome row (skins/tokens/radius/borders). Static only. -->
<column gap="4" style({ size({ w: full }).pad(4).bg(background).fg(foreground) })>
  <row gap="3" align="center">
    <button variant="solid" tone="primary">Primary</button>
    <button>Default</button>
    <badge>Badge</badge>
    <switch />
  </row>
  <field style({ size({ w: full }) })>
    <label slot="label">Name</label>
    <input type="text" placeholder="Type here" style({ size({ w: full }) })>
  </field>
  <card style({ size({ w: full }) })>
    <span slot="header" style({ type({ weight: semibold }) })>Card header</span>
    <span style({ fg(muted-foreground) })>Card body with chrome.</span>
  </card>
</column>
