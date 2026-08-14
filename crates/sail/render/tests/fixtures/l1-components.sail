<!-- Ten load-bearing L1 surfaces for paint goldens (RENDER-1). -->
<column gap="4" style({ size({ w: full }).pad(4), bg(background).fg(foreground) })>
  <row gap="3" align="center">
    <button variant="solid" tone="primary" style({ size({ h: 8 }) })>Primary</button>
    <button style({ size({ h: 8 }) })>Default</button>
    <badge>Badge</badge>
    <switch style({ size({ w: 10, h: 6 }) }) />
  </row>

  <field style({ size({ w: full }) })>
    <label slot="label">Name</label>
    <input type="text" placeholder="Type here" style({ size({ w: 40, h: 8 }) })>
  </field>

  <card style({ size({ w: full, min-h: 28 }).pad(4) })>
    <span slot="header" style({ type({ weight: semibold }) })>Card header</span>
    <span style({ fg(muted-foreground) })>Card body with chrome.</span>
  </card>

  <navbar placement="top" bordered style({ size({ w: 40, h: 12 }) })>
    <span slot="brand" style({ fg(foreground), type({ weight: semibold }) })>Nav</span>
  </navbar>

  <row nowrap align="stretch" style({ grow(1).size({ w: full }) })>
    <column gap="1" style({ shrink(0).size({ w: 40 }).pad(3), bg(sidebar).fg(sidebar-foreground).border({ color: sidebar-border, edge: e }) })>
      <span>Side</span>
    </column>
    <column gap="2" style({ grow(1).size({ min-w: 0 }).pad(3) })>
      <span>Main</span>
    </column>
  </row>

  <dialog open>
    <span slot="header">Dialog</span>
    <span>Dialog body</span>
  </dialog>

  <table>
    <thead>
      <tr>
        <th>Col A</th>
        <th>Col B</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>1</td>
        <td>2</td>
      </tr>
    </tbody>
  </table>
</column>
