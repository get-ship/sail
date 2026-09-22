<!--
  <resizable> — two-pane split. Start + end slots with a drag handle between.
  orientation horizontal (default) or vertical. use:resizable on the handle:
  pointer drag and arrow keys set flex-basis % on the start pane.
  Handle role=separator + aria-orientation are stamped by the action.

  Stays INTERNAL single-tag use:, not a <controller>: split-drag is
  node-lifetime end to end — pointermove must follow the pointer past the
  handle (capture + window listeners), the host box is measured every move,
  and the result lands as inline flex-basis on a sibling pane. Controllers
  are signals-only, CSS resize resizes one element from its own corner
  (no native splitter), and no generic drag action exists to bind instead.
-->

<scope>
  props: {
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  }

  // h(full) is required so vertical flex-basis % has a definite container.
  style host = {display(flex).size({ w: 100%, h: 100%, min-h: 32, min-w: 0 })}
  style horiz = {dir(row)}
  style vert = {dir(column)}
  style pane = {grow(1).shrink(1).size({ min-w: 0, min-h: 0, basis: 0 }).overflow({ all: hidden })}
  // Hit is 3 (12px); the 1 (4px) visual line is ::before, centered.
  // Rest chrome is horizontal (CSS flex-direction default + no host attr).
  // Vertical overrides require orientation stamped onto the handle (use:resizable).
  style handle = {position(relative).shrink(0).self(stretch).appearance(none).pad(0).margin(0).border({ width: 0 }).bg(transparent).select(none).touch(none).tapHighlight(none).size({ min-w: 0, min-h: 0, w: 3 }).cursor(col-resize)}
  style handleV = {size({ w: auto, h: 3 }).cursor(row-resize)}
  style lineH = {position({ scheme: absolute, t: 0, b: 0, l: 1 }).size({ w: 1 }).bg(border).content(empty).pointerEvents(none)}
  style lineV = {position({ scheme: absolute, l: 0, r: 0, t: 1, b: auto }).size({ w: auto, h: 1 }).bg(border).content(empty).pointerEvents(none)}
</scope>

<div style(host, {match: { orientation: { horizontal: horiz, vertical: vert } }})>
  <div style(pane)>
    <slot name="start"></slot>
  </div>
  <button
    type="button"
    anatomy
    label="Resize"
    use:resizable
    style(handle, {match: { orientation: { vertical: handleV } }, when: {before: style(lineH), hover: {when: {before: bg(border-strong)}}, focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 })}, compound: [
      {orientation: vertical, when: {before: style(lineV)}},
    ]})
  ></button>
  <div style(pane)>
    <slot name="end"></slot>
  </div>
</div>
