// Hono binding for the deck adaptor. Mount it on an existing Hono app; deck
// claims its own routes (/_deck/*, the app asset namespace, the SPA shell +
// fallback) and defers everything else (a `null` from the router) to the next
// handler — so the host keeps /api, /mcp, /events, and all mutation routes.
//
// `hono` is an optional peer dep; this module is only imported when the host
// already has Hono. The types are imported type-only to avoid a hard dep.

import type { Context, MiddlewareHandler } from "hono";
import { createDeckRouter, type DeckAdaptorConfig } from "./router";

/**
 * Returns Hono middleware that serves deck-owned routes and calls `next()` for
 * everything else. Register it BEFORE the host's own catch-all/404 but its
 * deferral means ordering against /api etc. is forgiving (deck never claims
 * reserved prefixes like /api or /_deck/{sse,data} unless `config.live` does).
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
