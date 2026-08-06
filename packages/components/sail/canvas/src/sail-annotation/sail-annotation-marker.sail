<!--
  <sail-annotation-marker> — SAIL port of <ship-annotation-marker>.
  Visual pin or region-box marker for a staged / sent annotation.

  Prop surface
  ──────────────────────────────────────────────────────────────────────────────
  kind   (point|region, 'point', reflect) — which visual form to render
  x      (number, 0)                      — point mode: left offset in px
  y      (number, 0)                      — point mode: top offset in px
  rect   (string, '')                     — region mode: "x,y,w,h" in px
  active (bool, false, reflect)           — highlights the marker
  index  (number, 0)                      — ordinal badge inside point pins
  ──────────────────────────────────────────────────────────────────────────────
  Geometry is DYNAMIC (annotation coordinates computed at capture time), so it
  rides the `style:` channel: point pins bind left/top, region boxes bind the
  full rect. Coordinates are px in the annotation overlay's box — the marker's
  own box is unpositioned, so both parts resolve against the layer's overlay.

  A pin is centered ON its point: the 0×0 anchor + <center> puts the pin's
  midpoint exactly on (x, y), which is what the capture gesture recorded.
-->

<scope>
  props: {
    kind: 'point' | 'region' = 'point'
    x = 0
    y = 0
    rect = ''
    active = false
    index = 0
  }
</scope>

<controller>

import type { Scope } from './sail-annotation-marker.sail'

interface BoxRect {
  x: number
  y: number
  w: number
  h: number
}

/** `rect` is the wire form "x,y,w,h" in overlay px. `w === 0 && h === 0` is the
 *  no-box resting value (an unparseable or absent rect draws nothing). */
const NO_BOX: BoxRect = { x: 0, y: 0, w: 0, h: 0 }

function parseRect(raw: string): BoxRect {
  const parts = raw.split(',').map(Number)
  if (parts.length !== 4 || !parts.every((n) => Number.isFinite(n))) return NO_BOX
  const [x, y, w, h] = parts as [number, number, number, number]
  return { x, y, w, h }
}

export default controller(({ kind, rect, index, active }: Scope) => ({
    isPoint: derived(() => kind() !== 'region'),
    box: derived(() => parseRect(rect())),
    hasBox: derived(() => {
      const b = parseRect(rect())
      return b.w !== 0 || b.h !== 0
    }),
    // The badge is 1-based: the first annotation reads "1".
    badge: derived(() => String(index() + 1)),
    state: derived(() => (active() ? 'active' : '')),
  }))

</controller>
<if {isPoint()}>
  <div use:pin={{x: x(), y: y()}}>
    style({ box: position(absolute).w(0).h(0).left(bind(pinX)).top(bind(pinY)) })

    <center>
      style({ box: h(0) })

      <row
        role="button"
        tabindex="0"
        gap="0"
        justify="center"
        data-state={state()}

      >
        style({ box: w(5).h(5), paint: radius(full).bg(primary).fg(primary-foreground).borderWidth(1).border(background).shadow(sm).cursor(pointer).pointerEvents(auto), text: fontSize(xs).fontWeight(medium).fontFamily(body) }).when(stateActive, { paint: outline(ring-color) })
{badge()}</row>
    </center>
  </div>
<else-if {hasBox()}>
  <div
    role="button"
    tabindex="0"
    use:rect={box()}

  >
    style({ box: position(absolute).left(bind(boxLeft)).top(bind(boxTop)).w(bind(boxW)).h(bind(boxH)), paint: bg(hover-bg).borderWidth(1).border(primary).border(dashed).radius(sm).cursor(pointer).pointerEvents(auto) })
</div>
</else-if></if>
