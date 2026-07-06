import type { ComponentExample } from '@ship/sui-runtime'

export const examples: ComponentExample[] = [
  {
    name: 'Quick-dispatch popover',
    description: 'Anchored panel hung off a trigger button.',
    slots: `
      <button slot="trigger" variant="outline" size="sm">Dispatch ▾</button>
      <column gap="2">
        <strong>Run ship-it on…</strong>
        <button variant="ghost" size="sm">current worktree</button>
        <button variant="ghost" size="sm">a fresh branch</button>
        <button variant="ghost" size="sm">last failed run</button>
      </column>
    `,
  },
]
