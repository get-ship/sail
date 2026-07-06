import type { ComponentExample } from '@ship/sui-runtime'

export const examples: ComponentExample[] = [
  {
    name: 'Dispatch a pipeline run',
    description: 'The flagship use — kick off ship-it against a worktree.',
    props: { open: true, size: 'lg' },
    slots: `
      <span slot="header">Dispatch <code>ship-it</code> 🚀</span>
      <sui-field label="Spec" hint="What should this run do?">
        <textarea rows="3" placeholder="Wire the components explorer to live examples…"></textarea>
      </sui-field>
      <sui-field label="Provider">
        <sui-select>
          <option>claude-opus-4-7</option>
          <option>claude-sonnet-4-6</option>
        </sui-select>
      </sui-field>
      <div class="toolbar toolbar--footer" slot="footer">
        <button variant="outline">Cancel</button>
        <button variant="solid" tone="primary">Dispatch</button>
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
        <button variant="ghost">Dismiss</button>
        <button variant="solid" tone="primary">Re-dispatch</button>
      </div>
    `,
  },
]
