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

  style base = {box: flow({ display: inline-block }).size({ min-w: 8, min-h: 3 }), paint: align(middle).radius(sm)}
  // text + line share the full-width bar geometry.
  style shapeBar = {box: size({ w: full, h: 3, min-h: 0 })}
  style shapeBlock = {box: flow({ display: block }).size({ w: full, min-h: 20 })}
  style shapeCircle = {box: size({ min-w: 7, min-h: 7, aspect: square }), paint: radius(full)}
</scope>

<span
  style(base, {match: {shape: {text: shapeBar, line: shapeBar, block: shapeBlock, circle: shapeCircle}}, compound: [
      {shape: text, size: sm, box: size({ h: 2 })},
      {shape: text, size: md, box: size({ h: 3 })},
      {shape: text, size: lg, box: size({ h: 4 })},
      {shape: line, size: sm, box: size({ h: 2 })},
      {shape: line, size: md, box: size({ h: 3 })},
      {shape: line, size: lg, box: size({ h: 4 })},
      {shape: circle, size: sm, box: size({ min-w: 0, min-h: 0 })},
      {shape: circle, size: md, box: size({ min-w: 0, min-h: 0 })},
      {shape: circle, size: lg, box: size({ min-w: 0, min-h: 0 })},
    ]})
></span>
