// @ship/deck-hono — deck transport adaptor for Hono / Bun.
//
// The serve core is framework-agnostic: `createDeckRouter(config)` returns a
// `(Request) => Promise<Response | null>` handler. Use it directly with
// `Bun.serve`, or use the Hono binding (`@ship/deck-hono/hono`) to mount it as
// middleware alongside an existing Hono app (ship-ops's /api, /mcp, /events).

export { createDeckRouter, type DeckAdaptorConfig } from "./router";
export {
  type DaemonMode,
  type CachePolicy,
  type DeckResponse,
  intoResponse,
  htmlView,
  jsAsset,
  jsView,
  cssAsset,
  cssNoCache,
  svgAsset,
  jsonAsset,
  jsonNoCache,
  notFound,
  renderError,
  redirect,
  sse,
  HTML_SECURITY_HEADERS,
} from "./response";
