<!--
  <sail-mermaid> — mermaid source text → themed SVG. `value` is source (not URL).
  Lazy pack (`lazy: true`); async render into bindHtml via use:paintDiagram.
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

/** Dark surface via data-mode on <html>. */
function isDark(): boolean {
  return document.documentElement.dataset.mode === "dark";
}

function cssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

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
    const html = signal<string>("");
    const dark = signal<boolean>(isDark());
    const hasError = derived(() => error() !== "");

    const paintDiagram: Action<void> = (node) => {
      const stop = bindHtml(node as HTMLElement, () => html());
      return { destroy: stop };
    };

    mount((host: HTMLElement) => {
      // Monotonic seq: last started async render wins.
      let seq = 0;

      const syncTheme = (): void => {
        dark.set(isDark());
      };
      const obs = new MutationObserver(syncTheme);
      obs.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ["theme", "data-mode"],
      });
      document.addEventListener("sail-theme-change", syncTheme);

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
          if (mine !== seq) return;
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
<center use:paintDiagram hidden={hasError()} style({ box: w(full).minH(8) })></center>

<if {error()}>
  <column
    gap="2"
    style({
      box: w(full).pad(3).padX(4),
      paint: borderWidth(1).border(destructive).radius(md).bg(muted),
    })
  >
    <span style({ box: display(contents), text: fontSize(sm) })>
      <row gap="2" align="center" style({ paint: fg(destructive) })>
        <span style({ box: display(contents), text: fontWeight(semibold) })>
          <icon name="alert-triangle" label="" size="sm"></icon>
          Diagram failed to render
        </span>
      </row>
      <pre style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).whitespace(pre-wrap) })>{error()}</pre>
      <details>
        <summary style({ paint: cursor(pointer).fg(muted-foreground), text: fontSize(xs) })>mermaid source</summary>
        <pre
          style({
            box: mt(2).pad(2).padX(3).overflowX(auto),
            paint: bg(surface-2).borderWidth(1).border(border-subtle).radius(sm),
            text: fontFamily(mono).fontSize(xs).whitespace(pre-wrap),
          })
        >{value()}</pre>
      </details>
    </span>
  </column>
</if>
