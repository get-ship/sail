<!--
  <toast-region> — fixed stacking region. Style value language only.
-->

<scope>
  props: {
    position: 'top-start' | 'top-end' | 'bottom-start' | 'bottom-end' = 'top-start'
  }

  const base = style.position(fixed).z(toast-region-z).pad(4).pointerEvents(none).top(0).right(auto).bottom(auto).left(0).w(toast-region-width)
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
