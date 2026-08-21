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

  style section = { size(max).overflow(visible) }
  style heading = { type(heading) }
  style leadText = { fg(muted).type(body-sm) }
  style liveFrame = { size(grow).overflow(visible).radius(lg).border(subtle).bg(app) }
  style codeCol = { size(grow) }
</scope>

<column gap="3" style(section)>
  <if {title() !== ''}>
    <h3 style(heading)>{title()}</h3>
  </if>
  <if {lead() !== ''}>
    <p style(leadText)>{lead()}</p>
  </if>
  <row nowrap gap="4" align="start" style(section)>
    <column gap="3" style(liveFrame)>
      <slot></slot>
    </column>
    <div style(codeCol)>
      <doc-code code={code()} lang="sail"></doc-code>
    </div>
  </row>
</column>
