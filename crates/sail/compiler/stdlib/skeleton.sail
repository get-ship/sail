<!--
  <skeleton> — loading placeholder sheen. Style product for geometry;
  SKELETON_MAP owns the shimmer gradient / background-size / animation
  (Style cannot express that composite) via muted.
  shape + size ordinary props.
-->

<scope>
  props: {
    shape: 'text' | 'line' | 'block' | 'circle' = 'text'
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  style base = {display(inline-block).size({ min-w: 8, min-h: 3 }).align(middle).radius(sm)}
  // text + line share the full-width bar geometry.
  style shapeBar = {size({ w: full, h: 3, min-h: 0 })}
  style shapeBlock = {display(block).size({ w: full, min-h: 20 })}
  style shapeCircle = {size({ min-w: 7, min-h: 7, aspect: square }).radius(full)}
</scope>

<span
  style(base, {match: {shape: {text: shapeBar, line: shapeBar, block: shapeBlock, circle: shapeCircle}}, compound: [
      {shape: text, size: sm, size({ h: 2 })},
      {shape: text, size: md, size({ h: 3 })},
      {shape: text, size: lg, size({ h: 4 })},
      {shape: line, size: sm, size({ h: 2 })},
      {shape: line, size: md, size({ h: 3 })},
      {shape: line, size: lg, size({ h: 4 })},
      {shape: circle, size: sm, size({ min-w: 0, min-h: 0 })},
      {shape: circle, size: md, size({ min-w: 0, min-h: 0 })},
      {shape: circle, size: lg, size({ min-w: 0, min-h: 0 })},
    ]})
></span>
