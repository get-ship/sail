import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

interface Link { kind: string; href: string; label: string }

export default controller<Scope>({
  scope: () => {
    const title = signal('What next?')
    const links = signal<Link[]>([
      { kind: 'cookbook',  href: '/sui/cookbook/',                  label: 'patterns & recipes' },
      { kind: 'reference', href: '/sui/template/',                  label: 'full template surface' },
      { kind: 'reference', href: '/sui/controller/',                label: 'controller contract' },
      { kind: 'reference', href: '/sui/hooks/',                     label: 'template + controller hooks' },
      { kind: 'reference', href: '/sui/data-subscriptions/',        label: 'live data + envelopes' },
      { kind: 'agents',    href: '/sui/agent-surfaces/',            label: 'MCP / LSP / sui-eval' },
      { kind: 'examples',  href: '/sui/examples/',                  label: 'canonical view shapes' },
    ])
    return { title, links }
  },
})
