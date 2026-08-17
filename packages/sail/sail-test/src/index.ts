/**
 * @ship/sail-test — runtime-agnostic Sail testing helpers.
 *
 * Semantic queries use role / accessible name / ARIA state as stamped by
 * ARIA-1 (see /sail/reference/semantics). DOM structure and data-* stamps
 * are not the app-test API.
 *
 * Pair with any runner. `@ship/test-setup` remains a vitest/jsdom adapter
 * (renderRoute/compile) and may consume these helpers; it is not the public
 * identity of Sail testing.
 *
 * @packageDocumentation
 */

export {
  getByRole,
  queryByRole,
  getAllByRole,
  getByText,
  queryByText,
  getRole,
  accessibleName,
  click,
  type ByRoleOptions,
} from './queries'

export {
  installViewport,
  forceBreakpoint,
  BREAKPOINT_MIN_WIDTH,
  type BreakpointToken,
} from './viewport'

export { snapshotSail, type SnapshotResult } from './snapshot'

export {
  contractQuery,
  getByContract,
  isLeanTestMap,
  queryContract,
  type TestEntry,
  type TestMap,
} from './contract'

// Runtime-owned mock seam (owner-tree cache). Re-exported for one import surface.
export {
  mockQueryState,
  clearQueryMocks,
  type QueryKey,
  type LiveState,
} from '@ship/sail-runtime/query-mock'
