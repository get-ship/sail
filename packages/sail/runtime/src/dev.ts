// `@ship/sail-runtime/dev` — dev-only entry point.
//
// Imports under this subpath stay out of the production runtime
// bundle (sail-runtime.js externalizes only `@ship/sail-runtime`, so
// `/dev` references would error if shipped). The shell + tests
// import from here when they want the leak detector wired in.
//
// HMR lives in Deck, not here: the live-reload client is served at
// `/_deck/live-reload.js`. Named-view hosts use `mount().reload()`;
// SPA `routes/` remounts via `__deckHmr.swapModules` (app-granular).
// The runtime owns the remount/reload *mechanism*; Deck owns the
// transport + the `/_deck/` URLs.

export { enableLeakWarn, enableLeakWarnInScope } from './scope-resource-warn'
