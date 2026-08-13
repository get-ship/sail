<!--
  <studio-page-header> — surface top bar: title + actions.
  Slots: leading, title (overrides title prop), actions, default.
-->
<scope>
  props: {
    title = ''
  }
  style headerRoot = { box: flow({ shrink: 0 }).size({ h: 7 }).space({ pad-x: 3 }), paint: border({ edge: b, color: border-subtle, width: 1 }).fill({ color: surface-1 }) }
  style leadRow = { box: flow({ grow: 1 }).size({ min-w: 0 }) }
  style eyebrow = { paint: ink({ color: muted-foreground }), text: type({ size: xs, weight: semibold, tracking: wide, case: uppercase }) }
  style actionsRow = { box: flow({ shrink: 0 }) }
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
