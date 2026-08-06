<column>
  style({ box: minH(screen), paint: bg(background) })

  <navbar placement="top" sticky bordered>
    <span slot="brand">
      style({ paint: fg(navbar-foreground), text: fontSize(lg).fontWeight(semibold) })

      Dashboard
    </span>
  </navbar>

  <row nowrap align="stretch">
    style({ box: grow(1) })

    <column gap="1">
      style({ box: shrink(0).w(56).padY(5).padX(4), paint: bg(sidebar).fg(sidebar-foreground).borderE(sidebar-border) })

      <a
        href="#home"

      >
        style({ box: padY(2).padX(3), paint: fg(sidebar-foreground).radius(md) }).when(hover, { paint: bg(sidebar-accent).fg(sidebar-accent-foreground) })

        Home
      </a>
      <a
        href="#reports"

      >
        style({ box: padY(2).padX(3), paint: fg(sidebar-foreground).radius(md) }).when(hover, { paint: bg(sidebar-accent).fg(sidebar-accent-foreground) })

        Reports
      </a>
      <a
        href="#team"

      >
        style({ box: padY(2).padX(3), paint: fg(sidebar-foreground).radius(md) }).when(hover, { paint: bg(sidebar-accent).fg(sidebar-accent-foreground) })

        Team
      </a>
    </column>

    <column gap="6">
      style({ box: grow(1).minW(0).pad(6) })

      <span>
        style({ paint: fg(foreground), text: fontSize(2xl).fontWeight(semibold) })

        Welcome back
      </span>

      <card>
        <span slot="header">
          style({ text: fontWeight(semibold) })

          Summary
        </span>
        <column gap="2">
          <span>
            style({ paint: fg(muted-foreground) })

            Here is a quick overview of your workspace activity.
          </span>
          <span>
            style({ paint: fg(muted-foreground), text: fontSize(sm) })

            Check Reports for the latest metrics, or visit Team to manage members.
          </span>
        </column>
      </card>
    </column>
  </row>
</column>
