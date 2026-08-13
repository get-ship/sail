<!--
  <doc-example> — docs example: always side-by-side live + source.
  Code|Live|Split stays on the Usage “complete program” block in the viewer —
  examples keep the simpler split that reads well on detail pages.
-->

<scope>
  props: {
    title = ''
    lead = ''
    code = ''
  }

  style section = { box: size({ w: full }).space({ pad-y: 4 }).overflow({ all: visible }) }
  style heading = { text: type({ size: md, weight: semibold }) }
  style leadText = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style liveFrame = {
    box: flow({ grow: 1 }).size({min-w: 0, basis: 0}).space({ pad: 4 }).overflow({ all: visible }),
    paint: radius(lg).border({ color: border, width: 1 }).fill({ color: surface-0 }),
}
  style codeCol = { box: flow({ grow: 1 }).size({min-w: 0, basis: 0}) }
</scope>

<column gap="3" style(section)>
  <if {title() !== ''}>
    <h3 style(heading)>{title()}</h3>
  </if>
  <if {lead() !== ''}>
    <p style(leadText)>{lead()}</p>
  </if>
  <row gap="4" align="start" style({ box: size({ w: full, min-w: 0 }).overflow({ all: visible }) })>
    <column gap="3" style(liveFrame)>
      <slot></slot>
    </column>
    <div style(codeCol)>
      <doc-code code={code()} lang="sail"></doc-code>
    </div>
  </row>
</column>
