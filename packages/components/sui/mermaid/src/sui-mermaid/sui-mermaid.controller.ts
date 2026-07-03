// <sui-mermaid> controller — render mermaid SOURCE TEXT to a themed SVG.
//
// The `value` prop is the mermaid SOURCE (not a URL — the caller fetches the
// .mmd and hands the text in). We render it to SVG, themed to the app
// (dark/light + app font), with an inline error box for bad syntax. On
// render failure we emit a `sui-mermaid-error` CustomEvent so a host can
// surface the failure (e.g. log/analytics, "open in editor" affordance).
//
// Async-render seam: mermaid.render() returns a Promise, so we can't bindHtml
// the source directly (bindHtml wants a SYNC string). Instead a render effect
// tracks `value` + the theme, runs the async render off to the side, and
// writes the resulting SVG (or an escaped error box) into a local `html`
// signal. A single bindHtml(container, () => html()) then morphs that into
// the container — the sanctioned, purity-lint-clean path for reactive
// innerHTML (a raw `el.innerHTML =` inside an effect is forbidden by
// sui-controller-purity).
//
// Stale renders are guarded by a monotonic seq (the last started render wins)
// so a fast theme/source flip never paints an earlier, slower result.
//
// Lazy loading: mermaid is a heavy dep. The pack's deck bundle is declared
// `lazy: true`, so importing `@ship/sui-mermaid` doesn't fetch mermaid at
// app boot — only when the bundle is first loaded (i.e. when the host
// element is first instantiated) does the mermaid module get pulled. The
// static import below is correct: the specifier is a literal, known at
// author time; the lazy-load boundary is the deck bundle, not the import
// expression.

import { controller, signal, effect, bindHtml } from "@ship/sui-runtime";
import mermaid from "mermaid";
import type { Scope } from "./sui-mermaid.sui";

// The slice of the mount `state` we touch: the `value` prop signal and the
// `bind:this={container}` ref, both exposed as callables on `$signals` (the
// mount-style controller contract — same seam <sui-markdown> uses).
interface MermaidState {
  $signals: {
    value: () => string | undefined;
    container: () => HTMLElement | null;
  };
}

type RenderResult =
  | { ok: true; svg: string }
  | { ok: false; error: string };

/** `.dark` on <html> is the app's dark-theme switch (base.css: `:root.dark`). */
function isDark(): boolean {
  return document.documentElement.classList.contains("dark");
}

/** A concrete CSS custom-property value off :root (cascades into shadow DOM). */
function cssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

function esc(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/** A legible, escaped error tile — never a blank/crashed render. */
function errorHtml(message: string, src: string): string {
  return (
    `<div class="mmd-err">` +
    `<div class="mmd-err-h">⚠ Diagram failed to render</div>` +
    `<pre class="mmd-err-msg">${esc(message)}</pre>` +
    `<details class="mmd-err-src"><summary>mermaid source</summary>` +
    `<pre>${esc(src)}</pre></details>` +
    `</div>`
  );
}

/** Initialize mermaid for the current theme + render `src` to an SVG string.
 *  Built-in `dark`/`default` themes give a clean, readable palette per mode; we
 *  layer the app font + a transparent background so the diagram sits on the card
 *  surface. securityLevel:"strict" sanitizes the output (safe for AI-authored
 *  source). Throws on bad syntax — caught into a typed result. */
async function renderMermaid(src: string, dark: boolean): Promise<RenderResult> {
  const theme: "dark" | "default" = dark ? "dark" : "default";
  try {
    mermaid.initialize({
      startOnLoad: false,
      securityLevel: "strict",
      theme,
      themeVariables: {
        fontFamily: cssVar("--font-body") || "inherit",
        background: "transparent",
      },
      flowchart: { useMaxWidth: true },
    });
    const id = `sui-mmd-${Math.random().toString(36).slice(2)}`;
    const { svg } = await mermaid.render(id, src);
    return { ok: true, svg };
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) };
  }
}

export default controller<Scope>({
  mount(state: MermaidState, host: HTMLElement) {
    const container = state.$signals.container();
    if (!container) return () => {};

    // The HTML to paint: the rendered SVG, an error box, or empty.
    const html = signal<string>("");
    // Reactive theme cell — flips when the app toggles dark/light so the render
    // effect re-fires and the diagram recolors to match.
    const dark = signal<boolean>(isDark());
    // Monotonic guard so a stale async render never overwrites a newer one.
    let seq = 0;

    // Watch the app theme: the `.dark`/`data-theme` swap on <html>, plus the
    // sui-theme-picker's `sui-theme-change` event.
    const syncTheme = (): void => dark.set(isDark());
    const obs = new MutationObserver(syncTheme);
    obs.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class", "data-theme"],
    });
    document.addEventListener("sui-theme-change", syncTheme);

    // Render effect — tracks `value` + `dark`; runs the async render and writes
    // the result into `html` (the bindHtml below morphs it into the container).
    const stopRender = effect(() => {
      const src = (state.$signals.value() ?? "").trim();
      const dk = dark();
      if (!src) {
        html.set("");
        return;
      }
      const mine = ++seq;
      void renderMermaid(src, dk).then((res) => {
        if (mine !== seq) return; // a newer render started — drop this one
        if (!res.ok) {
          // Surface the failure to the host — the ctx.host runtime fix landed,
          // so host.dispatchEvent is safe to use from the controller now.
          host.dispatchEvent(
            new CustomEvent("sui-mermaid-error", {
              detail: { message: res.error, source: src },
              bubbles: true,
              composed: true,
            }),
          );
        }
        html.set(res.ok ? res.svg : errorHtml(res.error, src));
      });
    });

    // The one sanctioned reactive innerHTML write (morph-based, identity-safe).
    const stopBind = bindHtml(container, () => html());

    return () => {
      stopRender();
      stopBind();
      obs.disconnect();
      document.removeEventListener("sui-theme-change", syncTheme);
    };
  },
});
