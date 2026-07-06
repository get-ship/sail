import type { ComponentExample } from '@ship/sui-runtime'

export const examples: ComponentExample[] = [
  {
    name: 'Delete a worktree with unsaved work',
    description: 'The destructive-confirm pattern — the reason this exists.',
    props: { open: true, tone: 'destructive' },
    slots: `
      <span slot="title">Delete worktree <code>sui-framework</code>?</span>
      <p>17 uncommitted changes across 6 files will be lost. The branch
      <code>job/sui-framework</code> is not merged. This cannot be undone.</p>
      <div class="toolbar toolbar--footer" slot="footer">
        <button variant="outline">Keep it</button>
        <button variant="solid" tone="destructive">Delete worktree</button>
      </div>
    `,
  },
]
