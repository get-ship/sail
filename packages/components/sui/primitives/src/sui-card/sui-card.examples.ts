import type { ComponentExample } from '@ship/sui-runtime'

export const examples: ComponentExample[] = [
  {
    name: 'Agent at work',
    description: 'A live agent card — the kind the activity monitor renders.',
    slots: `
      <span slot="header">gatekeeper · <span tone="info" sx="display:inline-flex min-h:control-h-2xs px:2 text:xs weight:semibold radius:sm bg:tone-bg border:tone-border fg:tone-fg">reviewing</span></span>
      <p><code>job/sui-framework</code> — 14 files, 3 findings so far. Burned 41k tokens.</p>
      <div class="toolbar toolbar--footer" slot="footer">
        <button variant="ghost" size="sm">Open review</button>
        <button variant="outline" size="sm" tone="destructive">Stop</button>
      </div>
    `,
  },
  {
    name: 'Pipeline summary',
    description: 'Stat-led card for a pipeline definition.',
    slots: `
      <span slot="header">ship-it</span>
      <p>Plan → build → gate → merge. <strong>87%</strong> success across 23 runs; last run passed 6m ago.</p>
      <div class="toolbar toolbar--footer" slot="footer">
        <button variant="solid" tone="primary" size="sm">Dispatch</button>
      </div>
    `,
  },
]
