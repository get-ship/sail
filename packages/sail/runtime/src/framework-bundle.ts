// Framework-bundle entry for `deck build-assets` / the importmap singleton.
//
// Author TypeScript still resolves `@ship/sail-runtime` → `./index.ts`
// (curated public language surface). The browser importmap maps
// `@ship/sail-runtime`, `/internal`, and `/actions` to the same
// `/_deck/runtime.js` URL so the signal graph is never duplicated; that
// shared module must therefore export:
//   - the language surface + ABI (`./internal` re-exports the root and
//     adds emit-only symbols), and
//   - the full `use:` action surface (`./actions`).
//
// Do not import this file from application source — it exists only as the
// framework singleton entry.

export * from './internal'
export * from './actions'
