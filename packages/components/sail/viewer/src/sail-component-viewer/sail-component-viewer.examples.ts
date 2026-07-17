// <sail-component-viewer> showcase fixtures — the embeddable composition tag, the
// one tag a host drops in for a live, reactive preview of a SAIL component. The
// Studio components explorer / docs read these.
//
// The viewer's structured inputs (the live `source` signal, the CEM
// `attributes[]`, the bound `compile()`, slots) all arrive via `prop:*`
// identity, NOT scalar attributes (`ComponentExample.props` is scalar-only —
// verified). So each scenario pairs a scalar `props` row (the
// `component-name`/`viewport-width` reflect attrs) with the raw source it
// previews, exported as `sources` keyed by scenario name. A host wires
// `prop:source={sources[name]}` + `prop:compile={compile}` (+ optionally
// `prop:cem`) to bring the viewer alive.

import type { ComponentExample } from '@ship/sail-runtime'

/** Raw `.sail` sources the viewer previews, keyed by the matching example
 *  `name`. The host injects these via `prop:source` (identity) — they can't
 *  ride a scalar attribute. Real Ship-domain content; no placeholders. */
export const sources: Record<string, string> = {
  'Full viewer (with CEM)': `<prop name="variant" values="solid|soft|ghost" default="solid" reflect />
<prop name="tone" values="default|primary" default="default" reflect />
<button class="lane-action">{variant}/{tone}</button>`,
  'Agentic-live (no CEM)': `<div class="lane">
  <strong>gatekeeper</strong> · reviewing <code>job/sail-framework</code>
  <span> — fresh agent source, no compiled CEM yet</span>
</div>`,
  'Narrow viewport': `<div class="card">
  <strong>sail-component-viewer</strong>
  <p>the deepest dogfood — a viewer of primitives, built from primitives</p>
</div>`,
}

export const examples: ComponentExample[] = [
  {
    name: 'Full viewer (with CEM)',
    description:
      'The full composition: live stage + CEM-driven knobs + state matrix + props table + source/snippet, wired in lockstep.',
    props: { 'component-name': 'sail-viewer-demo' },
  },
  {
    name: 'Agentic-live (no CEM)',
    description:
      'The degraded mode — fresh agent source with no compiled CEM yet: source + live render + error surfaces only, knobs/matrix/props-table hidden (not errored).',
    props: {},
  },
  {
    name: 'Narrow viewport',
    description: 'The same composition framed at a mobile viewport width — the stage reflows, theme/density unchanged.',
    props: { 'viewport-width': 'mobile' },
  },
]
