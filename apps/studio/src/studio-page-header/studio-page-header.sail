<!--
  <studio-page-header> — surface top bar: title + actions.
  Slots: leading, title (overrides title prop), actions, default (extra center).
  Ban-clean: layout keywords + L1 only.
-->
<scope>
  props: {
    title = ''
  }
</scope>

<row
  studio-page-header
  align="center"
  gap="2"
  shrink="0"
  nowrap
  justify="between"
  sx={style.h(7).padX(3).borderBWidth(1).borderB(border-subtle).bg(surface-1)}
>
  <row align="center" gap="2" nowrap grow min-w="0">
    <slot name="leading"></slot>
    <slot name="title">
      <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
        {title()}
      </p>
    </slot>
    <slot></slot>
  </row>
  <row align="center" gap="2" nowrap shrink="0">
    <slot name="actions"></slot>
  </row>
</row>
