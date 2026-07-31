<column sx={style.minH(screen).bg(background)}>
  <navbar placement="top" sticky bordered>
    <span slot="brand" sx={style.fontSize(lg).fontWeight(semibold).fg(navbar-foreground)}>
      Dashboard
    </span>
  </navbar>

  <row grow nowrap align="stretch">
    <column
      shrink="0"
      w="56"
      gap="1"
      sx={style.bg(sidebar).fg(sidebar-foreground).padY(5).padX(4).borderE(sidebar-border)}
    >
      <a
        href="#home"
        sx={style.fg(sidebar-foreground).padY(2).padX(3).radius(md).when({ hover: style.bg(sidebar-accent).fg(sidebar-accent-foreground) })}
      >
        Home
      </a>
      <a
        href="#reports"
        sx={style.fg(sidebar-foreground).padY(2).padX(3).radius(md).when({ hover: style.bg(sidebar-accent).fg(sidebar-accent-foreground) })}
      >
        Reports
      </a>
      <a
        href="#team"
        sx={style.fg(sidebar-foreground).padY(2).padX(3).radius(md).when({ hover: style.bg(sidebar-accent).fg(sidebar-accent-foreground) })}
      >
        Team
      </a>
    </column>

    <column grow gap="6" sx={style.pad(6)} min-w="0">
      <span sx={style.fontSize(2xl).fontWeight(semibold).fg(foreground)}>
        Welcome back
      </span>

      <card>
        <span slot="header" sx={style.fontWeight(semibold)}>
          Summary
        </span>
        <column gap="2">
          <span sx={style.fg(muted-foreground)}>
            Here is a quick overview of your workspace activity.
          </span>
          <span sx={style.fontSize(sm).fg(muted-foreground)}>
            Check Reports for the latest metrics, or visit Team to manage members.
          </span>
        </column>
      </card>
    </column>
  </row>
</column>
