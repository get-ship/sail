<!--
  <sail-mermaid> — mermaid source text → themed SVG. `value` is source (not URL).
  Lazy pack (`lazy: true`); async render into bindHtml via use:paintDiagram.

  C-MMD-1: never paint Mermaid's stock bomb ("Syntax error in text / mermaid
  version …"). suppressErrorRendering forces throws; we only bindHtml success
  SVG or our themed error tile. Init once (module); theme-only reconfigure.
-->

<scope>
  props: {
    value = ''
  }
</scope>

<controller>
import { bindHtml } from "@ship/sail-runtime/internal";
import type { Action } from "@ship/sail-runtime";
import mermaid from "mermaid";

type RenderResult =
  | { ok: true; svg: string }
  | { ok: false; error: string };

let mermaidReady = false;
let mermaidTheme: "dark" | "default" | null = null;

function ensureMermaid(theme: "dark" | "default"): void {
  if (mermaidReady && mermaidTheme === theme) return;
  mermaid.initialize({
    startOnLoad: false,
    securityLevel: "strict",
    suppressErrorRendering: true,
    theme,
    themeVariables: {
      fontFamily: hostCssVar("--font-body") || "inherit",
      background: "transparent",
    },
    flowchart: { useMaxWidth: true },
  });
  mermaidReady = true;
  mermaidTheme = theme;
}

function cleanMermaidTemp(id: string): void {
  for (const eid of [id, `d${id}`, `i${id}`]) {
    hostFindById(eid)?.remove?.()
  }
}

function isStockErrorSvg(svg: string): boolean {
  return /Syntax error in text/i.test(svg);
}

async function renderMermaid(src: string, dark: boolean): Promise<RenderResult> {
  const theme: "dark" | "default" = dark ? "dark" : "default";
  const id = `sail-mmd-${Math.random().toString(36).slice(2)}`;
  try {
    ensureMermaid(theme);
    const { svg } = await mermaid.render(id, src);
    if (!svg || isStockErrorSvg(svg)) {
      return { ok: false, error: !svg ? "Empty diagram SVG" : "Mermaid stock error diagram suppressed" };
    }
    return { ok: true, svg };
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) };
  } finally {
    cleanMermaidTemp(id);
  }
}

export default controller((ctx) => {
    const error = signal<string>("");
    const html = signal<string>("");
    const dark = signal<boolean>(hostIsDarkMode());
    const hasError = derived(() => error() !== "");

    const paintDiagram: Action<void> = (node) => {
      const stop = bindHtml(node, () => html());
      return { destroy: stop };
    };

    mount((host) => {
      let seq = 0;
      const syncTheme = (): void => { dark.set(hostIsDarkMode()); };
      const stopObs = hostObserveRootAttrs(["theme", "data-mode"], syncTheme);
      const stopListen = hostListenDocument("sail-theme-change", syncTheme);
      const stopRender = effect(() => {
        const src = (ctx.value() ?? "").trim();
        const dk = dark();
        if (!src) { html.set(""); error.set(""); return; }
        const mine = ++seq;
        void renderMermaid(src, dk).then((res) => {
          if (mine !== seq) return;
          if (!res.ok) (host).onError?.({ message: res.error, source: src });
          html.set(res.ok ? res.svg : "");
          error.set(res.ok ? "" : res.error);
        });
      });
      return () => { stopRender(); stopObs(); stopListen(); };
    });

    return { paintDiagram, html, dark, error, hasError }
  });
</controller>

<center use:paintDiagram hidden={hasError()} style({ size({ w: 100%, min-h: 8 }) })></center>

<if {error()}>
  <column
    gap="2"
    style({ size({ w: 100% }).pad({ all: 3, x: 4 }).border({ color: destructive, width: 1 }).radius(md).bg(muted) })
  >
    <span style({ display(contents).type({ size: sm }) })>
      <row gap="2" align="center" style({ fg(destructive) })>
        <span style({ display(contents).type({ weight: semibold }) })>
          <icon name="alert-triangle" label="" size="sm"></icon>
          Diagram failed to render
        </span>
      </row>
      <pre style({ fg(muted-foreground).type({ family: mono, size: xs, wrap: pre-wrap }) })>{error()}</pre>
      <details>
        <summary style({ cursor(pointer).fg(muted-foreground).type({ size: xs }) })>mermaid source</summary>
        <pre
          style({ pad({ all: 2, x: 3 }).margin({ t: 2 }).overflow({ x: auto }).bg(surface-2).border({ color: border-subtle, width: 1 }).radius(sm).type({ family: mono, size: xs, wrap: pre-wrap }) })
        >{value()}</pre>
      </details>
    </span>
  </column>
</if>
