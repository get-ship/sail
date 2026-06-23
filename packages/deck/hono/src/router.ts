// The deck serve router — a framework-agnostic `Request -> Response | null`
// handler. It mirrors the GET serve surface of `deck-axum` (crates/deck/
// deck-axum/src/lib.rs router()) for an app built with `deck build` (the SPA
// model): the /_deck/* framework singletons, the per-app hashed assets, the
// index.html shell, and the SPA fallback. A `null` return means "not a deck
// route" — the host (e.g. ship-ops's Hono app) owns it (/api, /mcp, mutations).
//
// The live-data SSE / data port (/_deck/views/{name}/sse, /_deck/data) is a
// separate concern injected via `config.live` (see live.ts); the host owns it
// because only the host has the data.

import { readFile } from "node:fs/promises";
import { join, normalize, sep } from "node:path";
import {
  type DaemonMode,
  type DeckResponse,
  cssAsset,
  htmlView,
  intoResponse,
  jsAsset,
  jsonAsset,
  notFound,
  svgAsset,
} from "./response.ts";

export interface DeckAdaptorConfig {
  /** The `deck build` output dir (`<project>/.deck`): index.html + assets/. */
  buildDir: string;
  /** deck-core/assets/ — the baked framework singletons + live-reload.js + lucide-icons.json. */
  frameworkAssetsDir: string;
  /** package.json `name`; the URL asset namespace is its last '/'-segment. */
  appName: string;
  /** development (default) gates cache-control, ETag, and error redaction. */
  mode?: DaemonMode;
  /**
   * Optional live-data handler. Given a Request whose path is under /_deck and
   * not a static asset, it may return a Response (e.g. an SSE stream) or null
   * to fall through to an honest 404. Wired by the host to its data port.
   */
  live?: (req: Request, mode: DaemonMode) => Promise<Response | null> | Response | null;
}

// Framework singletons: /_deck/<route> -> deck-core/assets/<file> (assets.rs:318-387).
const FRAMEWORK_JS: Record<string, string> = {
  "/_deck/runtime.js": "sui-runtime.js",
  "/_deck/dom.js": "sui-dom.js",
  "/_deck/route.js": "deck-route.js",
  "/_deck/sdk-client.js": "sui-sdk-client.js",
  "/_deck/live-reload.js": "live-reload.js",
};

// SPA fallback exclusions (pages.rs:133-161): these prefixes keep an honest 404.
const RESERVED_PREFIXES = ["/api", "/_deck", "/hooks", "/auth", "/daemon", "/space", "/shell"];

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

/**
 * Build the pure deck serve handler. Returns a function that resolves a Request
 * to a Response for deck-owned routes, or `null` to defer to the host.
 */
export function createDeckRouter(config: DeckAdaptorConfig) {
  const mode: DaemonMode = config.mode ?? "development";
  const ns = config.appName.split("/").pop()!;
  const assetsDir = join(config.buildDir, "assets");
  const indexPath = join(config.buildDir, "index.html");

  const serve = (resp: DeckResponse) => intoResponse(resp, mode);

  return async function handle(req: Request): Promise<Response | null> {
    const url = new URL(req.url);
    const path = url.pathname;

    if (req.method !== "GET" && req.method !== "HEAD") return null;

    // 1. framework singletons
    const fw = FRAMEWORK_JS[path];
    if (fw) {
      const body = await readOpt(join(config.frameworkAssetsDir, fw));
      return body ? serve(jsAsset(body)) : serve(notFound(`framework asset ${fw} missing`, mode));
    }

    // 2. framework icons: /_deck/icons/<name>.svg (lucide set, deck-core/assets/lucide-icons.json)
    if (path.startsWith("/_deck/icons/")) {
      const name = path.slice("/_deck/icons/".length).replace(/\.svg$/, "");
      const svg = await lucideIcon(config.frameworkAssetsDir, name);
      return svg ? serve(svgAsset(svg)) : serve(notFound(`icon ${name} not found`, mode));
    }

    // 3. host-owned live-data under /_deck (SSE / data port)
    if (path.startsWith("/_deck/")) {
      const live = config.live ? await config.live(req, mode) : null;
      if (live) return live;
      return serve(notFound(`no deck route ${path}`, mode));
    }

    // 4. per-app hashed assets: /<ns>/<file> -> <buildDir>/assets/<file>
    if (path.startsWith(`/${ns}/`)) {
      const rel = decodeURIComponent(path.slice(`/${ns}/`.length));
      const resolved = safeResolve(assetsDir, rel);
      if (!resolved) return serve(notFound("bad asset path", mode));
      const body = await readOpt(resolved);
      return body ? serve(assetByExt(rel, body)) : serve(notFound(`asset ${rel} not found`, mode));
    }

    // 5. index shell
    if (path === "/") {
      const body = await readOpt(indexPath);
      return body ? serve(htmlView(body)) : serve(notFound("index.html not built", mode));
    }

    // 6. SPA fallback: GET nav to a client route -> the index shell.
    //    Excludes reserved prefixes and any path containing a '.' (asset/RPC).
    const isReserved = RESERVED_PREFIXES.some((p) => path === p || path.startsWith(p + "/"));
    if (!isReserved && !path.includes(".")) {
      const body = await readOpt(indexPath);
      if (body) return serve(htmlView(body));
    }

    // 7. not a deck route — let the host handle it (or 404 there)
    return null;
  };
}

// lucide icon lookup — deck-core bakes the set into lucide-icons.json
// ({ name: "<inner svg>" }) and wraps it (assets.rs:347-379). We mirror the wrap.
let _iconMap: Record<string, string> | null | undefined;
async function lucideIcon(frameworkAssetsDir: string, name: string): Promise<string | null> {
  if (!/^[a-z0-9-]{1,64}$/.test(name)) return null;
  if (_iconMap === undefined) {
    const raw = await readOpt(join(frameworkAssetsDir, "lucide-icons.json"));
    _iconMap = raw ? (JSON.parse(new TextDecoder().decode(raw)) as Record<string, string>) : null;
  }
  const inner = _iconMap?.[name];
  if (!inner) return null;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${inner}</svg>`;
}
