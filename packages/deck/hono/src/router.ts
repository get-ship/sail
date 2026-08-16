// The deck serve router — a framework-agnostic `Request -> Response | null`
// handler. It mirrors the GET product-serve surface of `deck-axum` (see
// `crates/deck/deck-axum/src/lib.rs` `router()` and `pages.rs` `spa_fallback`)
// for an app built with `deck build` (the SPA model): `/_deck/*` framework
// singletons, `/api` token + theme CSS, hashed assets, the `index.html` shell,
// and the SPA fallback. A `null` return means "not a deck route" — the host
// (any Hono / Bun app) owns it.
//
// Live-data SSE is host-owned. Inject it via `config.live`; Deck does not
// special-case `views/<name>/sse`.

import { readFile } from "node:fs/promises";
import { join, normalize, sep } from "node:path";
import {
  type DaemonMode,
  type DeckResponse,
  cssAsset,
  cssNoCache,
  html_route,
  intoResponse,
  jsAsset,
  jsonAsset,
  jsonNoCache,
  notFound,
  svgAsset,
} from "./response";

export interface DeckAdaptorConfig {
  /** The `deck build` output dir (`<project>/dist`): index.html + assets/. */
  buildDir: string;
  /**
   * Baked framework singletons (deck-core/assets/): runtime/dom/… plus
   * live-reload.js and error-overlay.js when this adaptor serves dev assets.
   */
  frameworkAssetsDir: string;
  /** package.json `name`; the URL asset namespace is its last '/'-segment. */
  appName: string;
  /** development (default) gates cache-control, ETag, and error redaction. */
  mode?: DaemonMode;
  /**
   * Host-owned backend prefixes. Unmatched GETs under these are NOT given
   * the SPA shell — same as deck-axum `host_backend_prefixes`. Default: none
   * (the standalone default). Deck does not hardcode product prefixes
   * (`/auth`, `/hooks`, `/daemon`, `/space`, `/shell`); the host injects
   * what it owns.
   */
  hostBackendPrefixes?: string[];
  /**
   * Optional live-data handler. Given a Request whose path is under `/_deck`
   * and not a static framework asset, it may return a Response (e.g. an SSE
   * stream) or null to fall through to an honest 404. Wired by the host to
   * its data port.
   */
  live?: (req: Request, mode: DaemonMode) => Promise<Response | null> | Response | null;
}

// Framework singletons: /_deck/<route> -> file name under frameworkAssetsDir
// (deck-core/assets bake) OR, when missing, under <buildDir>/_deck/ for
// self-contained app builds (`--bundle-framework`).
const FRAMEWORK_JS: Record<string, { bake: string; bundled: string }> = {
  "/_deck/runtime.js": { bake: "sail-runtime.js", bundled: "runtime.js" },
  "/_deck/dom.js": { bake: "sail-dom.js", bundled: "dom.js" },
  "/_deck/route.js": { bake: "deck-route.js", bundled: "route.js" },
  "/_deck/sdk-client.js": { bake: "sail-sdk-client.js", bundled: "sdk-client.js" },
  "/_deck/live-reload.js": { bake: "live-reload.js", bundled: "live-reload.js" },
  "/_deck/error-overlay.js": { bake: "error-overlay.js", bundled: "error-overlay.js" },
  "/_deck/compiler-wasm.js": { bake: "sail-compiler-wasm.js", bundled: "compiler-wasm.js" },
  "/_deck/lsp-completions.js": { bake: "sail-lsp-completions.js", bundled: "lsp-completions.js" },
  "/_deck/lsp-diagnostics.js": { bake: "sail-lsp-diagnostics.js", bundled: "lsp-diagnostics.js" },
};

// Deck-owned `/api` token + theme surface (pages.rs + deck-axum router()).
const ROUTE_VIEW_TOKENS_CSS = "/api/assets/view-tokens.css";
const ROUTE_SAIL_CSS = "/api/assets/sail.css";
const ROUTE_THEME_CSS = "/api/assets/theme.css";
const ROUTE_THEMES_LIST = "/api/v1/themes";

function safeResolve(base: string, rel: string): string | null {
  const full = normalize(join(base, rel));
  const root = normalize(base);
  if (full !== root && !full.startsWith(root + sep)) return null; // traversal escape
  return full;
}

async function readOpt(path: string): Promise<Uint8Array | null> {
  try {
    const buf = await readFile(path);
    return new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength);
  } catch {
    return null;
  }
}

function assetByExt(file: string, body: Uint8Array): DeckResponse {
  if (file.endsWith(".js")) return jsAsset(body);
  if (file.endsWith(".css")) return cssAsset(body);
  if (file.endsWith(".svg")) return svgAsset(body);
  if (file.endsWith(".map") || file.endsWith(".json")) return jsonAsset(JSON.parse(new TextDecoder().decode(body)));
  return { status: 200, contentType: "application/octet-stream", cachePolicy: "asset", body, extraHeaders: [] };
}

/** SSOT: `deck-axum` `pages.rs` `spa_fallback`. */
function deckOwnsPath(path: string): boolean {
  return path.startsWith("/api") || path.startsWith("/_deck") || path.includes(".");
}

function hostOwnsPath(path: string, prefixes: readonly string[]): boolean {
  return prefixes.some((p) => path.startsWith(p));
}

/**
 * Build the pure deck serve handler. Returns a function that resolves a Request
 * to a Response for deck-owned routes, or `null` to defer to the host.
 */
