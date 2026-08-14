<!-- Ten load-bearing L1 surfaces for paint goldens (RENDER-1). -->
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

  <navbar placement="top" bordered>
    <span slot="brand" style({ fg(foreground).type({ weight: semibold }) })>Nav</span>
  </navbar>

  <row nowrap align="stretch" style({ grow(1).size({ w: full }) })>
    <column gap="1" style({ shrink(0).size({ w: 40 }).pad(3).bg(sidebar).fg(sidebar-foreground).border({ color: sidebar-border, edge: e }) })>
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
