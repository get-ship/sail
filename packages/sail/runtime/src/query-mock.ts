// Test-only query seam. Not on the product singleton (`/internal`).
// Authors get this through `@ship/sail-test`, not `@ship/sail-runtime`.

export { mockQueryState, clearQueryMocks } from './native-cache'
export type { QueryKey } from './native-cache'
export type { LiveState } from './live-state'
