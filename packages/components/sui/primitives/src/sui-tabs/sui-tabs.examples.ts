import type { ComponentExample } from '@ship/sui-runtime'

export const examples: ComponentExample[] = [
  {
    name: 'Session inspector tabs',
    description: 'The tabset the session view uses to switch panes.',
    slots: `
      <sui-tab name="thread" active>Thread</sui-tab>
      <sui-tab name="diffs">Diffs <sui-badge tone="info" size="sm">14</sui-badge></sui-tab>
      <sui-tab name="logs">Logs</sui-tab>
      <sui-tab name="cost">Cost · $2.41</sui-tab>
    `,
  },
]
