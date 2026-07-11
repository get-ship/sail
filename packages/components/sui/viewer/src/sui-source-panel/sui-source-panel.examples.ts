// <sui-source-panel> showcase fixtures — real Ship-domain targets the panel
// shows source + a paste-able snippet for. The Studio components explorer /
// docs read these.
//
// `sui-source-panel`'s structured inputs (the raw `source`, the CEM `props`,
// the current `value`, the composed `slotContent`) arrive via `prop:*`
// identity/snapshot, NOT scalar attributes (`ComponentExample.props` is
// scalar-only — verified). So each scenario pairs a scalar `props` row (the
// `component-name`/`lang` reflect attrs) with the raw source + current values
// it diffs, exported keyed by scenario name. A host wires `prop:source` +
// `prop:props` + `prop:value` to bring the panel alive.

import type { ComponentExample } from '@ship/sui-runtime'

/** Raw `.sui` sources shown in the source pane, keyed by example `name`. */
export const sources: Record<string, string> = {
  'Button — solid primary': `<button variant="solid" tone="primary">Dispatch lane</button>`,
  'Alert — destructive': `<alert tone="destructive">Gate failed: 3 findings block the merge.</alert>`,
  'Chip — removable': `<chip removable tone="primary">TypeScript</chip>`,
}

/** Current knob `value` snapshots the snippet diffs against the CEM default. */
export const values: Record<string, Record<string, unknown>> = {
  'Button — solid primary': { variant: 'solid', tone: 'primary', size: 'md' },
  'Alert — destructive': { tone: 'destructive' },
  'Chip — removable': { removable: true, tone: 'primary' },
}

/** Composed slot content for each snippet's inner. */
export const slots: Record<string, string> = {
  'Button — solid primary': 'Dispatch lane',
  'Alert — destructive': 'Gate failed: 3 findings block the merge.',
  'Chip — removable': 'TypeScript',
}

export const examples: ComponentExample[] = [
  {
    name: 'Button — solid primary',
    description: 'Source + a snippet that emits only the non-default props (size=md is diffed away).',
    props: { 'component-name': 'button', lang: 'html' },
  },
  {
    name: 'Alert — destructive',
    description: 'A status alert; the snippet carries just the tone that differs from default.',
    props: { 'component-name': 'sui-field', lang: 'html' },
  },
  {
    name: 'Chip — removable',
    description: 'A bool prop (removable) renders as a bare attribute in the recomputed snippet.',
    props: { 'component-name': 'sui-chip', lang: 'html' },
  },
]
