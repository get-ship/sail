<!--
  <toast-region> — fixed stacking region. Style value language only.

  Paint (Style product only):
    host sx={[…]} — base fixed/z/pad/pointer-events + default (top-start)
                    insets + panel width; style({ match: { position: {  } } }) carries the four
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

  style base = { box: position(fixed).pad(4).top(0).right(auto).bottom(auto).left(0).w(panel-w-toast), paint: z(z-toast).pointerEvents(none) }
</scope>

<column gap="2" style(base, { match: { position: { top-start: { box: top(0).right(auto).bottom(auto).left(0) }, top-end: { box: top(0).right(0).bottom(auto).left(auto) }, bottom-start: { box: top(auto).right(auto).bottom(0).left(0).flexDir(column-reverse) }, bottom-end: { box: top(auto).right(0).bottom(0).left(auto).flexDir(column-reverse) } } } })>
  <slot></slot>
</column>
