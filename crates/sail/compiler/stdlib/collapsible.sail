<!--
  <collapsible> — native disclosure. Behavior: use:detailsToggle + use:detailsDisabled.
  Style product: host type + content pad; disabled dims via compound.

  Summary must kill the UA disclosure marker (list-style + ::marker) and
  own a single chevron — otherwise authors who add their own caret get a
  double-triangle pile-up on the status rail.
-->

<scope>
  props: {
    open = false
    disabled = false
  }

  const host = style({ box: display(block), text: fontFamily(body) })
  // list(none) + after chevron: same contract as <summary> stdlib.
  // Consumers put their row in the trigger slot; summary is a flex shell.
  const summary = style({ box: display(flex).padY(0).padX(0), paint: cursor(pointer).list(none).select(none) }).when(after, { box: ms(auto).size(4), paint: content(empty).bg(muted-foreground) })
  const content = style({ box: padX(4).padY(3) })
</scope>

<details use:detailsToggle use:detailsDisabled>
  style([
  host,
  compound({ disabled: true }, style({ paint: opacity(50).cursor(not-allowed) })),
])

  <summary>
    style(summary)
<slot name="trigger"></slot></summary>
  <div><div>
    style(content)
<slot></slot></div></div>
</details>
