// `@ship/sail-runtime/dev` — dev-only entry point.
//
// Imports under this subpath stay out of the production runtime
// bundle (sui-runtime.js externalizes only `@ship/sail-runtime`, so
// `/dev` references would error if shipped). The shell + tests
// import from here when they want the leak detector wired in.
//
// HMR lives in Deck, not here: the live-reload client is served at
// `/_deck/live-reload.js` and drives the runtime's `mount().reload()`
// seam. The runtime owns the reload *mechanism*; Deck owns the
// transport + the `/_deck/` URLs.

export { enableLeakWarn, enableLeakWarnInScope } from './scope-resource-warn'
