<column style({ size({ min-h: screen }).bg(background) })>
  <navbar placement="top" sticky bordered>
    <span slot="brand" style({ fg(foreground).type({ size: lg, weight: semibold }) })>
      Dashboard
    </span>
  </navbar>

  <row nowrap align="stretch" style({ grow(1) })>
    <column gap="1" style({ shrink(0).size({ w: 56 }).pad({ x: 4, y: 5 }).bg(sidebar).fg(sidebar-foreground).border({ color: sidebar-border, edge: e }) })>
      <a
        href="#home" style({ pad({ x: 3, y: 2 }).fg(sidebar-foreground).radius(md), when: { hover: bg(sidebar-accent).fg(sidebar-accent-foreground) } })>
        Home
      </a>
      <a
        href="#reports" style({ pad({ x: 3, y: 2 }).fg(sidebar-foreground).radius(md), when: { hover: bg(sidebar-accent).fg(sidebar-accent-foreground) } })>
        Reports
      </a>
      <a
        href="#team" style({ pad({ x: 3, y: 2 }).fg(sidebar-foreground).radius(md), when: { hover: bg(sidebar-accent).fg(sidebar-accent-foreground) } })>
        Team
      </a>
    </column>

    <column gap="6" style({ grow(1).size({ min-w: 0 }).pad(6) })>
      <span style({ fg(foreground).type({ size: 2xl, weight: semibold }) })>
        Welcome back
      </span>

      <card>
        <span slot="header" style({ type({ weight: semibold }) })>
          Summary
        </span>
        <column gap="2">
          <span style({ fg(muted-foreground) })>
            Here is a quick overview of your workspace activity.
          </span>
          <span style({ fg(muted-foreground).type({ size: sm }) })>
            Check Reports for the latest metrics, or visit Team to manage members.
          </span>
        </column>
      </card>
    </column>
  </row>
</column>
