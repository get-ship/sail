<!--
  <accordion> — ONE disclosure element (replaces collapsible + details).
  Behavior: use:detailsToggle + use:detailsDisabled.
  Exclusive multi-item: shared `name` (native details name). Omit name for
  independent open; set `multiple` when several may stay open together.
  Chrome: disabled dim, summary chevron, stacked edge merge via descendant map.
-->

<scope>
  props: {
    open = false
    disabled = false
    name = ''
    multiple = false
  
    onToggle: (detail: { open: boolean }) => void
}

  // Shell + trigger from component.accordion tokens (global fallbacks in the pack).
  const host = style({ box: display(block), paint: borderWidth(1).border(accordion-border).radius(lg).bg(accordion), text: fontFamily(body) })
  // list(none) + after chevron: kill UA marker; single themed caret.
  const summary = style({ box: display(flex).padY(3).padX(4), paint: fg(accordion-foreground).cursor(pointer).list(none).select(none).radius(lg).transition(colors).duration(fast).ease(out), text: fontSize(sm).fontWeight(semibold) }).when(hover, { paint: bg(accordion-hover-bg) }).when(after, { box: ms(auto).size(4), paint: content(empty).bg(accordion-chevron) })
  const content = style({ box: padX(4).padY(3) })
</scope>

<details use:detailsToggle use:detailsDisabled style([
  host,
  compound({ disabled: true }, style({ paint: opacity(50).cursor(not-allowed) })),
])>
  <summary style(summary)><slot name="trigger"></slot></summary>
  <div><div style(content)><slot></slot></div></div>
</details>
