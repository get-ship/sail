<!--
  <skeleton> — loading placeholder sheen. Style product for geometry;
  platform/skeleton.css owns the shimmer gradient / background-size /
  animation (Style cannot express that composite). Paint is muted mixed
  with muted-foreground so the bar reads on light and dark surfaces.
  shape + size ordinary props. No host-level skeleton prop on other
  elements — compose circle + lines.
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
  style shapeCircle = {size(8).radius(full)}
</scope>

<span
  style(base, {match: {shape: {text: shapeBar, line: shapeBar, block: shapeBlock, circle: shapeCircle}}, compound: [
      {shape: text, size: sm, size({ h: 2 })},
      {shape: text, size: md, size({ h: 3 })},
      {shape: text, size: lg, size({ h: 4 })},
      {shape: line, size: sm, size({ h: 2 })},
      {shape: line, size: md, size({ h: 3 })},
      {shape: line, size: lg, size({ h: 4 })},
      {shape: circle, size: sm, size(6)},
      {shape: circle, size: md, size(8)},
      {shape: circle, size: lg, size(10)},
    ]})
></span>
