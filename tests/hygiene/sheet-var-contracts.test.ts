// Runtime↔sheet var contracts: these custom properties are written by an
// action and read by framework CSS / attributes / engine scales. Both sides
// are framework-owned; this pin keeps them from drifting apart silently.
// If a side moves, update BOTH entries — these are pairs, not independent
// strings.
import { describe, expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'

const ROOT = '.'

const read = (p: string): string => readFileSync(`${ROOT}/${p}`, 'utf8')

const PAIRS: Array<{
  varName: string
  producer: [string, RegExp]
  consumer: [string, RegExp]
}> = [
  {
    // textarea/input grow-to-fit: action measures, base sheet applies.
    varName: '--autosize-h',
    producer: ['packages/sail/runtime/src/actions/autosize.ts', /autosize-h/],
    consumer: ['packages/sail/tokens/src/sail-base.css', /height:\s*var\(--autosize-h/],
  },
  {
    // slider track fill percent: action computes, skin paints the gradient.
    varName: '--rng-pct',
    producer: ['packages/sail/runtime/src/actions/range-fill.ts', /rng-pct/],
    consumer: [
      'packages/sail/tokens/src/element-skins.css',
      /var\(--rng-pct,\s*0%\)/,
    ],
  },
  {
    // virtualized grid column template: action computes, tracks attr reads.
    varName: '--sail-grid-template',
    producer: ['packages/sail/runtime/src/actions/grid-tracks.ts', /sail-grid-template/],
    consumer: [
      'packages/components/sail/primitives/src/sail-tree-grid/sail-tree-grid.sail',
      /tracks="--sail-grid-template"/,
    ],
  },
  {
    // header reveal offset: action measures, fill scale subtracts it.
    varName: '--chrome-offset',
    producer: ['packages/sail/runtime/src/actions/chrome-offset.ts', /chrome-offset/],
    consumer: [
      'crates/sail/spinnaker/src/scales.rs',
      /var\(--chrome-offset/,
    ],
  },
]

describe('sheet-var contracts', () => {
  for (const { varName, producer, consumer } of PAIRS) {
    test(`${varName}: producer and consumer ship together`, () => {
      const [pfile, ppin] = producer
      expect(read(pfile).match(ppin)).not.toBeNull()
      const [cfile, cpin] = consumer
      expect(read(cfile).match(cpin)).not.toBeNull()
    })
  }
})
