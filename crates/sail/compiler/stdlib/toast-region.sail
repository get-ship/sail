<!--
  <toast-region> — fixed stacking region. Style value language only.

  Paint (Style product only):
    host sx={[…]} — base fixed/z/pad/pointer-events + default (top-start)
                    insets + panel width; match(position) carries the four
                    inset matrices; bottom positions reverse the column.
    CVA-in-props — the position map survives as the reflected union (first
                    key = default); its legacy sx string values are discarded
                    at parse (Style match owns the paint — scope_block.rs).
  Legacy host atom string this replaces (pinned by
  sx_host_paint_probe::toast_region_host_paint_on_sx_cva):
    fixed z-(--z-toast) p-4 pointer-events-none … w-panel-w-toast
-->

<scope>
  props: {
    position: {
      'top-start': 'top-0 right-auto bottom-auto left-0',
      'top-end': 'top-0 right-0 bottom-auto left-auto',
      'bottom-start': 'top-auto right-auto bottom-0 left-0 col-reverse',
      'bottom-end': 'top-auto right-0 bottom-0 left-auto col-reverse',
    }
  }

  const base = style.position(fixed).z(z-toast).pad(4).pointerEvents(none).top(0).right(auto).bottom(auto).left(0).w(panel-w-toast)
</scope>

<column gap="2" sx={[
  base,
  match(position, {
    top-start: style.top(0).right(auto).bottom(auto).left(0),
    top-end: style.top(0).right(0).bottom(auto).left(auto),
    bottom-start: style.top(auto).right(auto).bottom(0).left(0).flexDir(column-reverse),
    bottom-end: style.top(auto).right(0).bottom(0).left(auto).flexDir(column-reverse),
  }),
]}>
  <slot></slot>
</column>
