import type { ComponentExample } from '@ship/sui-runtime'

export const examples: ComponentExample[] = [
  {
    name: 'Dispatch a pipeline run',
    description: 'The flagship use — kick off ship-it against a worktree.',
    props: { open: true, size: 'lg' },
    slots: `
      <span slot="header">Dispatch <code>ship-it</code> 🚀</span>
      <sui-field label="Spec" hint="What should this run do?">
        <sui-textarea rows="3" placeholder="Wire the components explorer to live examples…"></sui-textarea>
      </sui-field>
      <sui-field label="Provider">
        <sui-select>
          <option>claude-opus-4-7</option>
          <option>claude-sonnet-4-6</option>
        </sui-select>
      </sui-field>
      <div class="toolbar toolbar--footer" slot="footer">
        <sui-button variant="outline">Cancel</sui-button>
        <sui-button tone="primary" variant="solid">Dispatch</sui-button>
      </div>
    `,
  },
  {
    name: 'Session ended mid-thought',
    description: 'A terse, single-action dialog.',
    props: { open: true },
    slots: `
      <span slot="header">gatekeeper went quiet</span>
      <p>The session for <code>job/sui-framework</code> ended after 41 minutes with no <code>step_done</code>. Re-dispatch from the last snapshot?</p>
      <div class="toolbar toolbar--footer" slot="footer">
        <sui-button variant="ghost">Dismiss</sui-button>
        <sui-button tone="primary" variant="solid">Re-dispatch</sui-button>
      </div>
    `,
  },
]
