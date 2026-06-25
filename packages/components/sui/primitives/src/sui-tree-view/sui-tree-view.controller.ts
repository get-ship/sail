import { controller, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-tree-view.sui'

export interface TreeNode {
  id: string
  label: string
  /** Optional per-node `<sui-icon>` name — lets a file tree show a glyph per
   *  file type. Rendered by `<sui-tree>` ahead of the label; falls back to the
   *  `node-icon` slot when unset. */
  icon?: string
  children?: TreeNode[]
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const nodes = signal<TreeNode[]>([])
    return { nodes }
  },

  mount(state, host) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const initNodes = (host as any).nodes
    if (initNodes != null) state.nodes = initNodes

    Object.defineProperty(host, 'nodes', {
      configurable: true,
      get: () => state.nodes,
      set: (v: TreeNode[]) => { state.nodes = v },
    })

    if (!host.hasAttribute('role')) host.setAttribute('role', 'tree')
  },
})
