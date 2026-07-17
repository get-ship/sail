// <sui-render> showcase fixtures — real Ship-domain `.sui` specimens the
// engine mounts live. The Studio components explorer / docs read these.
//
// `sui-render` is the engine primitive: its target data (the raw `.sui`
// `source`, the bound `compile()`) arrives via `prop:*` identity, NOT scalar
// attributes (`ComponentExample.props` is scalar-only — verified). So each
// scenario pairs a scalar `props` row (the `mode`/`viewport-width` reflect
// attrs) with the raw view-mode source it renders, exported as `sources` keyed
// by scenario name. A host wires `prop:source={sources[name]}` +
// `prop:compile={compile}` to bring the specimen alive.

import type { ComponentExample } from '@ship/sail-runtime'

/** Raw view-mode `.sui` sources, keyed by the matching example `name`. The
 *  host injects these via `prop:source` (identity) — they can't ride a scalar
 *  attribute. Real, characterful Ship-domain content; no placeholders. */
export const sources: Record<string, string> = {
  'Agent lane status': `<div class="lane">
  <strong>gatekeeper</strong> · reviewing <code>job/sui-framework</code>
  <span> — 14 files, 3 findings, 41k tokens burned</span>
</div>`,
  'Pipeline run row': `<div class="run">
  <strong>ship-it</strong> · plan → build → gate → merge
  <span> — 87% across 23 runs; last passed 6m ago</span>
</div>`,
  'Worktree narrow preview': `<div class="wt">
  <strong>sui-viewer</strong>
  <p>packages/components/sui/viewer · 7 files changed</p>
</div>`,
}

export const examples: ComponentExample[] = [
  {
    name: 'Agent lane status',
    description: 'A live view-mode render of an agent lane line — the agentic preview loop.',
    props: { mode: 'source' },
  },
  {
    name: 'Pipeline run row',
    description: 'A pipeline summary rendered live; flip the source and it re-mounts.',
    props: { mode: 'source' },
  },
  {
    name: 'Worktree narrow preview',
    description: 'The same specimen framed at a mobile viewport width — reflow without theme change.',
    props: { mode: 'source', 'viewport-width': 'mobile' },
  },
]
