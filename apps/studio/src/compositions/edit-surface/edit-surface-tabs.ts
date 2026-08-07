/** Tab open/close helpers for edit-surface doc strip.
 * L1 `<tabs>` lacks dirty/close/middle-click for dynamic open files →
 * custom strip (absorption vote: studio-tabs / L2 candidate).
 */
import { closeTab, getShellState, setActiveTab, type DocTab } from '@ship/sail-studio-shell'
import { openDocument } from '@ship/studio/edit'

export type TabRow = DocTab & {
  active: boolean
  dirtyDot: string
  canClose: boolean
  showClose: boolean
  key: string
}

/**
 * Path-aware tab label: bare basename when unique among open tabs;
 * otherwise parent/basename (e.g. edit/page.sail) so multi page.sail is legible.
 */
export function pathAwareTabLabel(path: string, openIds: readonly string[]): string {
  const norm = path.replace(/\\/g, '/')
  const base = norm.split('/').pop() || norm
  const sameBase = openIds.filter((id) => {
    const p = id.replace(/\\/g, '/')
    return (p.split('/').pop() || p) === base
  })
  if (sameBase.length <= 1) return base
  const parts = norm.split('/').filter(Boolean)
  if (parts.length >= 2) return `${parts[parts.length - 2]}/${parts[parts.length - 1]}`
  return base
}

/** Map shell/edit open docs into strip rows: label/dirty/close/active. */
export function mapDocTabs(
  tabs: DocTab[],
  activeId: string | null,
  dirtyById?: ReadonlyMap<string, boolean> | Set<string>,
): TabRow[] {
  const isDirty = (id: string, shellDirty: boolean): boolean => {
    if (shellDirty) return true
    if (!dirtyById) return false
    if (dirtyById instanceof Set) return dirtyById.has(id)
    return !!dirtyById.get(id)
  }
  const canClose = tabs.length > 1
  const openIds = tabs.map((t) => t.id)
  return tabs.map((t) => {
    const active = t.id === activeId
    const dirty = isDirty(t.id, t.dirty)
    return {
      ...t,
      label: pathAwareTabLabel(t.id, openIds),
      dirty,
      active,
      dirtyDot: dirty ? '●' : '',
      canClose,
      // Always show close when multi-tab (discoverable; not hover-only).
      showClose: canClose,
      key: t.id,
    }
  })
}

/** Prefer real .sail open tabs; drop shell placeholder; fall back to active path. */
export function resolveDocTabs(
  shellTabs: DocTab[],
  activeTabId: string | null,
  activePath: string | null,
  activeDirty: boolean,
): DocTab[] {
  const real = shellTabs.filter((t) => t.id.endsWith('.sail') || t.id !== 'shell')
  if (real.length) return real
  if (!activePath) return []
  return [
    {
      id: activePath,
      label: activePath.split('/').pop() ?? activePath,
      dirty: activeDirty,
    },
  ]
}

export function tabIdFromEvent(e: Event): string | null {
  const el = (e.target as Element | null)?.closest?.('[tab-id]') as HTMLElement | null
  return el?.getAttribute('tab-id') ?? null
}

export function isMiddleButton(e: Event): boolean {
  return (e as MouseEvent).button === 1
}

export function closeDocTab(closedId: string): string | null {
  const wasActive = getShellState().activeTabId === closedId
  closeTab(closedId)
  return wasActive ? getShellState().activeTabId : null
}

export async function openShellTabDocument(id: string): Promise<void> {
  setActiveTab(id)
  if (!id.endsWith('.sail')) return
  const s = getShellState()
  await openDocument(id, { loadPreview: false, theme: s.themeId, mode: s.modeId })
}
