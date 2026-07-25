// Framework-bundle entry for `deck build-assets` / the importmap singleton.
//
// Author TypeScript still resolves `@ship/sail-runtime` → `./index.ts`
// (curated public surface). The browser importmap maps both
// `@ship/sail-runtime` and `@ship/sail-runtime/actions` to the same
// `/_deck/runtime.js` URL so the signal graph is never duplicated; that
// shared module must therefore export the full `use:` action surface the
// compiler auto-imports from `/actions`, not only the curated front door.
//
// Do not import this file from application source — it exists only as the
// framework singleton entry.

export * from './index'
export * from './actions'
