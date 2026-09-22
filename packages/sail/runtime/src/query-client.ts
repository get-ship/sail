// query-client — RETIRED as of the native-engine swap (see
// `queried.ts`'s header). Owner-tree homing for the cache now lives in
// `native-cache.ts`'s `useNativeCache()`; this file is kept as a thin
// forward so nothing importing `useQueryClient` from here breaks, but
// new code should import `useNativeCache` directly from
// `./native-cache` — this module is scheduled for removal once
// nothing references it.

export { useNativeCache as useQueryClient } from './native-cache'
