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

  style base = { box: display(inline-block).minW(8).minH(3), paint: align(middle).radius(sm) }
  // text + line share the full-width bar geometry.
  style shapeBar = { box: w(full).h(3).minH(0) }
  style shapeBlock = { box: display(block).w(full).minH(20) }
  style shapeCircle = { box: minW(7).minH(7), paint: radius(full).aspect(square) }
</scope>

<span
  style(base, {
    match: {
      shape: {
        text: shapeBar,
        line: shapeBar,
        block: shapeBlock,
        circle: shapeCircle,
      },
    },
    compound: [
      { shape: text, size: sm, box: h(2) },
      { shape: text, size: md, box: h(3) },
      { shape: text, size: lg, box: h(4) },
      { shape: line, size: sm, box: h(2) },
      { shape: line, size: md, box: h(3) },
      { shape: line, size: lg, box: h(4) },
      { shape: circle, size: sm, box: size(icon-size-2xl).minW(0).minH(0) },
      { shape: circle, size: md, box: size(7).minW(0).minH(0) },
      { shape: circle, size: lg, box: size(12).minW(0).minH(0) },
    ],
  })
></span>
