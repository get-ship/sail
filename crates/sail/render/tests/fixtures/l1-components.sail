<!-- Ten load-bearing L1 surfaces for paint goldens (RENDER-1). -->
<column gap="4" w="full">
  style.pad(4).bg(background).fg(foreground)

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
      style.fontWeight(semibold)
Card header</span>
    <span>
      style.fg(muted-foreground)
Card body with chrome.</span>
  </card>

  <navbar placement="top" bordered>
    <span slot="brand">
      style.fontWeight(semibold).fg(navbar-foreground)
Nav</span>
  </navbar>

  <row grow nowrap align="stretch" w="full">
    <column
      shrink="0"
      w="40"
      gap="1"

    >
      style.bg(sidebar).fg(sidebar-foreground).pad(3).borderE(sidebar-border)

      <span>Side</span>
    </column>
    <column grow gap="2" min-w="0">
      style.pad(3)

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
