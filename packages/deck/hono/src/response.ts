// DeckResponse — the transport-agnostic response port, ported from
// `crates/deck/deck-core/src/response.rs`. deck-core produces a DeckResponse;
// deck-axum's `into_axum` maps it 1:1 onto an axum Response. This module is the
// TS equivalent: a DeckResponse plus `intoResponse`, the 1:1 mapper onto a Web
// `Response`. The wire bytes (content types, cache-control, ETag, security
// headers) are part of the deck contract and are mirrored exactly.

export type DaemonMode = "development" | "production";

/** Cache posture, resolved against the running mode at serialize time. */
export type CachePolicy = "asset" | "view" | "none";

export interface DeckResponse {
  status: number;
  contentType: string;
  cachePolicy: CachePolicy;
  body: Uint8Array | string;
  /** Extra headers (e.g. Location). Security headers are added by `htmlView`. */
  extraHeaders: Array<[string, string]>;
}

// ---- content types (response.rs:115-140) ----
const CT_JS = "application/javascript; charset=utf-8";
const CT_CSS = "text/css; charset=utf-8";
const CT_HTML = "text/html; charset=utf-8";
const CT_JSON = "application/json; charset=utf-8";
const CT_SVG = "image/svg+xml; charset=utf-8";
const CT_TEXT = "text/plain; charset=utf-8";
const CT_SSE = "text/event-stream";

// ---- HTML security headers (response.rs:51-58) — order-pinned, html only ----
export const HTML_SECURITY_HEADERS: ReadonlyArray<readonly [string, string]> = [
  ["X-Frame-Options", "SAMEORIGIN"],
  ["X-Content-Type-Options", "nosniff"],
  [
    "Content-Security-Policy",
    "default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data:; connect-src 'self'; frame-ancestors 'self'",
  ],
];

// ---- cache-control bytes by policy x mode (response.rs:61-75) ----
function cacheControl(policy: CachePolicy, mode: DaemonMode): string {
  if (mode === "development") return "no-cache";
  switch (policy) {
    case "asset":
      return "public, immutable, max-age=31536000";
    case "view":
      return "must-revalidate, max-age=0";
    case "none":
      return "no-cache";
  }
}

// FNV-1a over the body, hex. The Rust side uses std DefaultHasher (SipHash,
// non-portable); the contract only requires a *stable strong ETag* per server,
// so any stable hash is acceptable — only presence and the `"..."` framing matter.
function strongEtag(body: Uint8Array): string {
  let h = 0x811c9dc5;
  for (let i = 0; i < body.length; i++) {
    h ^= body[i]!;
    h = Math.imul(h, 0x01000193);
  }
  return `"${(h >>> 0).toString(16)}"`;
}

function toBytes(body: Uint8Array | string): Uint8Array {
  return typeof body === "string" ? new TextEncoder().encode(body) : body;
}

// ---- constructors (response.rs:115-166) ----

export function jsAsset(body: Uint8Array | string): DeckResponse {
  return { status: 200, contentType: CT_JS, cachePolicy: "asset", body, extraHeaders: [] };
}
export function jsView(body: Uint8Array | string): DeckResponse {
  return { status: 200, contentType: CT_JS, cachePolicy: "view", body, extraHeaders: [] };
}
export function cssAsset(body: Uint8Array | string): DeckResponse {
  return { status: 200, contentType: CT_CSS, cachePolicy: "asset", body, extraHeaders: [] };
}
/** Tokens / theme CSS force no-cache regardless of mode (themes.rs:39-63). */
export function cssNoCache(body: Uint8Array | string): DeckResponse {
  return { status: 200, contentType: CT_CSS, cachePolicy: "none", body, extraHeaders: [] };
}
export function svgAsset(body: Uint8Array | string): DeckResponse {
  return { status: 200, contentType: CT_SVG, cachePolicy: "asset", body, extraHeaders: [] };
}
export function jsonAsset(value: unknown): DeckResponse {
  return { status: 200, contentType: CT_JSON, cachePolicy: "asset", body: JSON.stringify(value), extraHeaders: [] };
}
export function jsonNoCache(value: unknown): DeckResponse {
  return { status: 200, contentType: CT_JSON, cachePolicy: "none", body: JSON.stringify(value), extraHeaders: [] };
}
/** HTML view — the single mint point that appends HTML_SECURITY_HEADERS. */
export function htmlView(body: Uint8Array | string): DeckResponse {
  return {
    status: 200,
    contentType: CT_HTML,
    cachePolicy: "view",
    body,
    extraHeaders: HTML_SECURITY_HEADERS.map(([k, v]) => [k, v] as [string, string]),
  };
}
/** 404, mode-redacted (response.rs:149-160). */
export function notFound(msg: string, mode: DaemonMode): DeckResponse {
  return {
    status: 404,
    contentType: CT_TEXT,
    cachePolicy: "none",
    body: mode === "production" ? "not found" : msg,
    extraHeaders: [],
  };
}
/** 500, mode-redacted (response.rs:161-166). */
export function renderError(msg: string, mode: DaemonMode): DeckResponse {
  return {
    status: 500,
    contentType: CT_TEXT,
    cachePolicy: "none",
    body: mode === "production" ? "render error" : msg,
    extraHeaders: [],
  };
}
/** 303 redirect (setup.rs:43-46). */
export function redirect(location: string): DeckResponse {
  return { status: 303, contentType: CT_TEXT, cachePolicy: "none", body: "", extraHeaders: [["Location", location]] };
}

/** Raw SSE response (the live-data wire builds frames separately). */
export function sse(stream: ReadableStream<Uint8Array>): Response {
  return new Response(stream, {
    status: 200,
    headers: {
      "Content-Type": CT_SSE,
      "Cache-Control": "no-cache",
      Connection: "keep-alive",
    },
  });
}

// ---- the 1:1 mapper (into_axum, lib.rs:55-70) ----

export function intoResponse(resp: DeckResponse, mode: DaemonMode): Response {
  const bytes = toBytes(resp.body);
  const headers = new Headers();
  headers.set("Content-Type", resp.contentType);
  headers.set("Cache-Control", cacheControl(resp.cachePolicy, mode));
  // ETag only in Production, only for cacheable bodies (response.rs:101-104).
  if (mode === "production" && resp.cachePolicy !== "none") {
    headers.set("ETag", strongEtag(bytes));
  }
  for (const [k, v] of resp.extraHeaders) headers.append(k, v);
  // 303 carries no body. `bytes` is a Uint8Array — a valid Response body at
  // runtime; the cast bridges a TS-lib friction: TS 5.7+ types Uint8Array as
  // generic over ArrayBufferLike, which some consumer tsconfigs don't accept
  // as BodyInit. The adaptor must typecheck under any host's lib, so we coerce.
  const body: BodyInit | null = resp.status === 303 ? null : (bytes as unknown as BodyInit);
  return new Response(body, { status: resp.status, headers });
}
