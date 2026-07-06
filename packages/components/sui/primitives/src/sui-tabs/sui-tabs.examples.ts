import type { ComponentExample } from '@ship/sui-runtime'

export const examples: ComponentExample[] = [
  {
    name: 'Session inspector tabs',
    description: 'The tabset the session view uses to switch panes.',
    slots: `
      <sui-tab name="thread" active>Thread</sui-tab>
      <sui-tab name="diffs">Diffs <span tone="info" sx="display:inline-flex min-h:control-h-2xs px:1_5 text:xs weight:semibold radius:sm bg:tone-bg border:tone-border fg:tone-fg">14</span></sui-tab>
      <sui-tab name="logs">Logs</sui-tab>
      <sui-tab name="cost">Cost · $2.41</sui-tab>
    `,
  },
]
