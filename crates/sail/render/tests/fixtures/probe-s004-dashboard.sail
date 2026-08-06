<column>
  style.minH(screen).bg(background)

  <navbar placement="top" sticky bordered>
    <span slot="brand">
      style.fontSize(lg).fontWeight(semibold).fg(navbar-foreground)

      Dashboard
    </span>
  </navbar>

  <row grow nowrap align="stretch">
    <column
      shrink="0"
      w="56"
      gap="1"

    >
      style.bg(sidebar).fg(sidebar-foreground).padY(5).padX(4).borderE(sidebar-border)

      <a
        href="#home"

      >
        style.fg(sidebar-foreground).padY(2).padX(3).radius(md).when({ hover: style.bg(sidebar-accent).fg(sidebar-accent-foreground) })

        Home
      </a>
      <a
        href="#reports"

      >
        style.fg(sidebar-foreground).padY(2).padX(3).radius(md).when({ hover: style.bg(sidebar-accent).fg(sidebar-accent-foreground) })

        Reports
      </a>
      <a
        href="#team"

      >
        style.fg(sidebar-foreground).padY(2).padX(3).radius(md).when({ hover: style.bg(sidebar-accent).fg(sidebar-accent-foreground) })

        Team
      </a>
    </column>

    <column grow gap="6" min-w="0">
      style.pad(6)

      <span>
        style.fontSize(2xl).fontWeight(semibold).fg(foreground)

        Welcome back
      </span>

      <card>
        <span slot="header">
          style.fontWeight(semibold)

          Summary
        </span>
        <column gap="2">
          <span>
            style.fg(muted-foreground)

            Here is a quick overview of your workspace activity.
          </span>
          <span>
            style.fontSize(sm).fg(muted-foreground)

            Check Reports for the latest metrics, or visit Team to manage members.
          </span>
        </column>
      </card>
    </column>
  </row>
</column>
