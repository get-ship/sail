import type { ComponentExample } from '@ship/sui-runtime'

export const examples: ComponentExample[] = [
  {
    name: 'Worktree row actions',
    description: 'The overflow menu on a worktree list row.',
    slots: `
      <sui-button slot="trigger" variant="ghost" size="sm">⋯</sui-button>
      <button role="menuitem" data-value="open">Open in terminal</button>
      <button role="menuitem" data-value="rename">Rename branch</button>
      <button role="menuitem" data-value="snapshot">Snapshot now</button>
      <button role="menuitem" data-value="delete">Delete worktree</button>
    `,
  },
]
