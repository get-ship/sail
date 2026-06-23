import type { ComponentExample } from '@ship/sui-runtime'

export const examples: ComponentExample[] = [
  {
    name: 'Quick-dispatch popover',
    description: 'Anchored panel hung off a trigger button.',
    slots: `
      <sui-button slot="trigger" variant="outline" size="sm">Dispatch ▾</sui-button>
      <sui-stack gap="2">
        <strong>Run ship-it on…</strong>
        <sui-button variant="ghost" size="sm">current worktree</sui-button>
        <sui-button variant="ghost" size="sm">a fresh branch</sui-button>
        <sui-button variant="ghost" size="sm">last failed run</sui-button>
      </sui-stack>
    `,
  },
]
