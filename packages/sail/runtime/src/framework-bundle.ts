// Framework-bundle entry for `deck build-assets` / the importmap singleton.
//
// Author TypeScript still resolves `@ship/sail-runtime` → `./index.ts`
// (curated public language surface). The browser importmap maps
// `@ship/sail-runtime`, `/internal`, `/actions`, and `/virtual` to the same
// `/_deck/runtime.js` URL so the signal graph is never duplicated; that
// shared module must therefore export:
//   - the language surface + ABI (`./internal` re-exports the root and
//     adds emit-only symbols),
//   - the full `use:` action surface (`./actions`), and
//   - `createVirtualizer` (`./virtual`) — used by sail-table / sail-tree /
//     sail-message-list; off the curated root barrel but on the importmap.
//
// Do not import this file from application source — it exists only as the
// framework singleton entry.

export * from './internal'
export * from './actions'
export * from './virtual'
