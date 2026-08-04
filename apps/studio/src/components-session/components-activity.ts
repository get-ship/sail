/**
 * Cross-route activity bag for Components (Library ↔ Inspect remounts).
 * In-memory only until a persisted signal primitive lands (DOMDEC gap).
 */
export type ComponentsActivityBag = {
  selectedId: string
  filterQuery: string
}

const empty = (): ComponentsActivityBag => ({ selectedId: '', filterQuery: '' })

let bag: ComponentsActivityBag = empty()

export function readComponentsActivity(): ComponentsActivityBag {
  return { ...bag }
}

export function writeComponentsActivity(patch: Partial<ComponentsActivityBag>): void {
  bag = {
    selectedId: patch.selectedId !== undefined ? patch.selectedId : bag.selectedId,
    filterQuery:
      patch.filterQuery !== undefined ? patch.filterQuery : bag.filterQuery,
  }
}

/** Test helper */
export function __clearComponentsActivityForTests(): void {
  bag = empty()
}
