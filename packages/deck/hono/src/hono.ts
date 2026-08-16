// Hono binding for the deck adaptor. Mount it on an existing Hono app; deck
// claims its own routes (`/_deck/*`, `/api` tokens/themes, the app asset
// namespace, the SPA shell + fallback) and defers everything else (a `null`
// from the router) to the next handler. Live-data SSE stays host-owned
// (`config.live`).
//
// `hono` is an optional peer dep; this module is only imported when the host
// already has Hono. The types are imported type-only to avoid a hard dep.

import type { Context, MiddlewareHandler } from "hono";
import { createDeckRouter, type DeckAdaptorConfig } from "./router";

/**
 * Returns Hono middleware that serves deck-owned routes and calls `next()` for
 * everything else. Register host backend routes (or list them on
 * `hostBackendPrefixes`) so they are not SPA-fallback'd; register this
 * middleware before the host's own catch-all/404. Deck owns `/_deck` and
 * `/api` (tokens/themes) — unmatched paths under those prefixes 404 rather
 * than falling through as a client route.
 */
export function deckMiddleware(config: DeckAdaptorConfig): MiddlewareHandler {
  const handle = createDeckRouter(config);
  return async (c: Context, next) => {
    const res = await handle(c.req.raw);
    if (res) return res;
    await next();
  };
}

export { createDeckRouter, type DeckAdaptorConfig };