export function createDeckRouter(config: DeckAdaptorConfig) {
  const mode: DaemonMode = config.mode ?? "development";
  const ns = config.appName.split("/").pop()!;
  const assetsDir = join(config.buildDir, "assets");
  const indexPath = join(config.buildDir, "index.html");
  const hostPrefixes = config.hostBackendPrefixes ?? [];

  const serve = (resp: DeckResponse) => intoResponse(resp, mode);

  async function serveIndex(missing: string): Promise<Response> {
    const body = await readOpt(indexPath);
    return body ? serve(html_route(body)) : serve(notFound(missing, mode));
  }

  async function serveApiCss(file: string, missing: string): Promise<Response> {
    const body =
      (await readOpt(join(config.frameworkAssetsDir, file))) ??
      (await readOpt(join(config.buildDir, "api", "assets", file)));
    return body ? serve(cssNoCache(body)) : serve(notFound(missing, mode));
  }

  return async function handle(req: Request): Promise<Response | null> {
    const url = new URL(req.url);
    const path = url.pathname;

    if (req.method !== "GET" && req.method !== "HEAD") return null;

    // 1. framework singletons (bake dir first, then self-contained app _deck/)
    const fw = FRAMEWORK_JS[path];
    if (fw) {
      const body =
        (await readOpt(join(config.frameworkAssetsDir, fw.bake))) ??
        (await readOpt(join(config.buildDir, "_deck", fw.bundled)));
      return body
        ? serve(jsAsset(body))
        : serve(notFound(`framework asset ${fw.bake} missing`, mode));
    }

    // 2. used icons from the app bake: /_deck/icons/<role>.svg
    if (path.startsWith("/_deck/icons/")) {
      const name = path.slice("/_deck/icons/".length).replace(/\.svg$/, "");
      if (!/^[a-z0-9-]{1,64}$/.test(name)) {
        return serve(notFound(`icon ${name} not found`, mode));
      }
      const body = await readOpt(join(config.buildDir, "_deck", "icons", `${name}.svg`));
      return body
        ? serve(svgAsset(body))
        : serve(notFound(`icon ${name} not in app output`, mode));
    }

    // 3. host-owned live-data under /_deck (SSE / data port)
    if (path.startsWith("/_deck/")) {
      const live = config.live ? await config.live(req, mode) : null;
      if (live) return live;
      return serve(notFound(`no deck route ${path}`, mode));
    }

    // 4. Deck-owned /api token + theme surface
    if (path === ROUTE_VIEW_TOKENS_CSS) {
      return serveApiCss("view-tokens.css", "view-tokens.css missing");
    }
    if (path === ROUTE_SAIL_CSS) {
      return serveApiCss("sail.css", "sail.css missing");
    }
    if (path === ROUTE_THEME_CSS) {
      const body =
        (await readOpt(join(config.frameworkAssetsDir, "theme.css"))) ??
        (await readOpt(join(config.buildDir, "api", "assets", "theme.css")));
      // deck-core default: empty override (base tokens apply).
      return serve(cssNoCache(body ?? ""));
    }
    if (path === ROUTE_THEMES_LIST) {
      const raw =
        (await readOpt(join(config.frameworkAssetsDir, "themes.json"))) ??
        (await readOpt(join(config.buildDir, "api", "v1", "themes")));
      if (raw) {
        try {
          return serve(jsonNoCache(JSON.parse(new TextDecoder().decode(raw))));
        } catch {
          return serve(notFound("themes list is not valid JSON", mode));
        }
      }
      // deck-core default: empty override list.
      return serve(jsonNoCache([]));
    }
    if (path.startsWith("/api")) {
      return serve(notFound(`no deck route ${path}`, mode));
    }

    // 5. per-app hashed assets: /<ns>/<file> -> <buildDir>/assets/<file>
    if (path.startsWith(`/${ns}/`)) {
      const rel = decodeURIComponent(path.slice(`/${ns}/`.length));
      const resolved = safeResolve(assetsDir, rel);
      if (!resolved) return serve(notFound("bad asset path", mode));
      const body = await readOpt(resolved);
      return body ? serve(assetByExt(rel, body)) : serve(notFound(`asset ${rel} not found`, mode));
    }

    // 6. index shell
    if (path === "/") {
      return serveIndex("index.html not built");
    }

    // 7. Dotted path — a static asset (or Connect-RPC `*.*` path). Deck owns
    //    these; try the built SPA tree, then an honest 404. Never SPA-fallback.
    if (path.includes(".")) {
      const rel = decodeURIComponent(path.slice(1));
      const resolved = safeResolve(config.buildDir, rel);
      if (resolved) {
        const body = await readOpt(resolved);
        if (body) return serve(assetByExt(rel, body));
      }
      return serve(notFound(`asset ${rel} not found`, mode));
    }

    // 8. SPA fallback (pages.rs spa_fallback): GET navigations that are not
    //    deck-owned and not host-owned get the index shell. Host-owned prefixes
    //    defer (`null`) so a Hono/Bun host can serve them. Everything else is
    //    an honest 404 — never a silent SPA lie.
    const deckOwns = deckOwnsPath(path);
    const hostOwns = hostOwnsPath(path, hostPrefixes);
    if (req.method === "GET" && !deckOwns && !hostOwns) {
      return serveIndex("index.html not built");
    }
    if (hostOwns) return null;
    return serve(notFound("not found", mode));
  };
}
