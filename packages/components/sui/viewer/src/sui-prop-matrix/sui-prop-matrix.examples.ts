// <sui-prop-matrix> showcase fixtures — the Storybook-style state sweep over a
// target's enum cross-product. The Studio components explorer / docs read these.
//
// The matrix derives axes from a CEM `attributes[]` array and renders one live
// `<sui-render>` per cell. Its target data (the CEM `props`, the bound
// `compile()`, the component-mode `source`, the `active` knob values, the slot
// markup) arrives via `prop:*` identity, NOT scalar attributes
// (`ComponentExample.props` is scalar-only — verified). So each scenario pairs a
// scalar `props` row (the `component-name` reflect attr) with the CEM + source
// it sweeps, exported as `cems` / `sources` keyed by scenario name. A host wires
// `prop:props={cems[name]}` + `prop:source={sources[name]}` + `prop:compile={fn}`
// to bring the grid alive.

import type { ComponentExample } from '@ship/sui-runtime'
import type { CemAttr } from './matrix'

/** CEM `attributes[]` per scenario — the enum axes the matrix sweeps. Injected
 *  via `prop:props` (identity); they cannot ride a scalar attribute. */
export const cems: Record<string, CemAttr[]> = {
  'Variant × tone': [
    { name: 'variant', type: { text: "'solid' | 'soft' | 'ghost'" }, default: 'solid', values: ['solid', 'soft', 'ghost'] },
    { name: 'tone', type: { text: "'default' | 'primary' | 'destructive'" }, default: 'default', values: ['default', 'primary', 'destructive'] },
    { name: 'label', type: { text: 'string' }, default: '' },
  ],
  'Status tone sweep': [
    { name: 'tone', type: { text: "'info' | 'success' | 'warning' | 'destructive'" }, default: 'info', values: ['info', 'success', 'warning', 'destructive'] },
  ],
}

/** Component-mode `.sui` sources per scenario — the specimen each cell compiles
 *  + instantiates, reflecting that cell's axis props. Injected via `prop:source`
 *  (identity). Real, characterful Ship-domain content; no placeholders. */
export const sources: Record<string, string> = {
  'Variant × tone': `<prop name="variant" values="solid|soft|ghost" default="solid" reflect />
<prop name="tone" values="default|primary|destructive" default="default" reflect />
<button class="lane-action">retry lane</button>`,
  'Status tone sweep': `<prop name="tone" values="info|success|warning|destructive" default="info" reflect />
<span class="run-state">gate {tone}</span>`,
}

export const examples: ComponentExample[] = [
  {
    name: 'Variant × tone',
    description: 'The full variant × tone cross-product of a lane-action control — every state at a glance.',
    props: { 'component-name': 'sui-lane-action' },
  },
  {
    name: 'Status tone sweep',
    description: 'A single tone axis sweeping a run-state badge across info/success/warning/destructive.',
    props: { 'component-name': 'sui-run-state' },
  },
]
