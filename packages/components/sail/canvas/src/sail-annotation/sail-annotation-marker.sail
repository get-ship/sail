<!--
  <sail-annotation-marker> — point pin or region box for a staged/sent annotation.
  Geometry is capture-time overlay px via use:pin / use:rect + Style bind.
  Point pins center on (x, y) via 0×0 anchor + <center>.
  Point pin is 24×24 so the tap target meets the 24px floor.
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

  style anchor = { position({ scheme: absolute, t: bind(pinY), l: bind(pinX) }).size({ w: 0, h: 0 }) }
  style pin = { size({ w: 6, h: 6 }).radius(full).bg(primary).fg(primary-foreground).border({ color: background, width: 1 }).elevation(sm).cursor(pointer).pointerEvents(auto), when: { stateActive: border({ kind: outline, color: ring-color }) } }
  style badge = { display(contents).type({ family: body, size: xs, weight: medium }) }
  style region = { position({ scheme: absolute, t: bind(boxTop), l: bind(boxLeft) }).size({ w: bind(boxW), h: bind(boxH) }).bg(hover-bg).border({ color: primary, width: 1, style: dashed }).radius(sm).cursor(pointer).pointerEvents(auto) }
</scope>

<controller>

import type { Scope } from './sail-annotation-marker.sail'

interface BoxRect {
  x: number
  y: number
  w: number
  h: number
}

/** Wire form "x,y,w,h" in overlay px. Zero size = no region box. */
const NO_BOX: BoxRect = { x: 0, y: 0, w: 0, h: 0 }

function parseRect(raw: string): BoxRect {
  const parts = raw.split(',').map(Number)
  if (parts.length !== 4 || !parts.every((n) => Number.isFinite(n))) return NO_BOX
  const [x, y, w, h] = parts as [number, number, number, number]
  return { x, y, w, h }
}

export default controller(({ kind, rect, index, active }: Scope) => {
    const box = derived(() => parseRect(rect()))
    return {
      isPoint: derived(() => kind() !== 'region'),
      box,
      hasBox: derived(() => {
        const b = box()
        return b.w !== 0 || b.h !== 0
      }),
      badge: derived(() => String(index() + 1)),
      state: derived(() => (active() ? 'active' : '')),
    }
  })

</controller>
<if {isPoint()}>
  <div use:pin={{x: x(), y: y()}} style(anchor)>
    <center style({ size({ h: 0 }) })>
      <row
        role="button"
        tabindex="0"
        gap="0"
        justify="center"
        data-state={state()}
        style(pin)
      >
        <span style(badge)>{badge()}</span>
      </row>
    </center>
  </div>
{else-if {hasBox()} =>
  <div
    role="button"
    tabindex="0"
    use:rect={box()}
    style(region)
  ></div>
}</if>
