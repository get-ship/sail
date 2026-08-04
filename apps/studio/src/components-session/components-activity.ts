/**
 * Cross-route activity bag for Components (Library ↔ Inspect remounts).
 * sessionStorage only — not a product listeners store.
 */
export type ComponentsActivityBag = {
  selectedId: string
  filterQuery: string
}

const KEY = 'sail-studio-components-activity'

const empty = (): ComponentsActivityBag => ({ selectedId: '', filterQuery: '' })

export function readComponentsActivity(): ComponentsActivityBag {
  try {
    if (typeof sessionStorage === 'undefined') return empty()
    const raw = sessionStorage.getItem(KEY)
    if (!raw) return empty()
    const o = JSON.parse(raw) as Partial<ComponentsActivityBag>
    return {
      selectedId: typeof o.selectedId === 'string' ? o.selectedId : '',
      filterQuery: typeof o.filterQuery === 'string' ? o.filterQuery : '',
    }
  } catch {
    return empty()
  }
}

export function writeComponentsActivity(patch: Partial<ComponentsActivityBag>): void {
  try {
    if (typeof sessionStorage === 'undefined') return
    const cur = readComponentsActivity()
    const next: ComponentsActivityBag = {
      selectedId: patch.selectedId !== undefined ? patch.selectedId : cur.selectedId,
      filterQuery:
        patch.filterQuery !== undefined ? patch.filterQuery : cur.filterQuery,
    }
    sessionStorage.setItem(KEY, JSON.stringify(next))
  } catch {
    /* private mode / denied */
  }
}

/** Test helper */
export function __clearComponentsActivityForTests(): void {
  try {
    if (typeof sessionStorage !== 'undefined') sessionStorage.removeItem(KEY)
  } catch {
    /* */
  }
}
