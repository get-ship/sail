<!--
  <studio-page-header> — surface top bar: title + actions.
  Slots: leading, title (overrides title prop), actions, default.
-->
<scope>
  props: {
    title = ''
  }
  style headerRoot = { shrink(0).size({ h: 7 }).pad({ x: 3 }).border({ edge: b, color: border-subtle, width: 1 }).bg(surface-1) }
  style leadRow = { grow(1).size({ min-w: 0 }) }
  style eyebrow = { fg(muted-foreground).type({ size: xs, weight: semibold, tracking: wide, case: uppercase }) }
  style actionsRow = { shrink(0) }
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
