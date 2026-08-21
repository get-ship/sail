<scope>
  props: {
    kind: 'subtle' | 'default' | 'strong' | 'color-primary' | 'color-accent' | 'color-muted' | 'width-hairline' | 'width-1' | 'width-2' | 'width-4' | 'width-8' | 'width-control' | 'width-ring' | 'style-solid' | 'style-dashed' | 'style-dotted' | 'style-double' | 'style-hidden' | 'style-none' | 'edge-all' | 'edge-x' | 'edge-y' | 'edge-t' | 'edge-r' | 'edge-b' | 'edge-l' | 'edge-s' | 'edge-e' | 'kind-stroke' | 'kind-outline' | 'kind-ring' | 'kind-image' | 'offset-0' | 'offset-1' | 'offset-2' | 'offset-ring' | 'offset-inset' | 'grad-primary' | 'grad-accent' | 'grad-surface' | 'grad-none' = 'color-primary'
  }
  style presetSubtle = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border(subtle) }
  style presetDefault = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border(default) }
  style presetStrong = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border(strong) }
  style colorPrimary = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ color: { ref: primary }, width: 4, style: solid }) }
  style colorAccent = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ color: { ref: accent }, width: 4, style: solid }) }
  style colorMuted = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ color: { ref: muted }, width: 4, style: solid }) }
  style widthHairline = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ width: hairline, color: { ref: primary }, style: solid }) }
  style width1 = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ width: 1, color: { ref: primary }, style: solid }) }
  style width2 = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ width: 2, color: { ref: primary }, style: solid }) }
  style width4 = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ width: 4, color: { ref: primary }, style: solid }) }
  style width8 = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ width: 8, color: { ref: primary }, style: solid }) }
  style widthControl = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ width: control-stroke, color: { ref: primary }, style: solid }) }
  style widthRing = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ width: ring-width, color: { ref: primary }, style: solid }) }
  style styleSolid = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ style: solid, width: 4, color: { ref: primary } }) }
  style styleDashed = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ style: dashed, width: 4, color: { ref: primary } }) }
  style styleDotted = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ style: dotted, width: 4, color: { ref: primary } }) }
  style styleDouble = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ style: double, width: 4, color: { ref: primary } }) }
  style styleHidden = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ style: hidden, width: 4, color: { ref: primary } }) }
  style styleNone = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ style: none, width: 4, color: { ref: primary } }) }
  style edgeAll = { size({ w: 24, h: 16 }), bg(surface), border({ edge: all, color: { ref: primary }, width: 4, style: solid }) }
  style edgeX = { size({ w: 24, h: 16 }), bg(surface), border({ edge: x, color: { ref: primary }, width: 4, style: solid }) }
  style edgeY = { size({ w: 24, h: 16 }), bg(surface), border({ edge: y, color: { ref: primary }, width: 4, style: solid }) }
  style edgeT = { size({ w: 24, h: 16 }), bg(surface), border({ edge: t, color: { ref: primary }, width: 4, style: solid }) }
  style edgeR = { size({ w: 24, h: 16 }), bg(surface), border({ edge: r, color: { ref: primary }, width: 4, style: solid }) }
  style edgeB = { size({ w: 24, h: 16 }), bg(surface), border({ edge: b, color: { ref: primary }, width: 4, style: solid }) }
  style edgeL = { size({ w: 24, h: 16 }), bg(surface), border({ edge: l, color: { ref: primary }, width: 4, style: solid }) }
  style edgeS = { size({ w: 24, h: 16 }), bg(surface), border({ edge: s, color: { ref: primary }, width: 4, style: solid }) }
  style edgeE = { size({ w: 24, h: 16 }), bg(surface), border({ edge: e, color: { ref: primary }, width: 4, style: solid }) }
  style kindStroke = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ kind: stroke, color: { ref: primary }, width: 4, style: solid }) }
  style kindOutline = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ kind: outline, color: { ref: primary }, width: 4, style: solid }) }
  style kindRing = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ kind: ring, color: { ref: primary }, width: 4 }) }
  style kindImage = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ kind: image, color: { ref: primary } }) }
  style offset0 = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ kind: outline, color: { ref: primary }, width: 4, style: solid, offset: 0 }) }
  style offset1 = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ kind: outline, color: { ref: primary }, width: 4, style: solid, offset: 1 }) }
  style offset2 = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ kind: outline, color: { ref: primary }, width: 4, style: solid, offset: 2 }) }
  style offsetRing = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ kind: outline, color: { ref: primary }, width: 4, style: solid, offset: ring-offset }) }
  style offsetInset = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ kind: outline, color: { ref: primary }, width: 4, style: solid, offset: ring-inset }) }
  style gradPrimary = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ width: 4, style: solid, gradient: primary }) }
  style gradAccent = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ width: 4, style: solid, gradient: accent }) }
  style gradSurface = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ width: 4, style: solid, color: muted, gradient: surface }) }
  style gradNone = { size({ w: 24, h: 16 }), bg(surface), radius(sm), border({ width: 4, style: solid, color: muted, gradient: none }) }
</scope>
<column style({ match: { kind: { subtle: presetSubtle, 'default': presetDefault, strong: presetStrong, 'color-primary': colorPrimary, 'color-accent': colorAccent, 'color-muted': colorMuted, 'width-hairline': widthHairline, 'width-1': width1, 'width-2': width2, 'width-4': width4, 'width-8': width8, 'width-control': widthControl, 'width-ring': widthRing, 'style-solid': styleSolid, 'style-dashed': styleDashed, 'style-dotted': styleDotted, 'style-double': styleDouble, 'style-hidden': styleHidden, 'style-none': styleNone, 'edge-all': edgeAll, 'edge-x': edgeX, 'edge-y': edgeY, 'edge-t': edgeT, 'edge-r': edgeR, 'edge-b': edgeB, 'edge-l': edgeL, 'edge-s': edgeS, 'edge-e': edgeE, 'kind-stroke': kindStroke, 'kind-outline': kindOutline, 'kind-ring': kindRing, 'kind-image': kindImage, 'offset-0': offset0, 'offset-1': offset1, 'offset-2': offset2, 'offset-ring': offsetRing, 'offset-inset': offsetInset, 'grad-primary': gradPrimary, 'grad-accent': gradAccent, 'grad-surface': gradSurface, 'grad-none': gradNone } } })>
  <slot></slot>
</column>
