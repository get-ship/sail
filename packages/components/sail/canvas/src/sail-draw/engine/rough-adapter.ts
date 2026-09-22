// sail-draw — the SOLE roughjs importer ("one contract per boundary"). Wraps
// the roughjs generator to roughen a clean shape into the hand-drawn
// ("sketchy") aesthetic, returning an SVG path `d` string the view binds to.
//
// The scene rendering binds to THIS adapter's `roughPath`, never to the
// roughjs core directly. roughjs is rendering-aesthetic only — geometry +
// hit-testing stay in shape.ts/stroke.ts (ours).

import rough from 'roughjs'
import type { Drawable, OpSet } from 'roughjs/bin/core'
import type { Vec2 } from '../../engine/types'
import { rectFromPoints, type ShapeKind } from './shape'

const generator = rough.generator()

/** Convert a roughjs OpSet (move/bcurveTo/lineTo ops) into an SVG path `d`. */
function opSetToPath(set: OpSet): string {
  const out: string[] = []
  for (const op of set.ops) {
    const d = op.data
    if (op.op === 'move') out.push(`M ${d[0]} ${d[1]}`)
    else if (op.op === 'lineTo') out.push(`L ${d[0]} ${d[1]}`)
    else if (op.op === 'bcurveTo') out.push(`C ${d[0]} ${d[1]} ${d[2]} ${d[3]} ${d[4]} ${d[5]}`)
  }
  return out.join(' ')
}

/** Flatten all stroke op-sets of a Drawable into one path `d`. */
function drawableToPath(drawable: Drawable): string {
  return drawable.sets
    .filter((s) => s.type === 'path')
    .map(opSetToPath)
    .join(' ')
    .trim()
}

/** Roughen a shape (from its two drag endpoints) into a sketchy SVG path.
 *  Seeded so a given shape re-renders identically across frames. */
export function roughPath(kind: ShapeKind, a: Vec2, b: Vec2, seed = 1): string {
  const opts = { seed, roughness: 1.2, bowing: 1 }
  let drawable: Drawable
  switch (kind) {
    case 'rect': {
      const r = rectFromPoints(a, b)
      drawable = generator.rectangle(r.x, r.y, r.w, r.h, opts)
      break
    }
    case 'ellipse': {
      const r = rectFromPoints(a, b)
      drawable = generator.ellipse(r.x + r.w / 2, r.y + r.h / 2, r.w, r.h, opts)
      break
    }
    case 'line':
    case 'arrow':
      drawable = generator.line(a.x, a.y, b.x, b.y, opts)
      break
  }
  return drawableToPath(drawable)
}
