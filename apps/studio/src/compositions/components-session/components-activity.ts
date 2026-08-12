/**
 * Cross-route activity bag for Components (Library ↔ Inspect remounts).
 * Backed by `persisted` (session storage) so remounts keep selection/filter.
 */
import { persisted } from '@ship/sail-runtime'

import { signal } from '@ship/sail-runtime'
export type ComponentsActivityBag = {
  selectedId: string
  filterQuery: string
}

const empty = (): ComponentsActivityBag => ({ selectedId: '', filterQuery: '' })

const bag = persisted<ComponentsActivityBag>(
  'studio.components-activity',
  empty(),
  { storage: 'session', debounceMs: 50 },
)

export function readComponentsActivity(): ComponentsActivityBag {
  return { ...bag() }
}

export function writeComponentsActivity(patch: Partial<ComponentsActivityBag>): void {
  const cur = bag()
  bag.set({
    selectedId: patch.selectedId !== undefined ? patch.selectedId : cur.selectedId,
    filterQuery:
      patch.filterQuery !== undefined ? patch.filterQuery : cur.filterQuery,
  })
}

/** Test helper — resets the in-memory signal (storage write is debounced). */
export function __clearComponentsActivityForTests(): void {
  bag.set(empty())
}
