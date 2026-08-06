<!--
  <sail-mermaid> — the canonical mermaid render primitive.

  Takes mermaid SOURCE TEXT in its `value` prop (NOT a URL — the caller fetches
  the .mmd/.mermaid file and hands the text in) and renders the SVG into its
  shadow root, themed to the app (dark/light, app font), with an inline error
  state for bad syntax.

  Mermaid is heavy; the pack is its own deck bundle (`lazy: true`) and the
  controller dynamically imports 'mermaid' on first render, so the dep is only
  fetched when a diagram actually needs to draw.

  Rendering is async (mermaid.render returns a Promise) so we don't bindHtml the
  source directly: a render effect (tracks `value` + the theme) does the async
  render and writes the resulting SVG (or an escaped error box) into an `html`
  signal, which a bindHtml() binding morphs into the container <div> — the
  sanctioned, purity-lint-clean path for reactive innerHTML. The container
  reaches the binding through `use:paintDiagram`, so the paint owns its own
  teardown.
-->

<scope>
  props: {
    value = ''
  }
</scope>
<controller>
// <sail-mermaid> controller — render mermaid SOURCE TEXT to a themed SVG.
//
// The `value` prop is the mermaid SOURCE (not a URL — the caller fetches the
// .mmd and hands the text in). We render it to SVG, themed to the app
// (dark/light + app font), with an inline error box for bad syntax. On
// render failure we emit a `sail-mermaid-error` CustomEvent so a host can
// surface the failure (e.g. log/analytics, "open in editor" affordance).
//
// Async-render seam: mermaid.render() returns a Promise, so we can't bindHtml
// the source directly (bindHtml wants a SYNC string). Instead a render effect
// tracks `value` + the theme, runs the async render off to the side, and
// writes the resulting SVG (or an escaped error box) into a local `html`
// signal. The `use:paintDiagram` action bindHtml's that into the container
// element it receives — the sanctioned, purity-lint-clean path for reactive
// innerHTML (a raw `el.innerHTML =` inside an effect is forbidden by
// sail-controller-purity).
//
// Stale renders are guarded by a monotonic seq (the last started render wins)
// so a fast theme/source flip never paints an earlier, slower result.
//
// Lazy loading: mermaid is a heavy dep. The pack's deck bundle is declared
// `lazy: true`, so importing `@ship/sail-mermaid` doesn't fetch mermaid at
// app boot — only when the bundle is first loaded (i.e. when the host
// element is first instantiated) does the mermaid module get pulled. The
// static import below is correct: the specifier is a literal, known at
// author time; the lazy-load boundary is the deck bundle, not the import
// expression.

import { bindHtml } from "@ship/sail-runtime/internal";
import type { Action } from "@ship/sail-runtime";
import mermaid from "mermaid";

type RenderResult =
  | { ok: true; svg: string }
  | { ok: false; error: string };

/** Dark surface via data-mode (modes-in-file). No `.dark` class. */
function isDark(): boolean {
  const root = document.documentElement;
  return root.dataset.mode === "dark";
}

/** A concrete CSS custom-property value off :root (cascades into shadow DOM). */
function cssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
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
    const id = `sail-mmd-${Math.random().toString(36).slice(2)}`;
    const { svg } = await mermaid.render(id, src);
    return { ok: true, svg };
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) };
  }
}

export default controller((ctx) => {
    const error = signal<string>("");
    // The SVG mermaid produced: foreign markup, and the only thing bindHtml paints.
    const html = signal<string>("");
    // Reactive theme cell — flips when the app toggles dark/light so the render
    // effect re-fires and the diagram recolors to match.
    const dark = signal<boolean>(isDark());
    const hasError = derived(() => error() !== "");

    // The one sanctioned reactive innerHTML write (morph-based, identity-safe).
    // It carries the diagram and nothing else. The container owns its own
    // painting, so the theme/render wiring below no longer has to bail out
    // when the element hasn't landed yet.
    const paintDiagram: Action<void> = (node) => {
      const stop = bindHtml(node as HTMLElement, () => html());
      return { destroy: stop };
    };

    mount((host: HTMLElement) => {
      // Monotonic guard so a stale async render never overwrites a newer one.
      let seq = 0;

      // Watch the app theme: data-theme + data-mode on <html>, plus the
      // sail-theme-picker's `sail-theme-change` event.
      const syncTheme = (): void => {
        dark.set(isDark());
      };
      const obs = new MutationObserver(syncTheme);
      obs.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ["theme", "data-mode"],
      });
      document.addEventListener("sail-theme-change", syncTheme);

      // Render effect — tracks `value` + `dark`; runs the async render and writes
      // the result into `html` (paintDiagram's bindHtml morphs it into the DOM).
      const stopRender = effect(() => {
        const src = (ctx.value() ?? "").trim();
        const dk = dark();
        if (!src) {
          html.set("");
          error.set("");
          return;
        }
        const mine = ++seq;
        void renderMermaid(src, dk).then((res) => {
          if (mine !== seq) return; // a newer render started — drop this one
          if (!res.ok) {
            ;(host as any).onError?.({ message: res.error, source: src });
          }
          html.set(res.ok ? res.svg : "");
          error.set(res.ok ? "" : res.error);
        });
      });

      return () => {
        stopRender();
        obs.disconnect();
        document.removeEventListener("sail-theme-change", syncTheme);
      };
    });

    return {
      paintDiagram,
      html,
      dark,
      error,
      hasError,
    }
  });

</controller>
<center use:paintDiagram hidden={hasError()}>
  style({ box: w(full).minH(8) })
</center>

<if {error()}>
  <column gap="2">
    style({ box: w(full).pad(3).padX(4), paint: borderWidth(1).border(destructive).radius(md).bg(muted) })
    <span>
      style({ box: display(contents), text: fontSize(sm) })
      <row gap="2" align="center">
      style({ paint: fg(destructive) })
      <span>
        style({ box: display(contents), text: fontWeight(semibold) })
        <icon name="alert-triangle" label="" size="sm"></icon>
      Diagram failed to render
      </span>
</row>
    <pre>
      style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).whitespace(pre-wrap) })
{error()}</pre>
    <details>
      <summary>
        style({ paint: cursor(pointer).fg(muted-foreground), text: fontSize(xs) })
mermaid source</summary>
      <pre>
        style({ box: mt(2).pad(2).padX(3).overflowX(auto), paint: bg(surface-2).borderWidth(1).border(border-subtle).radius(sm), text: fontFamily(mono).fontSize(xs).whitespace(pre-wrap) })
{value()}</pre>
    </details>
    </span>
</column>
</if>
