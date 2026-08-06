<!--
  <details> — native disclosure. Style product only.
  Platform owns open/close + a11y. summary paint is the summary element.
  Stacked accordion merge + open body spacing via descendant map.
-->

<scope>
  props: {
    open = false
  
    onToggle: (detail: { open: boolean }) => void
}

  const base = style({ paint: borderWidth(1).border(border).radius(lg).bg(surface-1) })
</scope>

<details>
  style(base)

  <slot></slot>
</details>
