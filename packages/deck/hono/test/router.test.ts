import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { Hono } from "hono";
import { deckMiddleware } from "../src/hono";
import { createDeckRouter, type DeckAdaptorConfig } from "../src/router";

const INDEX = "<!doctype html><html><body>SPA</body></html>";

function req(path: string, method = "GET"): Request {
  return new Request(`http://deck.test${path}`, { method });
}

async function bodyOf(res: Response | null): Promise<string | null> {
  return res ? res.text() : null;
}

describe("createDeckRouter — built SPA product serve", () => {
  let root: string;
  let buildDir: string;
  let frameworkAssetsDir: string;
  let base: DeckAdaptorConfig;
  let handle: ReturnType<typeof createDeckRouter>;

  beforeAll(() => {
    root = mkdtempSync(join(tmpdir(), "deck-hono-"));
    buildDir = join(root, "dist");
    frameworkAssetsDir = join(root, "fw");
    mkdirSync(join(buildDir, "assets"), { recursive: true });
    mkdirSync(join(buildDir, "_deck", "icons"), { recursive: true });
    mkdirSync(frameworkAssetsDir, { recursive: true });
    writeFileSync(join(buildDir, "index.html"), INDEX);
    writeFileSync(join(buildDir, "favicon.ico"), "ico");
    writeFileSync(join(buildDir, "assets", "entry.js"), "export const x = 1;\n");
    writeFileSync(join(buildDir, "_deck", "icons", "search.svg"), "<svg></svg>");
    writeFileSync(join(frameworkAssetsDir, "sail-runtime.js"), "export const runtime = 1;\n");
    writeFileSync(join(frameworkAssetsDir, "live-reload.js"), "export const reload = 1;\n");
    writeFileSync(join(frameworkAssetsDir, "error-overlay.js"), "export function attachErrorOverlay() {}\n");
    writeFileSync(join(frameworkAssetsDir, "view-tokens.css"), ":root { --fg: black }\n");
    writeFileSync(join(frameworkAssetsDir, "sail.css"), ".sail-spinner { display: inline-flex }\n");
    base = { buildDir, frameworkAssetsDir, appName: "demo" };
    handle = createDeckRouter(base);
  });

  afterAll(() => {
    rmSync(root, { recursive: true, force: true });
  });

  it("serves the index shell at /", async () => {
    const res = await handle(req("/"));
    expect(res?.status).toBe(200);
    expect(res?.headers.get("content-type")).toMatch(/text\/html/);
    expect(await bodyOf(res)).toBe(INDEX);
  });

  it("SPA-fallbacks GET navigations that are not deck-owned", async () => {
    for (const path of ["/studio/themes", "/settings", "/hooks/x", "/daemon", "/space/1", "/shell"]) {
      const res = await handle(req(path));
      expect(res?.status, path).toBe(200);
      expect(await bodyOf(res), path).toBe(INDEX);
    }
  });

  it("/auth/login is not claimed by Deck unless the host listed it", async () => {
    const standalone = await handle(req("/auth/login"));
    expect(standalone?.status).toBe(200);
    expect(await bodyOf(standalone)).toBe(INDEX);

    const hosted = createDeckRouter({ ...base, hostBackendPrefixes: ["/auth"] });
    const claimed = await hosted(req("/auth/login"));
    expect(claimed).toBeNull();

    const stillSpa = await hosted(req("/settings"));
    expect(stillSpa?.status).toBe(200);
    expect(await bodyOf(stillSpa)).toBe(INDEX);
  });

  it("does not SPA-fallback deck-owned /api or /_deck prefixes", async () => {
    const api = await handle(req("/api/unknown"));
    expect(api?.status).toBe(404);
    expect(await bodyOf(api)).not.toBe(INDEX);

    const deck = await handle(req("/_deck/not-a-singleton"));
    expect(deck?.status).toBe(404);
    expect(await bodyOf(deck)).not.toBe(INDEX);
  });

  it("does not SPA-fallback paths with a file extension (honest 404)", async () => {
    const res = await handle(req("/missing.js"));
    expect(res?.status).toBe(404);
    expect(await bodyOf(res)).not.toBe(INDEX);
  });

  it("serves public files from dist/ for dotted paths", async () => {
    const res = await handle(req("/favicon.ico"));
    expect(res?.status).toBe(200);
    expect(await bodyOf(res)).toBe("ico");
  });

  it("serves /assets/* from dist (base-assets layout)", async () => {
    const res = await handle(req("/assets/entry.js"));
    expect(res?.status).toBe(200);
    expect(res?.headers.get("content-type")).toMatch(/javascript/);
    expect(await bodyOf(res)).toContain("export const x");
  });

  it("serves the per-app hashed-asset namespace", async () => {
    const res = await handle(req("/demo/entry.js"));
    expect(res?.status).toBe(200);
    expect(await bodyOf(res)).toContain("export const x");
  });

  it("serves live-reload.js and error-overlay.js", async () => {
    const reload = await handle(req("/_deck/live-reload.js"));
    expect(reload?.status).toBe(200);
    expect(await bodyOf(reload)).toContain("reload");

    const overlay = await handle(req("/_deck/error-overlay.js"));
    expect(overlay?.status).toBe(200);
    expect(await bodyOf(overlay)).toContain("attachErrorOverlay");
  });

  it("serves /api token + theme routes", async () => {
    const tokens = await handle(req("/api/assets/view-tokens.css"));
    expect(tokens?.status).toBe(200);
    expect(tokens?.headers.get("content-type")).toMatch(/text\/css/);
    expect(tokens?.headers.get("cache-control")).toBe("no-cache");
    expect(await bodyOf(tokens)).toContain("--fg");

    const sail = await handle(req("/api/assets/sail.css"));
    expect(sail?.status).toBe(200);
    expect(await bodyOf(sail)).toContain(".sail-spinner");

    const theme = await handle(req("/api/assets/theme.css"));
    expect(theme?.status).toBe(200);
    expect(await bodyOf(theme)).toBe("");

    const list = await handle(req("/api/v1/themes"));
    expect(list?.status).toBe(200);
    expect(list?.headers.get("content-type")).toMatch(/json/);
    expect(await bodyOf(list)).toBe("[]");
  });

  it("defers live-data under /_deck to the host hook (no views/<name>/sse special case)", async () => {
    const live = createDeckRouter({
      ...base,
      live: (r) => {
        const p = new URL(r.url).pathname;
        if (p === "/_deck/data") return new Response("live-data", { status: 200 });
        return null;
      },
    });
    const data = await live(req("/_deck/data"));
    expect(data?.status).toBe(200);
    expect(await bodyOf(data)).toBe("live-data");

    const sse = await live(req("/_deck/views/clock/sse"));
    expect(sse?.status).toBe(404);
    expect(await bodyOf(sse)).not.toBe(INDEX);
  });

  it("does not claim non-GET methods", async () => {
    expect(await handle(req("/settings", "POST"))).toBeNull();
    expect(await handle(req("/auth/login", "PUT"))).toBeNull();
  });

  it("Hono middleware: listed /auth is host-owned; unlisted /auth is SPA", async () => {
    const hosted = new Hono();
    hosted.use("/*", deckMiddleware({ ...base, hostBackendPrefixes: ["/auth"] }));
    hosted.get("/auth/login", (c) => c.text("host-login"));

    const auth = await hosted.request("/auth/login");
    expect(auth.status).toBe(200);
    expect(await auth.text()).toBe("host-login");

    const spa = await hosted.request("/settings");
    expect(spa.status).toBe(200);
    expect(await spa.text()).toBe(INDEX);

    const standalone = new Hono();
    standalone.use("/*", deckMiddleware(base));
    standalone.get("/auth/login", (c) => c.text("host-login"));
    const stolen = await standalone.request("/auth/login");
    expect(stolen.status).toBe(200);
    expect(await stolen.text()).toBe(INDEX);
  });
});
