// Framework-bundle entry for `deck build-assets` / the importmap singleton.
//
// Author TypeScript still resolves `@ship/sail-runtime` → `./index.ts`
// (curated public language surface). The browser importmap maps
// `@ship/sail-runtime` and `/internal` to `/_deck/runtime.js` so the
// signal graph is never duplicated.
//
// Batteries are their own singletons. The compiler / ambient inject
// imports them only when the page uses them:
//   `use:` actions     → `/_deck/actions.js`  `@ship/sail-runtime/actions`
//   `fx:` / timeline   → `/_deck/motion.js`   `@ship/sail-runtime/motion`
//   `query`/`mutation` → `/_deck/query.js`    `@ship/sail-runtime/query`
//   `createVirtualizer`→ `/_deck/virtual.js`  `@ship/sail-runtime/virtual`
//
// Do not import this file from application source — it exists only as the
// framework singleton entry.

export * from './internal'
