<!--
  <skeleton> — loading placeholder sheen. Style product for geometry;
  SKELETON_MAP owns the shimmer gradient / background-size / animation
  (Style cannot express that composite) via component.skeleton tokens.
  shape + size ordinary props.
-->

<scope>
  props: {
    shape: 'text' | 'line' | 'block' | 'circle' = 'text'
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  const base = style({ box: display(inline-block).minW(8).minH(3), paint: align(middle).radius(sm) })
  const shapeText = style({ box: w(full).h(3).minH(0) })
  const shapeLine = style({ box: w(full).h(3).minH(0) })
  const shapeBlock = style({ box: display(block).w(full).minH(20) })
  const shapeCircle = style({ box: minW(7).minH(7), paint: radius(full).aspect(square) })
</scope>

<span style([
  base,
  match(shape, {
    text: shapeText,
    line: shapeLine,
    block: shapeBlock,
    circle: shapeCircle,
  }),
  compound({ shape: text, size: sm }, style({ box: h(2) })),
  compound({ shape: text, size: md }, style({ box: h(3) })),
  compound({ shape: text, size: lg }, style({ box: h(4) })),
  compound({ shape: line, size: sm }, style({ box: h(2) })),
  compound({ shape: line, size: md }, style({ box: h(3) })),
  compound({ shape: line, size: lg }, style({ box: h(4) })),
  compound({ shape: circle, size: sm }, style({ box: size(icon-size-2xl).minW(0).minH(0) })),
  compound({ shape: circle, size: md }, style({ box: size(7).minW(0).minH(0) })),
  compound({ shape: circle, size: lg }, style({ box: size(12).minW(0).minH(0) })),
])></span>
