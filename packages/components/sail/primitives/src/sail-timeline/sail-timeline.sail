<!--
  <sail-timeline> — SAIL port of <ship-timeline>.
  Vertical event list with left-gutter connector line and tone-colored dots.

  Prop parity vs <ship-timeline>
  ─────────────────────────────────────────────────────────────────────────────
  Lit                          SAIL                      Notes
  ─────────────────────────────────────────────────────────────────────────────
  (no props)                   (no props)               ✓
  ─────────────────────────────────────────────────────────────────────────────
  slots: (default) items       (default) items          ✓
  ─────────────────────────────────────────────────────────────────────────────
  parts: list                  list                     ✓
  ─────────────────────────────────────────────────────────────────────────────
  ARIA: role="list" on host    controller sets same     ✓
  ─────────────────────────────────────────────────────────────────────────────

  NEW SURFACE: `<sail-stack>` (removed from this surface) -> the built-in
  `<column>` keyword — same flex-column + gap behavior, no visual change.
  No LANG-GAP.
-->

<controller>

export default controller(() => {
  mount((host) => {
    if (!host.hasAttribute('role')) host.setAttribute('role', 'list')
  })
  return {}
})

</controller>
<column gap="1">
  style({ box: position(relative) })
<slot></slot></column>

