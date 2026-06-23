import { controller, signal, derived, effect, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-tree-item.sui'
import type { TreeNode } from './sui-tree-view.controller'

export default controller({
  scope: ({ label }: ViewScope<Scope>) => {
    const node = signal<TreeNode | null>(null)
    const nodeLabel = derived<string>(() => {
      const n = node()
      return n ? n.label : label()
    })
    const nodeChildren = derived<TreeNode[]>(() => {
      const n = node()
      return n?.children ?? []
    })
    const rowEl = ref<HTMLDivElement>(null)
    return { node, nodeLabel, nodeChildren, rowEl }
  },

  mount(state, host) {
    if (!host.hasAttribute('role')) host.setAttribute('role', 'treeitem')

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const initNode = (host as any).node
    if (initNode != null) state.node = initNode
    Object.defineProperty(host, 'node', {
      configurable: true,
      get: () => state.node,
      set: (v: TreeNode | null) => { state.node = v },
    })

    const stopLeafSync = effect(() => {
      const n = state.node
      if (n !== null) {
        const isLeaf = !n.children || n.children.length === 0
        if (isLeaf) host.setAttribute('leaf', '')
        else host.removeAttribute('leaf')
      }
    })

    const syncAria = effect(() => {
      const row = state.rowEl as HTMLDivElement | null
      if (!row) return
      if (!state.leaf) {
        row.setAttribute('aria-expanded', String(state.expanded))
      } else {
        row.removeAttribute('aria-expanded')
      }
      row.setAttribute('aria-selected', String(state.selected))
    })

    const toggle = (e: Event) => {
      if (state.leaf) return
      e.stopPropagation()
      const next = !state.expanded
      state.expanded = next
      host.dispatchEvent(
        new CustomEvent('sui-toggle', {
          detail: { expanded: next },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const select = () => {
      host.dispatchEvent(
        new CustomEvent('sui-select', {
          bubbles: true,
          composed: true,
          detail: {},
        }),
      )
    }

    const onRowClick = (e: MouseEvent) => {
      select()
      toggle(e)
    }

    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        select()
      } else if (e.key === 'ArrowRight' && !state.leaf && !state.expanded) {
        e.preventDefault()
        toggle(e)
      } else if (e.key === 'ArrowLeft' && state.expanded) {
        e.preventDefault()
        toggle(e)
      }
    }

    const stopRow = effect(() => {
      const row = state.rowEl as HTMLDivElement | null
      if (!row) return
      row.addEventListener('click', onRowClick)
      row.addEventListener('keydown', onKey)
      return () => {
        row.removeEventListener('click', onRowClick)
        row.removeEventListener('keydown', onKey)
      }
    })

    return () => {
      stopLeafSync()
      syncAria()
      stopRow()
    }
  },
})
