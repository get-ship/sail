<!-- PARITY-1 paint: load-bearing L1 chrome row (skins/tokens/radius/borders). Static only. -->
<column gap="4" w="full">
  style({ box: pad(4), paint: bg(background).fg(foreground) })

  <row gap="3" align="center">
    <button variant="solid" tone="primary">Primary</button>
    <button>Default</button>
    <badge>Badge</badge>
    <switch />
  </row>
  <field w="full">
    <label slot="label">Name</label>
    <input type="text" placeholder="Type here" w="full" />
  </field>
  <card w="full">
    <span slot="header">
      style({ text: fontWeight(semibold) })
Card header</span>
    <span>
      style({ paint: fg(muted-foreground) })
Card body with chrome.</span>
  </card>
</column>
