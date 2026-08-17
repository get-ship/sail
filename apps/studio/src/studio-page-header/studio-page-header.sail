<!--
  <studio-page-header> — surface top bar: title + actions.
  Slots: leading, title (overrides title prop), actions, default.
-->
<scope>
  props: {
    title = ''
  }
  // no preset: size lock + edge-only border
  style headerRoot = { size({ h: 7 }).pad(3).border({ edge: b, color: border-subtle, width: 1 }).bg(surface-1) }
  style leadRow = { size(grow) }
  style eyebrow = { fg(muted).type(label) }
  style actionsRow = { size(hug) }
</scope>

<row studio-page-header align="center" gap="2" nowrap justify="between" style(headerRoot)>
  <row align="center" gap="2" nowrap style(leadRow)>
    <slot name="leading"></slot>
    <slot name="title">
      <p style(eyebrow)>
        {title()}
      </p>
    </slot>
    <slot></slot>
  </row>
  <row align="center" gap="2" nowrap style(actionsRow)>
    <slot name="actions"></slot>
  </row>
</row>
