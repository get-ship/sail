/**
 * Element tree VM — hierarchy rows (depth + tag + L1 badge) + keyboard nav.
 * Click/Enter selects node and scrolls source to span.
 */
import { mountAmbient, signal } from '@ship/sail-runtime'
import {
  getEditState,
  nextTreeFocus,
  scrollSourceEditor,
  selectEditNode,
  subscribeEdit,
  uiElementRows,
} from '@ship/studio/edit'

function elDomId(id: string): string {
  return `el-${id.replace(/[^a-zA-Z0-9_-]+/g, '-')}`
}

type ElRowUi = {
  id: string
  tag: string
  label: string
  layerBadge: string
  selected: boolean
  focused: boolean
  /** Indent marker for hierarchy (token steps as spaces; ban-clean). */
  pad: string
  depth: number
  tabIndex: number
  domId: string
  key: string
}

export function createEditElementTree() {
  const elementRows = signal<ElRowUi[]>([])
  const hasElements = signal(false)
  const activePathLabel = signal('—')
  const activeDescendant = signal('')
  let kbdId: string | null = null

  const sync = () => {
    const es = getEditState()
    const raw = uiElementRows()
    if (!kbdId || !raw.some((r) => r.id === kbdId)) {
      kbdId = es.selectionNodeId ?? raw[0]?.id ?? null
    }
    const rows = raw.map((r) => {
      const focused = r.id === kbdId
      return {
        id: r.id,
        tag: r.tag,
        label: r.label,
        layerBadge: r.layerBadge,
        selected: r.selected,
        focused,
        pad: '\u00a0\u00a0'.repeat(Math.min(r.depth, 12)),
        depth: r.depth,
        tabIndex: focused ? 0 : -1,
        domId: elDomId(r.id),
        key: r.id,
      }
    })
    elementRows.set(rows)
    hasElements.set(rows.length > 0)
    activePathLabel.set(es.activePath?.split('/').pop() ?? '—')
    activeDescendant.set(kbdId ? elDomId(kbdId) : '')
  }

  sync()

  mountAmbient(() => {
    const unsub = subscribeEdit(() => sync())
    return () => unsub()
  })

  const selectAndScroll = (id: string) => {
    kbdId = id
    selectEditNode(id)
    sync()
    const ta =
      typeof document !== 'undefined'
        ? (document.querySelector('[data-source-editor]') as HTMLTextAreaElement | null)
        : null
    scrollSourceEditor(ta)
  }

  const onElementClick = (e: Event) => {
    const t = (e.target as Element | null)?.closest?.('[data-node-id]') as HTMLElement | null
    if (!t?.dataset.nodeId) return
    selectAndScroll(t.dataset.nodeId)
  }

  const onTreeKeydown = (e: Event) => {
    const ke = e as KeyboardEvent
    const ids = elementRows().map((r) => r.id)
    if (!ids.length) return
    if (ke.key === 'Enter' || ke.key === ' ') {
      ke.preventDefault()
      if (kbdId) selectAndScroll(kbdId)
      return
    }
    const next = nextTreeFocus(ids, kbdId, ke.key)
    if (
      next &&
      next !== kbdId &&
      (ke.key === 'ArrowDown' || ke.key === 'ArrowUp' || ke.key === 'Home' || ke.key === 'End')
    ) {
      ke.preventDefault()
      kbdId = next
      sync()
      if (typeof document !== 'undefined') {
        const id = elDomId(next)
        queueMicrotask(() => {
          const el = document.getElementById(id) as HTMLElement | null
          el?.focus?.()
        })
      }
    } else if (
      ke.key === 'ArrowDown' ||
      ke.key === 'ArrowUp' ||
      ke.key === 'Home' ||
      ke.key === 'End'
    ) {
      ke.preventDefault()
    }
  }

  return {
    elementRows,
    hasElements,
    activePathLabel,
    activeDescendant,
    onElementClick,
    onTreeKeydown,
  }
}
