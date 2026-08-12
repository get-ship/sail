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

  style section = { box: w(full).padY(4).overflow(visible) }
  style heading = { text: fontSize(md).fontWeight(semibold) }
  style leadText = { paint: fg(muted-foreground), text: fontSize(sm) }
  style liveFrame = {
    box: grow(1).basis(0).minW(0).pad(4).overflow(visible),
    paint: radius(lg).borderWidth(1).border(border).bg(surface-0),
  }
  style codeCol = { box: grow(1).basis(0).minW(0) }
</scope>

<column gap="3" style(section)>
  <if {title() !== ''}>
    <h3 style(heading)>{title()}</h3>
  </if>
  <if {lead() !== ''}>
    <p style(leadText)>{lead()}</p>
  </if>
  <row gap="4" align="start" style({ box: w(full).minW(0).overflow(visible) })>
    <column gap="3" style(liveFrame)>
      <slot></slot>
    </column>
    <div style(codeCol)>
      <doc-code code={code()} lang="sail"></doc-code>
    </div>
  </row>
</column>
