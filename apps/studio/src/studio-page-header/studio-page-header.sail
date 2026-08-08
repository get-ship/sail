<!--
  <studio-page-header> — surface top bar: title + actions.
  Slots: leading, title (overrides title prop), actions, default.
-->
<scope>
  props: {
    title = ''
  }
  style headerRoot = { box: shrink(0).h(7).padX(3), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1) }
  style leadRow = { box: grow(1).minW(0) }
  style eyebrow = { paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) }
  style actionsRow = { box: shrink(0) }
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
