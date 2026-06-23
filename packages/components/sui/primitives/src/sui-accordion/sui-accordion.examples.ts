import type { ComponentExample } from '@ship/sui-runtime'

export const examples: ComponentExample[] = [
  {
    name: 'Settings sections',
    description: 'Collapsible groups — the shape Settings uses.',
    slots: `
      <sui-accordion-item value="providers">
        <span slot="trigger">Providers</span>
        <p>claude-opus-4-7 · codex · gemini — 3 CLIs detected on PATH.</p>
      </sui-accordion-item>
      <sui-accordion-item value="permissions">
        <span slot="trigger">Permissions</span>
        <p>Agents may write <code>views/</code> and <code>packages/</code>; never <code>.ship/machine</code>.</p>
      </sui-accordion-item>
      <sui-accordion-item value="danger">
        <span slot="trigger">Danger zone</span>
        <p>Reset the workspace, revoke all sessions, or wipe local snapshots.</p>
      </sui-accordion-item>
    `,
  },
]
