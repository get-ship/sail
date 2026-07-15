// Nothing. The docs are built from L1 — the elements are in the language,
// and deck auto-registers any pack custom element a template actually uses
// (that is how <sui-component-viewer>, <sui-code-editor> and <sui-diff> upgrade
// on their routes with this file empty). So a catalog-wide side-effect import
// is not needed to make components render.
//
// It is also currently BLOCKED. Re-populating this with the pack imports
// (`@ship/sui-primitives`, `@ship/sui-charts`, …) fails the app build with:
//
//   `@ship/sui-runtime/virtual` is a framework-singleton subpath with no
//   importmap entry — bundling it inline would duplicate the singleton.
//   Add it to FRAMEWORK_ASSETS in deck-bundler/src/app.rs.
//
// The virtualizer subpath is pulled in by @ship/sui-primitives (sui-tree,
// sui-tree-grid, sui-table) and @ship/sui-chat-elements (sui-message-list).
// This is a deck-bundler importmap gap, NOT the floor rejecting pack styles —
// those .ts packs carry no raw <style>. It resolves in the deck lane
// (deck-bundler `FRAMEWORK_ASSETS`), after which this module can register the
// full catalog if a route ever needs a pack element it does not already use.
export {}
