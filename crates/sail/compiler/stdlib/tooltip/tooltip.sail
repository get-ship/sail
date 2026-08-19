<!--
  <tooltip> — hover/focus text bubble. use:tipDescribe supplies per-instance
  anchor geometry vars (placement offsets + arrow insets), content text,
  Escape, and data-hidden (instant hide state). Style value language only.
  Arrow is a border-colored caret (not a rotated square) so one edge is not a
  double/broken seam against the bubble fill + shadow.
-->

<scope>
  props: {
    placement: 'top' | 'bottom' | 'left' | 'right' = 'top'
    content = ''
  }

  // inline-flex + self(start): hug the trigger. As a column flex child the
  // host otherwise stretches full-width, so a long bubble centers on the
  // column instead of the button chrome.
  style host = {position({ scheme: relative, name: tooltip }).display(inline-flex).self(start).group()}
  // No opacity transition — linger after leave was a real product bug.
  // Hug the hint. Wrap only after a real max — max-w alone makes a skinny column.
  // Instant hide is state: use:tipDescribe stamps data-hidden; this skin
  // beats group-hover / group-focus-within so Escape-while-hovering snaps.
  style hide = {filter({ opacity: 0 })}
  style show = {filter({ opacity: 100 }), match: { 'data-hidden': { true: hide } }}
  style bubble = {position({ scheme: absolute, anchor: tooltip, area: top, z: popover }).pad({ y: 1, x: 2 }).size({ w: max, min-w: max }).overflow({ all: visible }).filter({ opacity: 0 }).bg(overlay).fg(foreground).radius(md).elevation(md).border({ width: 0 }).pointerEvents(none).type({ family: body, size: xs, weight: medium, wrap: nowrap }), match: { placement: { top: position({ area: top }), bottom: position({ area: bottom }), left: position({ area: left }), right: position({ area: right }) } }}
  // Zero-box border triangle; match placement points the filled edge outward.
  style arrowBox = {position({ scheme: absolute, b: auto }).size({ w: 0, h: 0 }).content(empty).bg(transparent).border({ width: 0, style: solid })}
  // placement=top → caret on bubble bottom, points down (border-top filled).
  // Border widths are px scale (0|1|2|3|4|8) — 4px sides ≈ prior diamond size.
  style arrowFromTop = {border({ width: 4, edge: l, color: transparent }).border({ width: 4, edge: r, color: transparent }).border({ width: 4, edge: t, color: overlay }).border({ width: 0, edge: b, color: transparent })}
  // placement=bottom → caret on bubble top, points up.
  style arrowFromBottom = {border({ width: 4, edge: l, color: transparent }).border({ width: 4, edge: r, color: transparent }).border({ width: 4, edge: b, color: overlay }).border({ width: 0, edge: t, color: transparent })}
  // placement=left → caret on bubble right, points right.
  style arrowFromLeft = {border({ width: 4, edge: t, color: transparent }).border({ width: 4, edge: b, color: transparent }).border({ width: 4, edge: l, color: overlay }).border({ width: 0, edge: r, color: transparent })}
  // placement=right → caret on bubble left, points left.
  style arrowFromRight = {border({ width: 4, edge: t, color: transparent }).border({ width: 4, edge: b, color: transparent }).border({ width: 4, edge: r, color: overlay }).border({ width: 0, edge: l, color: transparent })}
</scope>

<span use:tipDescribe style(host)>
  <slot></slot>
  <!-- tooltip role stamped by use:tipDescribe (closed part anatomy) -->
  <span
    data-tooltip-bubble
    style(bubble, {match: { placement: { top: { when: { after: style(arrowBox, arrowFromTop) } }, bottom: { when: { after: style(arrowBox, arrowFromBottom) } }, left: { when: { after: style(arrowBox, arrowFromLeft) } }, right: { when: { after: style(arrowBox, arrowFromRight) } } }, 'data-hidden': { true: hide } }, when: {group-hover: show, group-focus-within: show}})
  ></span>
</span>
