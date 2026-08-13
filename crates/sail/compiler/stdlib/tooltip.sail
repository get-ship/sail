<!--
  <tooltip> — hover/focus text bubble. use:tipDescribe supplies per-instance
  anchor geometry vars (placement offsets + arrow insets), content text, and
  Escape. Style value language only.
  Arrow is a border-colored caret (not a rotated square) so one edge is not a
  double/broken seam against the bubble fill + shadow.
-->

<scope>
  props: {
    placement: 'top' | 'bottom' | 'left' | 'right' = 'top'
    content = ''
  }

  style host = {box: place({ position: relative }), paint: group()}
  // No opacity transition — linger after leave was a real product bug.
  style bubble = {box: place({ position: absolute, top: bind(ttTop), bottom: bind(ttBottom), left: bind(ttLeft), right: bind(ttRight), z: popover }).space({ pad-y: 1, pad-x: 2 }), paint: motion({ x: bind(ttTx) }).filter({ opacity: 0 }).fill({ color: popover }).ink({ color: popover-foreground }).radius(md).elevation(md).border({ width: 0 }).pointerEvents(none), text: type({ family: body, size: xs, weight: medium, wrap: nowrap })}
  // Zero-box border triangle; match placement points the filled edge outward.
  style arrowBox = {box: place({ position: absolute, top: bind(ttArrowTop), bottom: bind(ttArrowBottom), left: bind(ttArrowLeft), right: bind(ttArrowRight) }).size({ w: 0, h: 0 }).space({ ms: bind(ttArrowMl), mt: bind(ttArrowMt) }), paint: content(empty).fill({ color: transparent }).border({ width: 0, style: solid })}
  // placement=top → caret on bubble bottom, points down (border-top filled).
  // Border widths are px scale (0|1|2|3|4|8) — 4px sides ≈ prior diamond size.
  style arrowFromTop = {paint: border({ width: 4, edge: l, color: transparent }).border({ width: 4, edge: r, color: transparent }).border({ width: 4, edge: t, color: popover }).border({ width: 0, edge: b, color: transparent })}
  // placement=bottom → caret on bubble top, points up.
  style arrowFromBottom = {paint: border({ width: 4, edge: l, color: transparent }).border({ width: 4, edge: r, color: transparent }).border({ width: 4, edge: b, color: popover }).border({ width: 0, edge: t, color: transparent })}
  // placement=left → caret on bubble right, points right.
  style arrowFromLeft = {paint: border({ width: 4, edge: t, color: transparent }).border({ width: 4, edge: b, color: transparent }).border({ width: 4, edge: l, color: popover }).border({ width: 0, edge: r, color: transparent })}
  // placement=right → caret on bubble left, points left.
  style arrowFromRight = {paint: border({ width: 4, edge: t, color: transparent }).border({ width: 4, edge: b, color: transparent }).border({ width: 4, edge: r, color: popover }).border({ width: 0, edge: l, color: transparent })}
</scope>

<span use:tipDescribe style(host)>
  <slot></slot>
  <!-- tooltip role stamped by use:tipDescribe (closed part anatomy) -->
  <span
    data-tooltip-bubble
    style(bubble, {match: {
        placement: {
          top: { when: { after: style(arrowBox, arrowFromTop) } },
          bottom: { when: { after: style(arrowBox, arrowFromBottom) } },
          left: { when: { after: style(arrowBox, arrowFromLeft) } },
          right: { when: { after: style(arrowBox, arrowFromRight) } },
        },
      }, when: {group-hover: {paint: filter({ opacity: 100 })}, group-focus-within: {paint: filter({ opacity: 100 })}}})
  ></span>
</span>
